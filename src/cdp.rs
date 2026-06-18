use anyhow::{anyhow, bail, Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::js;
use crate::workspace::{clear_current_tab, debug, read_current_tab, write_current_tab};

/// Chrome-internal schemes that are never useful page targets for an agent.
pub const INTERNAL: [&str; 4] = [
    "chrome://",
    "chrome-untrusted://",
    "devtools://",
    "chrome-extension://",
];

pub fn is_internal_url(url: &str) -> bool {
    INTERNAL.iter().any(|p| url.starts_with(p))
}

pub struct Cdp {
    pub ws: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    id: u64,
    pub session: Option<String>,
    pub target: Option<String>,
}

impl Cdp {
    pub async fn connect() -> Result<Self> {
        let ws_url = ws_url().await?;
        let (ws, _) = tokio::time::timeout(cdp_timeout(), connect_async(&ws_url))
            .await
            .context("cdp connect timeout")?
            .with_context(|| format!("connect {ws_url}"))?;
        Ok(Self {
            ws,
            id: 0,
            session: None,
            target: None,
        })
    }

    pub async fn eval(&mut self, expression: &str) -> Result<Value> {
        let expression = js::wrap_return(expression);
        let r = self
            .call(
                "Runtime.evaluate",
                json!({"expression":expression,"awaitPromise":true,"returnByValue":true}),
                true,
            )
            .await?;
        if let Some(e) = r.get("exceptionDetails") {
            bail!("eval exception: {}", js_exception(e))
        }
        Ok(r["result"]
            .get("value")
            .cloned()
            .unwrap_or_else(|| r["result"].clone()))
    }

    pub async fn call(&mut self, method: &str, params: Value, attach: bool) -> Result<Value> {
        if attach && self.session.is_none() {
            self.attach_page(None).await?
        }
        let s = attach.then(|| self.session.as_ref().unwrap().clone());
        self.send(method, params, s).await
    }

    pub async fn send(
        &mut self,
        method: &str,
        params: Value,
        session: Option<String>,
    ) -> Result<Value> {
        self.id += 1;
        let mut req = json!({"id":self.id,"method":method,"params":params});
        if let Some(s) = session {
            req["sessionId"] = json!(s)
        }
        self.ws.send(Message::Text(req.to_string().into())).await?;
        loop {
            let msg = tokio::time::timeout(cdp_timeout(), self.ws.next())
                .await
                .with_context(|| format!("cdp timeout waiting for {method}"))?
                .context("cdp websocket closed")??;
            let Message::Text(txt) = msg else { continue };
            let v: Value = serde_json::from_str(&txt)?;
            if v["id"] == self.id {
                if let Some(e) = v.get("error") {
                    bail!("cdp error: {e}")
                }
                return Ok(v.get("result").cloned().unwrap_or(json!({})));
            }
        }
    }

    pub async fn attach_page(&mut self, target: Option<String>) -> Result<()> {
        self.attach_page_light(target).await?;
        let sid = self.session.as_ref().context("session")?.clone();
        self.send("Runtime.enable", json!({}), Some(sid.clone()))
            .await?;
        self.send("Page.enable", json!({}), Some(sid.clone()))
            .await?;
        self.send("Network.enable", json!({}), Some(sid.clone()))
            .await?;
        let _ = self.send("Log.enable", json!({}), Some(sid)).await;
        Ok(())
    }

    pub async fn attach_page_existing(&mut self, target: Option<String>) -> Result<()> {
        let id = match target {
            Some(t) => t,
            None => self
                .first_tab()
                .await?
                .context("no existing page target to attach")?,
        };
        let r = self
            .send(
                "Target.attachToTarget",
                json!({"targetId":id,"flatten":true}),
                None,
            )
            .await?;
        write_current_tab(&id)?;
        let sid = r["sessionId"].as_str().context("sessionId")?.to_string();
        self.session = Some(sid);
        self.target = Some(id);
        Ok(())
    }

    pub async fn attach_page_light(&mut self, target: Option<String>) -> Result<()> {
        let id = if let Some(t) = target {
            t
        } else {
            match self.first_tab().await? {
                Some(id) => id,
                None => self
                    .send("Target.createTarget", json!({"url":"about:blank"}), None)
                    .await?["targetId"]
                    .as_str()
                    .context("targetId")?
                    .into(),
            }
        };
        let r = self
            .send(
                "Target.attachToTarget",
                json!({"targetId":id,"flatten":true}),
                None,
            )
            .await?;
        write_current_tab(&id)?;
        let sid = r["sessionId"].as_str().context("sessionId")?.to_string();
        self.session = Some(sid);
        self.target = Some(id);
        Ok(())
    }

    pub async fn attach_frame(&mut self, target: &str) -> Result<()> {
        let id = self.resolve_target(target, &["iframe", "page"]).await?;
        let r = self
            .send(
                "Target.attachToTarget",
                json!({"targetId":id,"flatten":true}),
                None,
            )
            .await?;
        let sid = r["sessionId"].as_str().context("sessionId")?.to_string();
        self.session = Some(sid.clone());
        self.target = Some(id);
        self.send("Runtime.enable", json!({}), Some(sid.clone()))
            .await?;
        let _ = self.send("Page.enable", json!({}), Some(sid.clone())).await;
        let _ = self
            .send("Network.enable", json!({}), Some(sid.clone()))
            .await;
        let _ = self.send("Log.enable", json!({}), Some(sid)).await;
        Ok(())
    }

    pub async fn current_target(&mut self) -> Result<String> {
        if let Some(id) = read_current_tab() {
            let targets = self.send("Target.getTargets", json!({}), None).await?;
            if targets["targetInfos"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|t| t["targetId"].as_str() == Some(&id) && t["type"] == "page")
            {
                return Ok(id);
            }
            clear_current_tab()?;
        }
        self.first_tab().await?.context("tab not found")
    }

    pub async fn wait_network_idle(&mut self, timeout: Duration, idle: Duration) -> Result<bool> {
        if self.session.is_none() {
            self.attach_page(None).await?
        }
        let active = self.session.clone();
        let started = Instant::now();
        let min_observe = idle.max(Duration::from_millis(1000));
        let deadline = Instant::now() + timeout;
        let mut last_activity = Instant::now();
        let mut inflight = std::collections::HashSet::<String>::new();
        let mut perf_count = self.performance_resource_count().await.unwrap_or(0);
        loop {
            let now = Instant::now();
            if now.duration_since(started) >= min_observe
                && inflight.is_empty()
                && now.duration_since(last_activity) >= idle
            {
                return Ok(true);
            }
            if now >= deadline {
                return Ok(false);
            }
            let until_idle = idle.saturating_sub(now.duration_since(last_activity));
            let until_deadline = deadline.saturating_duration_since(now);
            let wait_for = if inflight.is_empty() {
                until_deadline.min(until_idle.max(Duration::from_millis(1)))
            } else {
                until_deadline.min(Duration::from_millis(250))
            };
            match tokio::time::timeout(wait_for, self.ws.next()).await {
                Err(_) => {
                    if let Ok(n) = self.performance_resource_count().await {
                        if n != perf_count {
                            perf_count = n;
                            last_activity = Instant::now();
                        }
                    }
                    continue;
                }
                Ok(None) => bail!("cdp websocket closed"),
                Ok(Some(Err(e))) => return Err(e.into()),
                Ok(Some(Ok(Message::Text(txt)))) => {
                    let v: Value = serde_json::from_str(&txt)?;
                    if v.get("id").is_some() {
                        continue;
                    }
                    if active.as_deref().is_some() && v["sessionId"].as_str() != active.as_deref() {
                        continue;
                    }
                    let Some(method) = v["method"].as_str() else {
                        continue;
                    };
                    let rid = v["params"]["requestId"].as_str().map(str::to_string);
                    match method {
                        "Network.requestWillBeSent" => {
                            if let Some(r) = rid {
                                inflight.insert(r);
                            }
                            last_activity = Instant::now();
                        }
                        "Network.loadingFinished" | "Network.loadingFailed" => {
                            if let Some(r) = rid {
                                inflight.remove(&r);
                            }
                            last_activity = Instant::now();
                        }
                        m if m.starts_with("Network.") => last_activity = Instant::now(),
                        _ => {}
                    }
                }
                Ok(Some(Ok(_))) => {}
            }
        }
    }

    async fn performance_resource_count(&mut self) -> Result<u64> {
        Ok(self
            .eval("performance.getEntriesByType('resource').length")
            .await?
            .as_u64()
            .unwrap_or(0))
    }

    pub async fn first_tab(&mut self) -> Result<Option<String>> {
        let v = self.send("Target.getTargets", json!({}), None).await?;
        let Some(xs) = v["targetInfos"].as_array() else {
            return Ok(None);
        };
        if let Some(id) = read_current_tab() {
            if xs
                .iter()
                .any(|t| t["targetId"].as_str() == Some(&id) && t["type"] == "page")
            {
                return Ok(Some(id));
            }
        }
        let pages: Vec<_> = xs
            .iter()
            .filter(|t| t["type"] == "page" && !is_internal_url(t["url"].as_str().unwrap_or("")))
            .collect();
        let chosen = pages
            .iter()
            .rev()
            .find(|t| {
                let u = t["url"].as_str().unwrap_or("");
                !u.is_empty() && u != "about:blank"
            })
            .or_else(|| pages.first());
        Ok(chosen
            .and_then(|t| t["targetId"].as_str())
            .map(str::to_string))
    }

    pub async fn resolve_tab(&mut self, target: &str) -> Result<String> {
        if target.len() > 8 && !target.chars().all(|c| c.is_ascii_digit()) {
            return Ok(target.into());
        }
        let idx: usize = target
            .parse()
            .with_context(|| format!("bad tab target {target:?}; use numeric index or targetId"))?;
        self.send("Target.getTargets", json!({}), None).await?["targetInfos"]
            .as_array()
            .and_then(|xs| xs.iter().filter(|t| t["type"] == "page").nth(idx))
            .and_then(|t| t["targetId"].as_str())
            .map(str::to_string)
            .context("tab not found")
    }

    pub async fn resolve_target(&mut self, target: &str, kinds: &[&str]) -> Result<String> {
        let targets = self.send("Target.getTargets", json!({}), None).await?;
        let xs = targets["targetInfos"]
            .as_array()
            .context("missing targetInfos")?;
        let mut matches = xs
            .iter()
            .filter(|t| kinds.iter().any(|kind| t["type"].as_str() == Some(*kind)));
        if let Some(t) = matches
            .clone()
            .find(|t| t["targetId"].as_str() == Some(target))
        {
            return t["targetId"]
                .as_str()
                .map(str::to_string)
                .context("targetId");
        }
        if let Ok(idx) = target.parse::<usize>() {
            return matches
                .nth(idx)
                .and_then(|t| t["targetId"].as_str())
                .map(str::to_string)
                .context("target not found");
        }
        matches
            .find(|t| {
                t["url"].as_str().unwrap_or("").contains(target)
                    || t["title"].as_str().unwrap_or("").contains(target)
            })
            .and_then(|t| t["targetId"].as_str())
            .map(str::to_string)
            .with_context(|| format!("target not found: {target}"))
    }
}

fn js_exception(details: &Value) -> String {
    let exc = &details["exception"];
    let desc = exc["description"]
        .as_str()
        .map(str::to_string)
        .or_else(|| exc.get("value").map(|v| crate::output::render(v.clone())))
        .or_else(|| exc["className"].as_str().map(str::to_string))
        .or_else(|| details["text"].as_str().map(str::to_string))
        .unwrap_or_else(|| "JavaScript evaluation failed".into());
    match (
        details["lineNumber"].as_u64(),
        details["columnNumber"].as_u64(),
    ) {
        (Some(l), Some(c)) => format!("{desc} (line {l}, column {c})"),
        _ => desc,
    }
}

pub async fn ws_url() -> Result<String> {
    for k in ["BROWSER_CONTROL_CDP_WS", "BU_CDP_WS"] {
        if let Ok(x) = std::env::var(k) {
            return Ok(x);
        }
    }
    for k in ["BROWSER_CONTROL_CDP_URL", "BU_CDP_URL"] {
        if let Ok(x) = std::env::var(k) {
            return resolve_http(&x)
                .await
                .or_else(|_| devtools_active_port_ws(&x));
        }
    }
    if let Ok(x) = devtools_active_port_ws("http://127.0.0.1:0") {
        return Ok(x);
    }
    for p in [9222, 9223] {
        if let Ok(x) = resolve_http(&format!("http://127.0.0.1:{p}")).await {
            return Ok(x);
        }
    }
    Err(anyhow!(
        "no CDP endpoint; set BROWSER_CONTROL_CDP_WS/URL or BU_CDP_WS/URL"
    ))
}

pub async fn page_ws_url(target: Option<String>) -> Result<String> {
    let mut bases = vec![];
    for k in ["BROWSER_CONTROL_CDP_URL", "BU_CDP_URL"] {
        if let Ok(x) = std::env::var(k) {
            bases.push(x);
        }
    }
    if bases.is_empty() {
        for p in [9222, 9223] {
            bases.push(format!("http://127.0.0.1:{p}"));
        }
    }
    let cli = reqwest::Client::builder()
        .timeout(Duration::from_secs(1))
        .build()?;
    for base in bases {
        debug(&format!(
            "page_ws: GET {}/json/list",
            base.trim_end_matches('/')
        ));
        let Ok(resp) = cli
            .get(format!("{}/json/list", base.trim_end_matches('/')))
            .send()
            .await
        else {
            continue;
        };
        let Ok(resp) = resp.error_for_status() else {
            continue;
        };
        let Ok(xs) = resp.json::<Value>().await else {
            continue;
        };
        let Some(arr) = xs.as_array() else {
            continue;
        };
        let chosen = arr
            .iter()
            .find(|t| target.as_deref().is_some() && t["id"].as_str() == target.as_deref())
            .or_else(|| arr.iter().find(|t| t["type"] == "page"));
        if let Some(ws) = chosen.and_then(|t| t["webSocketDebuggerUrl"].as_str()) {
            debug("page_ws: found page websocket");
            return Ok(ws.to_string());
        }
    }
    bail!("page websocket not found")
}

pub async fn send_page_command(ws_url: &str, method: &str, params: Value) -> Result<Value> {
    debug("page_ws: connecting");
    let (mut ws, _) = tokio::time::timeout(cdp_timeout(), connect_async(ws_url))
        .await
        .context("page cdp connect timeout")??;
    debug("page_ws: sending command");
    ws.send(Message::Text(
        json!({"id":1,"method":method,"params":params})
            .to_string()
            .into(),
    ))
    .await?;
    debug("page_ws: waiting for response");
    loop {
        let msg = tokio::time::timeout(cdp_timeout(), ws.next())
            .await
            .with_context(|| format!("cdp timeout waiting for {method}"))?
            .context("page cdp websocket closed")??;
        let Message::Text(txt) = msg else { continue };
        let v: Value = serde_json::from_str(&txt)?;
        if v["id"] == 1 {
            if let Some(e) = v.get("error") {
                bail!("cdp error: {e}")
            }
            return Ok(v.get("result").cloned().unwrap_or(json!({})));
        }
    }
}

pub fn cdp_timeout() -> Duration {
    Duration::from_secs(
        std::env::var("BROWSER_CONTROL_TIMEOUT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(15),
    )
}

pub async fn resolve_http(base: &str) -> Result<String> {
    let v: Value = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?
        .get(format!("{}/json/version", base.trim_end_matches('/')))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    v["webSocketDebuggerUrl"]
        .as_str()
        .map(str::to_string)
        .context("missing webSocketDebuggerUrl")
}

pub fn devtools_active_port_ws(base: &str) -> Result<String> {
    let port_hint = base
        .rsplit_once(':')
        .and_then(|(_, p)| p.trim_matches('/').parse::<u16>().ok());
    let port_hint = port_hint.filter(|p| *p != 0);
    let paths = [Path::new(".browser-control/profile/DevToolsActivePort")];
    for p in paths {
        let Ok(raw) = fs::read_to_string(p) else {
            continue;
        };
        let mut lines = raw.lines();
        let port: u16 = lines
            .next()
            .context("missing DevToolsActivePort port")?
            .parse()?;
        if port_hint.is_some() && port_hint != Some(port) {
            continue;
        }
        let ws_path = lines.next().context("missing DevToolsActivePort path")?;
        return Ok(format!("ws://127.0.0.1:{port}{ws_path}"));
    }
    bail!("DevToolsActivePort not found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_internal_urls() {
        assert!(is_internal_url("chrome://settings"));
        assert!(is_internal_url("devtools://devtools/bundled"));
        assert!(is_internal_url("chrome-extension://abc/page.html"));
        assert!(!is_internal_url("https://example.com"));
        assert!(!is_internal_url("about:blank"));
    }
}

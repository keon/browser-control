use anyhow::{bail, Context, Result};
use futures_util::StreamExt;
use serde_json::{json, Value};
use std::{
    collections::{HashSet, VecDeque},
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use tokio_tungstenite::tungstenite::Message;

#[cfg(unix)]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};

use crate::cdp::{cdp_timeout, Cdp};
use crate::lifecycle::pid_alive;
use crate::workspace::{debug, now_ms, read_current_tab};

const EVENT_RING_CAP: usize = 200;
const DETAIL_RING_CAP: usize = 100;

pub fn daemon_pid_path() -> &'static Path {
    Path::new(".browser-control/daemon.pid")
}
pub fn daemon_state_path() -> &'static Path {
    Path::new(".browser-control/daemon-state.json")
}
pub fn daemon_sock_path() -> &'static Path {
    Path::new(".browser-control/daemon.sock")
}

#[derive(Default)]
struct DaemonRings {
    events: VecDeque<Value>,
    network: VecDeque<Value>,
    console: VecDeque<Value>,
}

impl DaemonRings {
    fn clear(&mut self) {
        self.events.clear();
        self.network.clear();
        self.console.clear();
    }
}

fn push_ring(ring: &mut VecDeque<Value>, value: Value, cap: usize) {
    if ring.len() >= cap {
        ring.pop_front();
    }
    ring.push_back(value);
}

fn ring_value(ring: &VecDeque<Value>) -> Value {
    Value::Array(ring.iter().cloned().collect())
}

pub async fn daemon_loop() -> Result<()> {
    fs::create_dir_all(".browser-control")?;
    fs::write(daemon_pid_path(), std::process::id().to_string())?;
    let listener = daemon_listener()?;
    let mut c = Cdp::connect().await?;
    c.attach_page(None).await?;
    let mut inflight = HashSet::<String>::new();
    let mut last_activity = now_ms();
    let mut pending_dialog: Option<Value> = None;
    let mut rings = DaemonRings::default();
    loop {
        if let Some(target) = read_current_tab() {
            if c.target.as_deref() != Some(target.as_str())
                && c.attach_page(Some(target)).await.is_ok()
            {
                inflight.clear();
                rings.clear();
                last_activity = now_ms();
            }
        }
        if let Ok(Ok((mut stream, _))) =
            tokio::time::timeout(Duration::from_millis(20), listener.accept()).await
        {
            if let Err(e) = handle_daemon_stream(
                &mut c,
                &inflight,
                last_activity,
                &mut pending_dialog,
                &rings,
                &mut stream,
            )
            .await
            {
                debug(&format!("daemon command failed: {e}"));
            }
        }
        write_daemon_state(
            &c,
            &inflight,
            last_activity,
            pending_dialog.as_ref(),
            &rings,
        )?;
        match tokio::time::timeout(Duration::from_millis(80), c.ws.next()).await {
            Err(_) => continue,
            Ok(None) => bail!("cdp websocket closed"),
            Ok(Some(Err(e))) => return Err(e.into()),
            Ok(Some(Ok(Message::Text(txt)))) => {
                let v: Value = serde_json::from_str(&txt)?;
                if v.get("id").is_some() {
                    continue;
                }
                if c.session.as_deref().is_some() && v["sessionId"].as_str() != c.session.as_deref()
                {
                    continue;
                }
                let Some(method) = v["method"].as_str() else {
                    continue;
                };
                let method_name = method.to_string();
                let params = v["params"].clone();
                let entry = json!({
                    "ts": now_ms(),
                    "method": method_name,
                    "sessionId": v["sessionId"].as_str(),
                    "params": params,
                });
                push_ring(&mut rings.events, entry.clone(), EVENT_RING_CAP);
                if entry["method"]
                    .as_str()
                    .is_some_and(|m| m.starts_with("Network."))
                {
                    push_ring(&mut rings.network, entry.clone(), DETAIL_RING_CAP);
                }
                if entry["method"].as_str().is_some_and(|m| {
                    m == "Runtime.consoleAPICalled"
                        || m == "Runtime.exceptionThrown"
                        || m.starts_with("Log.")
                }) {
                    push_ring(&mut rings.console, entry, DETAIL_RING_CAP);
                }
                let params = v["params"].clone();
                let rid = params["requestId"].as_str().map(str::to_string);
                match method {
                    "Page.javascriptDialogOpening" => {
                        pending_dialog = Some(params);
                        last_activity = now_ms();
                    }
                    "Page.javascriptDialogClosed" => {
                        pending_dialog = None;
                        last_activity = now_ms();
                    }
                    "Network.requestWillBeSent" => {
                        if let Some(r) = rid {
                            inflight.insert(r);
                        }
                        last_activity = now_ms();
                    }
                    "Network.loadingFinished" | "Network.loadingFailed" => {
                        if let Some(r) = rid {
                            inflight.remove(&r);
                        }
                        last_activity = now_ms();
                    }
                    m if m.starts_with("Network.") || m.starts_with("Page.") => {
                        last_activity = now_ms();
                    }
                    _ => {}
                }
            }
            Ok(Some(Ok(_))) => {}
        }
    }
}

#[cfg(unix)]
fn daemon_listener() -> Result<UnixListener> {
    let path = daemon_sock_path();
    let _ = fs::remove_file(path);
    let listener = UnixListener::bind(path)?;
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    Ok(listener)
}

#[cfg(not(unix))]
fn daemon_listener() -> Result<()> {
    bail!("daemon IPC currently requires Unix sockets")
}

#[cfg(unix)]
async fn handle_daemon_stream(
    c: &mut Cdp,
    inflight: &HashSet<String>,
    last_activity: u64,
    pending_dialog: &mut Option<Value>,
    rings: &DaemonRings,
    stream: &mut UnixStream,
) -> Result<()> {
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await?;
    let v: Value = serde_json::from_slice(&raw)?;
    let response = match v["cmd"].as_str() {
        Some("ping") => json!({"ok":true,"result":{"pong":true,"pid":std::process::id()}}),
        Some("state") => {
            json!({"ok":true,"result":daemon_state_value(c, inflight, last_activity, pending_dialog.as_ref(), rings)})
        }
        Some("events") => json!({"ok":true,"result":ring_value(&rings.events)}),
        Some("network") => json!({"ok":true,"result":ring_value(&rings.network)}),
        Some("console") => json!({"ok":true,"result":ring_value(&rings.console)}),
        Some("cdp") => {
            let Some(method) = v["method"].as_str() else {
                return write_daemon_error(stream, "missing CDP method").await;
            };
            let params = v.get("params").cloned().unwrap_or_else(|| json!({}));
            let attach = v["attach"].as_bool().unwrap_or(true);
            match c.call(method, params, attach).await {
                Ok(r) => json!({"ok":true,"result":r}),
                Err(e) => json!({"ok":false,"error":e.to_string()}),
            }
        }
        Some("dialog") => {
            let accept = v["accept"].as_bool().unwrap_or(true);
            match c
                .send(
                    "Page.handleJavaScriptDialog",
                    json!({"accept":accept}),
                    c.session.clone(),
                )
                .await
            {
                Ok(r) => {
                    *pending_dialog = None;
                    json!({"ok":true,"result":r})
                }
                Err(e) => {
                    *pending_dialog = None;
                    json!({"ok":false,"error":e.to_string()})
                }
            }
        }
        Some(cmd) => json!({"ok":false,"error":format!("unknown daemon command {cmd}")}),
        None => json!({"ok":false,"error":"missing daemon command"}),
    };
    stream
        .write_all(serde_json::to_string(&response)?.as_bytes())
        .await?;
    stream.shutdown().await?;
    Ok(())
}

#[cfg(unix)]
async fn write_daemon_error(stream: &mut UnixStream, error: &str) -> Result<()> {
    stream
        .write_all(json!({"ok":false,"error":error}).to_string().as_bytes())
        .await?;
    stream.shutdown().await?;
    Ok(())
}

#[cfg(not(unix))]
async fn handle_daemon_stream(
    _c: &mut Cdp,
    _inflight: &HashSet<String>,
    _last_activity: u64,
    _pending_dialog: &mut Option<Value>,
    _rings: &DaemonRings,
    _stream: &mut (),
) -> Result<()> {
    bail!("daemon IPC currently requires Unix sockets")
}

#[cfg(unix)]
pub async fn daemon_command(mut cmd: Value) -> Result<Value> {
    ensure_daemon()?;
    cmd["id"] = json!(format!("{}-{}", std::process::id(), now_ms()));
    let mut stream = tokio::time::timeout(cdp_timeout(), UnixStream::connect(daemon_sock_path()))
        .await
        .context("daemon connect timeout")??;
    stream
        .write_all(serde_json::to_string(&cmd)?.as_bytes())
        .await?;
    stream.shutdown().await?;
    let mut raw = Vec::new();
    tokio::time::timeout(cdp_timeout(), stream.read_to_end(&mut raw))
        .await
        .context("daemon response timeout")??;
    let v: Value = serde_json::from_slice(&raw)?;
    if v["ok"].as_bool().unwrap_or(false) {
        return Ok(v["result"].clone());
    }
    bail!("{}", v["error"].as_str().unwrap_or("daemon command failed"))
}

#[cfg(not(unix))]
pub async fn daemon_command(_cmd: Value) -> Result<Value> {
    bail!("daemon IPC currently requires Unix sockets")
}

pub fn ensure_daemon() -> Result<()> {
    fs::create_dir_all(".browser-control")?;
    if daemon_ready() {
        return Ok(());
    }
    // Reap any stale daemon process before respawning.
    kill_recorded_daemon();
    let _ = fs::remove_file(daemon_pid_path());
    let _ = fs::remove_file(daemon_state_path());
    let _ = fs::remove_file(daemon_sock_path());
    let exe = std::env::current_exe()?;
    Command::new(exe)
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if daemon_ready() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    bail!("browser-control daemon did not start")
}

pub fn stop_daemon() {
    kill_recorded_daemon();
    let _ = fs::remove_file(daemon_pid_path());
    let _ = fs::remove_file(daemon_state_path());
    let _ = fs::remove_file(daemon_sock_path());
}

fn kill_recorded_daemon() {
    if let Ok(raw) = fs::read_to_string(daemon_pid_path()) {
        let pid = raw.trim().to_string();
        if !pid.is_empty() && pid_alive(&pid) {
            #[cfg(unix)]
            {
                let _ = Command::new("kill").args(["-TERM", &pid]).status();
            }
            #[cfg(windows)]
            {
                let _ = Command::new("taskkill").args(["/PID", &pid, "/F"]).status();
            }
            for _ in 0..20 {
                if !pid_alive(&pid) {
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

fn daemon_pid_alive() -> bool {
    fs::read_to_string(daemon_pid_path())
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .map(|p| pid_alive(&p))
        .unwrap_or(false)
}

pub fn daemon_ready() -> bool {
    daemon_fresh() && daemon_pid_alive() && daemon_sock_path().exists()
}

fn daemon_fresh() -> bool {
    let Ok(raw) = fs::read_to_string(daemon_state_path()) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<Value>(&raw) else {
        return false;
    };
    let Some(heartbeat) = v["heartbeat"].as_u64() else {
        return false;
    };
    now_ms().saturating_sub(heartbeat) < 2_000
}

pub async fn wait_network_idle_daemon(timeout: Duration, idle: Duration) -> Result<bool> {
    ensure_daemon()?;
    let started = Instant::now();
    let deadline = started + timeout;
    let min_observe = idle.max(Duration::from_millis(1000));
    let baseline = now_ms();
    while Instant::now() < deadline {
        let v = daemon_command(json!({"cmd":"state"})).await?;
        let heartbeat = v["heartbeat"].as_u64().unwrap_or(0);
        if now_ms().saturating_sub(heartbeat) > 2_000 {
            bail!("daemon heartbeat stale")
        }
        let inflight = v["inflight"].as_u64().unwrap_or(0);
        let last = v["lastActivity"].as_u64().unwrap_or(0);
        let observed = last >= baseline;
        if started.elapsed() >= min_observe
            && inflight == 0
            && observed
            && now_ms().saturating_sub(last) >= idle.as_millis() as u64
        {
            return Ok(true);
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Ok(false)
}

/// Pending dialog params if the daemon is live and a dialog is open.
/// Never auto-starts the daemon — this is a read-only probe.
pub async fn pending_dialog() -> Option<Value> {
    if !daemon_ready() {
        return None;
    }
    let state = daemon_command(json!({"cmd":"state"})).await.ok()?;
    if state["pendingDialog"].as_bool().unwrap_or(false) {
        Some(state["dialog"].clone())
    } else {
        None
    }
}

pub async fn dialog_action(action: &str) -> Result<()> {
    let accept = action != "dismiss";
    if daemon_command(json!({"cmd":"dialog","accept":accept}))
        .await
        .is_err()
    {
        let params = json!({"accept":accept});
        debug("dialog: resolving page websocket");
        if let Ok(ws) = crate::cdp::page_ws_url(read_current_tab()).await {
            debug("dialog: using page websocket");
            crate::cdp::send_page_command(&ws, "Page.handleJavaScriptDialog", params).await?;
        } else {
            debug("dialog: falling back to browser websocket");
            let mut c = Cdp::connect().await?;
            c.attach_page_existing(None).await?;
            c.send("Page.handleJavaScriptDialog", params, c.session.clone())
                .await?;
        }
    }
    println!("ok");
    Ok(())
}

fn write_daemon_state(
    c: &Cdp,
    inflight: &HashSet<String>,
    last_activity: u64,
    dialog: Option<&Value>,
    rings: &DaemonRings,
) -> Result<()> {
    fs::write(
        daemon_state_path(),
        serde_json::to_string(&daemon_state_value(
            c,
            inflight,
            last_activity,
            dialog,
            rings,
        ))?,
    )?;
    Ok(())
}

fn daemon_state_value(
    c: &Cdp,
    inflight: &HashSet<String>,
    last_activity: u64,
    dialog: Option<&Value>,
    rings: &DaemonRings,
) -> Value {
    json!({
        "pid": std::process::id(),
        "heartbeat": now_ms(),
        "targetId": c.target.as_ref(),
        "sessionId": c.session.as_ref(),
        "inflight": inflight.len(),
        "lastActivity": last_activity,
        "pendingDialog": dialog.is_some(),
        "dialog": dialog,
        "events": rings.events.len(),
        "networkEvents": rings.network.len(),
        "consoleEvents": rings.console.len(),
    })
}

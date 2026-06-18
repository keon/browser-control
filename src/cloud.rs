use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::process::Command;

use crate::output::println_json;

/// A remote cloud-browser provider. `browser-control` speaks plain CDP, so a
/// provider only needs to say how to authenticate and where the session
/// lifecycle endpoints live.
///
/// Built-in presets cover Browser Use, Steel, Hyperbrowser, and Browserbase.
/// Any field can be overridden via environment variables (see `resolve`), so
/// other providers work without a code change:
///   BROWSER_CONTROL_CLOUD_PROVIDER   preset name (default `browser-use`)
///   BROWSER_CONTROL_CLOUD_API        base URL
///   BROWSER_CONTROL_CLOUD_AUTH_HEADER auth header name
///   BROWSER_CONTROL_CLOUD_API_KEY    API key (else the preset's own env var)
///   BROWSER_CONTROL_CLOUD_PROJECT_ID project id (providers that need one)
struct Provider {
    name: String,
    base: String,
    auth_header: String,
    key_env: String,
    key: String,
    create_path: String,
    list_path: String,
    stop_method: String,
    /// `{id}` is replaced with the session id.
    stop_path: String,
    /// Candidate response keys that may hold the CDP/WebSocket URL.
    cdp_fields: Vec<String>,
    project_id: Option<String>,
}

fn env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

fn s(v: &str) -> String {
    v.to_string()
}

/// Build a provider from a preset name, before environment overrides.
fn preset(name: &str) -> Result<Provider> {
    let p = match name {
        "browser-use" | "browseruse" | "bu" => Provider {
            name: s("browser-use"),
            base: s("https://api.browser-use.com/api/v3"),
            auth_header: s("X-Browser-Use-API-Key"),
            key_env: s("BROWSER_USE_API_KEY"),
            key: String::new(),
            create_path: s("/browsers"),
            list_path: s("/profiles?pageSize=100&pageNumber=1"),
            stop_method: s("PATCH"),
            stop_path: s("/browsers/{id}"),
            cdp_fields: vec![s("cdpUrl")],
            project_id: None,
        },
        // docs: https://docs.steel.dev/api-reference (Sessions API)
        // POST/GET /v1/sessions, POST /v1/sessions/{id}/release, header
        // `Steel-Api-Key`, create response carries `websocketUrl`.
        "steel" => Provider {
            name: s("steel"),
            base: s("https://api.steel.dev/v1"),
            auth_header: s("Steel-Api-Key"),
            key_env: s("STEEL_API_KEY"),
            key: String::new(),
            create_path: s("/sessions"),
            list_path: s("/sessions"),
            stop_method: s("POST"),
            stop_path: s("/sessions/{id}/release"),
            cdp_fields: vec![s("websocketUrl"), s("connectUrl"), s("cdpUrl")],
            project_id: None,
        },
        // docs: https://hyperbrowser.ai/docs/reference/api-reference
        // POST /api/session, PUT /api/session/{id}/stop, header `x-api-key`,
        // create response carries `wsEndpoint`.
        "hyperbrowser" => Provider {
            name: s("hyperbrowser"),
            base: s("https://api.hyperbrowser.ai"),
            auth_header: s("x-api-key"),
            key_env: s("HYPERBROWSER_API_KEY"),
            key: String::new(),
            create_path: s("/api/session"),
            list_path: s("/api/sessions?status=active"),
            stop_method: s("PUT"),
            stop_path: s("/api/session/{id}/stop"),
            cdp_fields: vec![s("wsEndpoint")],
            project_id: None,
        },
        // docs: https://docs.browserbase.com/reference/api (Sessions)
        // POST/GET /v1/sessions, release via POST /v1/sessions/{id} with
        // {status:"REQUEST_RELEASE"}, header `X-BB-API-Key`, create response
        // carries `connectUrl`; projectId optional (inferred from key).
        "browserbase" => Provider {
            name: s("browserbase"),
            base: s("https://api.browserbase.com/v1"),
            auth_header: s("X-BB-API-Key"),
            key_env: s("BROWSERBASE_API_KEY"),
            key: String::new(),
            create_path: s("/sessions"),
            list_path: s("/sessions"),
            stop_method: s("POST"),
            stop_path: s("/sessions/{id}"),
            cdp_fields: vec![s("connectUrl")],
            project_id: None,
        },
        // Unknown name: a fully env-configured custom provider.
        other => Provider {
            name: s(other),
            base: env("BROWSER_CONTROL_CLOUD_API")
                .context("custom cloud provider needs BROWSER_CONTROL_CLOUD_API")?,
            auth_header: env("BROWSER_CONTROL_CLOUD_AUTH_HEADER").unwrap_or_else(|| s("Authorization")),
            key_env: s("BROWSER_CONTROL_CLOUD_API_KEY"),
            key: String::new(),
            create_path: env("BROWSER_CONTROL_CLOUD_CREATE_PATH").unwrap_or_else(|| s("/sessions")),
            list_path: env("BROWSER_CONTROL_CLOUD_LIST_PATH").unwrap_or_else(|| s("/sessions")),
            stop_method: env("BROWSER_CONTROL_CLOUD_STOP_METHOD").unwrap_or_else(|| s("DELETE")),
            stop_path: env("BROWSER_CONTROL_CLOUD_STOP_PATH").unwrap_or_else(|| s("/sessions/{id}")),
            cdp_fields: env("BROWSER_CONTROL_CLOUD_CDP_FIELD")
                .map(|f| vec![f])
                .unwrap_or_else(|| {
                    vec![s("cdpUrl"), s("wsEndpoint"), s("websocketUrl"), s("connectUrl")]
                }),
            project_id: None,
        },
    };
    Ok(p)
}

/// Resolve the active provider: preset selection plus environment overrides.
fn resolve() -> Result<Provider> {
    let name = env("BROWSER_CONTROL_CLOUD_PROVIDER").unwrap_or_else(|| s("browser-use"));
    let mut p = preset(&name)?;
    if let Some(v) = env("BROWSER_CONTROL_CLOUD_API") {
        p.base = v;
    }
    if let Some(v) = env("BROWSER_CONTROL_CLOUD_AUTH_HEADER") {
        p.auth_header = v;
    }
    p.key = env("BROWSER_CONTROL_CLOUD_API_KEY")
        .or_else(|| env(&p.key_env))
        .with_context(|| format!("{} (or BROWSER_CONTROL_CLOUD_API_KEY) missing", p.key_env))?;
    p.project_id = env("BROWSER_CONTROL_CLOUD_PROJECT_ID").or_else(|| env("BROWSERBASE_PROJECT_ID"));
    Ok(p)
}

async fn request(p: &Provider, path: &str, method: &str, body: Option<Value>) -> Result<Value> {
    let url = format!("{}{}", p.base, path);
    let cli = reqwest::Client::new();
    let req = match method {
        "GET" => cli.get(url),
        "PATCH" => cli.patch(url),
        "POST" => cli.post(url),
        "PUT" => cli.put(url),
        "DELETE" => cli.delete(url),
        _ => bail!("bad method {method}"),
    }
    .header(p.auth_header.as_str(), p.key.as_str())
    .header("Content-Type", "application/json");
    let req = if let Some(b) = body { req.json(&b) } else { req };
    let text = req.send().await?.error_for_status()?.text().await?;
    Ok(if text.trim().is_empty() {
        json!({})
    } else {
        serde_json::from_str(&text)?
    })
}

fn default_create_body(p: &Provider) -> Value {
    match p.name.as_str() {
        "browser-use" => json!({ "timeout": 240 }),
        "hyperbrowser" => json!({ "timeoutMinutes": 5 }),
        _ => json!({}),
    }
}

fn stop_body(p: &Provider) -> Option<Value> {
    match p.name.as_str() {
        "browser-use" => Some(json!({ "action": "stop" })),
        "browserbase" => {
            let mut o = json!({ "status": "REQUEST_RELEASE" });
            if let Some(pid) = &p.project_id {
                o["projectId"] = json!(pid);
            }
            Some(o)
        }
        _ => None,
    }
}

fn first_field<'a>(v: &'a Value, keys: &[String]) -> Option<&'a str> {
    keys.iter().find_map(|k| v.get(k).and_then(|x| x.as_str()))
}

/// List cloud sessions/profiles for the active provider.
pub async fn cloud_list() -> Result<()> {
    let p = resolve()?;
    let list_path = p.list_path.clone();
    println_json(request(&p, &list_path, "GET", None).await?)
}

/// Create a remote browser session and print the CDP endpoint to attach to.
pub async fn cloud_start(body: Option<&str>) -> Result<()> {
    let p = resolve()?;
    let mut b = body
        .map(serde_json::from_str)
        .transpose()?
        .unwrap_or_else(|| default_create_body(&p));
    if let (Some(pid), Some(obj)) = (&p.project_id, b.as_object_mut()) {
        obj.entry("projectId").or_insert_with(|| json!(pid));
    }
    let create_path = p.create_path.clone();
    let resp = request(&p, &create_path, "POST", Some(b)).await?;
    println_json(resp.clone())?;
    if let Some(cdp) = first_field(&resp, &p.cdp_fields) {
        // Providers return either a ws(s):// CDP url directly or an http
        // endpoint that needs /json/version resolution.
        let ws = if cdp.starts_with("ws") {
            cdp.to_string()
        } else {
            crate::cdp::resolve_http(cdp).await.unwrap_or_else(|_| cdp.to_string())
        };
        eprintln!("export BROWSER_CONTROL_CDP_WS={ws}");
    }
    Ok(())
}

/// Stop a remote browser session by id.
pub async fn cloud_stop(id: &str) -> Result<()> {
    let p = resolve()?;
    let path = p.stop_path.replace("{id}", id);
    let method = p.stop_method.clone();
    let body = stop_body(&p);
    println_json(request(&p, &path, &method, body).await?)
}

pub fn passthrough(bin: &str, args: &[&str]) -> Result<()> {
    let s = Command::new(bin).args(args).status()?;
    if !s.success() {
        bail!("{bin} exited with {s}")
    }
    Ok(())
}

pub fn sync_profile(
    profile: &str,
    browser: Option<&str>,
    cloud_profile_id: Option<&str>,
) -> Result<()> {
    let mut c = Command::new("profile-use");
    c.args(["sync", "--profile", profile]);
    if let Some(b) = browser {
        c.args(["--browser", b]);
    }
    if let Some(id) = cloud_profile_id {
        c.args(["--cloud-profile-id", id]);
    }
    let s = c.status()?;
    if !s.success() {
        bail!("profile-use exited with {s}")
    }
    Ok(())
}

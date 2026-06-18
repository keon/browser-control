use anyhow::{bail, Context, Result};
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

pub fn launch(url: Option<&str>, port: u16, headless: bool) -> Result<()> {
    let chrome = std::env::var("BROWSER_CONTROL_CHROME")
        .ok()
        .or_else(find_chrome)
        .context("Chrome not found; set BROWSER_CONTROL_CHROME")?;
    fs::create_dir_all(".browser-control/profile")?;
    let profile_dir = fs::canonicalize(".browser-control/profile")?;
    let profile_str = profile_dir.to_string_lossy().to_string();
    let mut cmd = Command::new(chrome);
    if headless {
        cmd.arg("--headless=new");
    }
    let child = cmd
        .arg(format!("--remote-debugging-port={port}"))
        .arg("--no-first-run")
        .arg("--no-default-browser-check")
        .arg(format!("--user-data-dir={profile_str}"))
        .arg(url.unwrap_or("about:blank"))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    fs::write(".browser-control/chrome.pid", child.id().to_string())?;
    fs::write(".browser-control/chrome.port", port.to_string())?;
    fs::write(".browser-control/chrome.user-data-dir", &profile_str)?;
    println!("BROWSER_CONTROL_CDP_URL=http://127.0.0.1:{port}");
    Ok(())
}

pub fn stop() -> Result<()> {
    crate::daemon::stop_daemon();
    let p = Path::new(".browser-control/chrome.pid");
    if !p.exists() {
        println!("no recorded browser-control browser");
        return Ok(());
    }
    let pid = fs::read_to_string(p)?.trim().to_string();
    if !pid_looks_owned(&pid) {
        if !pid_alive(&pid) {
            let _ = fs::remove_file(p);
        }
        bail!("refusing to kill stale/non-browser-control pid {pid}")
    }
    #[cfg(unix)]
    {
        let _ = Command::new("kill").args(["-TERM", &pid]).status();
        for _ in 0..30 {
            if !pid_alive(&pid) {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        if pid_alive(&pid) {
            let _ = Command::new("kill").args(["-KILL", &pid]).status();
        }
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill").args(["/PID", &pid, "/F"]).status();
    }
    if pid_alive(&pid) {
        bail!("failed to stop {pid}")
    } else {
        let _ = fs::remove_file(p);
        println!("stopped {pid}");
        Ok(())
    }
}

pub fn pid_looks_owned(pid: &str) -> bool {
    #[cfg(unix)]
    {
        let Ok(out) = Command::new("ps")
            .args(["-p", pid, "-o", "command="])
            .output()
        else {
            return false;
        };
        let cmd = String::from_utf8_lossy(&out.stdout);
        if !cmd.contains("--remote-debugging-port=") {
            return false;
        }
        let stored = fs::read_to_string(".browser-control/chrome.user-data-dir")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        if let Some(dir) = stored {
            cmd.contains(&format!("--user-data-dir={dir}"))
        } else {
            cmd.contains("--user-data-dir=") && cmd.contains(".browser-control/profile")
        }
    }
    #[cfg(not(unix))]
    {
        !pid.trim().is_empty()
    }
}

pub fn pid_alive(pid: &str) -> bool {
    #[cfg(unix)]
    {
        Command::new("kill")
            .args(["-0", pid])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        false
    }
}

pub fn find_chrome() -> Option<String> {
    for p in [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "google-chrome",
        "chromium",
        "chromium-browser",
    ] {
        if Path::new(p).exists()
            || Command::new(p)
                .arg("--version")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        {
            return Some(p.into());
        }
    }
    None
}

/// Read-only diagnostics. Fails (non-zero exit) only when CDP is unreachable;
/// missing daemon/launched-chrome are informational since both are optional.
pub async fn doctor() -> Result<()> {
    fn row(label: &str, ok: bool, detail: &str) {
        let mark = if ok { "ok  " } else { "FAIL" };
        let detail = if detail.is_empty() {
            String::new()
        } else {
            format!(" — {detail}")
        };
        println!("  [{mark}] {label}{detail}");
    }
    println!("browser-control doctor");
    let mut healthy = true;
    match crate::cdp::ws_url().await {
        Ok(ws) => {
            row("cdp endpoint", true, &ws);
            match crate::cdp::Cdp::connect().await {
                Ok(mut c) => match c
                    .call("Browser.getVersion", serde_json::json!({}), false)
                    .await
                {
                    Ok(v) => row("browser", true, v["product"].as_str().unwrap_or("")),
                    Err(e) => {
                        healthy = false;
                        row("browser", false, &e.to_string())
                    }
                },
                Err(e) => {
                    healthy = false;
                    row("cdp connect", false, &e.to_string())
                }
            }
        }
        Err(e) => {
            healthy = false;
            row("cdp endpoint", false, &e.to_string())
        }
    }
    match fs::read_to_string(".browser-control/chrome.pid")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        Some(pid) => row(
            "launched chrome",
            pid_alive(&pid),
            &format!(
                "pid {pid}{}",
                if pid_looks_owned(&pid) {
                    ""
                } else {
                    " (not owned)"
                }
            ),
        ),
        None => println!("  [ -- ] launched chrome — none recorded (external/attached browser)"),
    }
    if crate::daemon::daemon_ready() {
        row("daemon", true, "fresh heartbeat");
    } else {
        println!("  [ -- ] daemon — not running (auto-starts on actions)");
    }
    row(
        "workspace",
        Path::new(".browser-control").is_dir(),
        ".browser-control",
    );
    if !healthy {
        bail!("doctor: cdp unhealthy")
    }
    Ok(())
}

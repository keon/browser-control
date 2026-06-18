use anyhow::{bail, Result};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub fn debug(msg: &str) {
    if std::env::var("BROWSER_CONTROL_DEBUG").is_ok() {
        eprintln!("{msg}");
    }
}

pub fn init() -> Result<()> {
    for d in [
        "scripts",
        "helpers",
        "traces",
        "domain-skills",
        "interaction-skills",
    ] {
        fs::create_dir_all(format!(".browser-control/{d}"))?;
    }
    for root in asset_roots() {
        copy_dir(
            &root.join("skills/domain-skills"),
            Path::new(".browser-control/domain-skills"),
        )?;
        copy_dir(
            &root.join("skills/interaction-skills"),
            Path::new(".browser-control/interaction-skills"),
        )?;
    }
    fs::write(".browser-control/AGENTS.md", "# browser-control\n\nUse `browser-control snapshot` first. Use refs for simple actions. Use `eval` or `cdp` when stuck. Write reusable scripts in `.browser-control/scripts`. Verify before claiming success.\n")?;
    println!(".browser-control");
    Ok(())
}

pub fn asset_roots() -> Vec<PathBuf> {
    let mut roots = vec![];
    if let Ok(p) = std::env::var("BROWSER_CONTROL_ASSET_DIR") {
        roots.push(PathBuf::from(p));
    }
    roots.push(PathBuf::from("."));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(bin) = exe.parent() {
            roots.push(bin.to_path_buf());
            if let Some(prefix) = bin.parent().and_then(|p| p.parent()) {
                roots.push(prefix.to_path_buf());
                roots.push(prefix.join("share/browser-control"));
            }
        }
    }
    roots
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        let sp = e.path();
        let dp = dst.join(e.file_name());
        if sp.is_dir() {
            copy_dir(&sp, &dp)?;
        } else if !dp.exists() {
            fs::copy(&sp, &dp)?;
        }
    }
    Ok(())
}

pub fn current_tab_path() -> &'static Path {
    Path::new(".browser-control/current-tab")
}

pub fn read_current_tab() -> Option<String> {
    fs::read_to_string(current_tab_path())
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn write_current_tab(id: &str) -> Result<()> {
    fs::create_dir_all(".browser-control")?;
    fs::write(current_tab_path(), id)?;
    Ok(())
}

pub fn clear_current_tab() -> Result<()> {
    let _ = fs::remove_file(current_tab_path());
    Ok(())
}

pub fn run_stdin() -> Result<()> {
    let mut src = String::new();
    std::io::stdin().read_to_string(&mut src)?;
    if src.trim().is_empty() {
        bail!("no command and empty stdin")
    }
    fs::create_dir_all(".browser-control")?;
    let p = ".browser-control/stdin.sh";
    fs::write(p, src)?;
    run_script(p, &[])
}

pub fn run_script(script: &str, args: &[String]) -> Result<()> {
    let exe = std::env::current_exe()?;
    let mut c = match script.rsplit_once('.').map(|x| x.1) {
        Some("py") => {
            let mut c = Command::new("python3");
            c.arg(script);
            c
        }
        Some("js") => {
            let mut c = Command::new("node");
            c.arg(script);
            c
        }
        Some("ts") => {
            let mut c = Command::new("bun");
            c.arg(script);
            c
        }
        Some("sh") => {
            let mut c = Command::new("bash");
            c.arg(script);
            c
        }
        _ => Command::new(script),
    };
    let s = c
        .args(args)
        .env("BROWSER_CONTROL_BIN", exe)
        .env("BROWSER_CONTROL_WORKSPACE", ".browser-control")
        .status()?;
    if !s.success() {
        bail!("script exited with {s}")
    }
    Ok(())
}

pub async fn write_failure_trace(err: &anyhow::Error) -> Result<()> {
    if std::env::var("BROWSER_CONTROL_NO_TRACE").is_ok() || std::env::args().any(|a| a == "daemon")
    {
        return Ok(());
    }
    let dir = PathBuf::from(format!(".browser-control/traces/{}", now_ms()));
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("error.txt"), format!("{err:#}"))?;
    if !crate::daemon::daemon_ready() {
        return Ok(());
    }
    for (name, cmd) in [
        ("daemon-state.json", serde_json::json!({"cmd":"state"})),
        ("events.json", serde_json::json!({"cmd":"events"})),
        ("network.json", serde_json::json!({"cmd":"network"})),
        ("console.json", serde_json::json!({"cmd":"console"})),
    ] {
        if let Ok(v) = crate::daemon::daemon_command(cmd).await {
            fs::write(dir.join(name), serde_json::to_string_pretty(&v)?)?;
        }
    }
    debug(&format!("failure trace: {}", dir.display()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_roots_include_cwd() {
        assert!(asset_roots().iter().any(|p| p == Path::new(".")));
    }
}

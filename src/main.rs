mod actions;
mod cdp;
mod cloud;
mod daemon;
mod js;
mod lifecycle;
mod output;
mod workspace;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use serde_json::json;
use std::{fs, time::Duration};

use actions::{
    drag, element_center, mouse_click, press, ref_selector, upload, wait_element, wheel, xy,
};
use cdp::{cdp_timeout, Cdp};
use js::{inspect_js, PAGE_INFO_JS, SNAPSHOT_JS, WAIT_LOAD_JS};
use output::{
    parse_params, print_inspect, print_snapshot, print_tabs, println_json, render, write_b64,
};
use workspace::{clear_current_tab, write_current_tab};

#[derive(Parser)]
#[command(
    name = "browser-control",
    about = "Tiny CDP browser-control CLI for coding agents"
)]
struct Cli {
    #[arg(long, global = true)]
    from_stdin: bool,
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    Init,
    Launch {
        url: Option<String>,
        #[arg(long, default_value_t = 9222)]
        port: u16,
        #[arg(long)]
        headless: bool,
    },
    Status,
    Doctor,
    Stop,
    Reload,
    Cdp {
        method: String,
        params: Option<String>,
    },
    Events,
    Network,
    Console,
    DaemonState,
    Open {
        url: String,
    },
    Eval {
        js: String,
        #[arg(long)]
        frame: Option<String>,
    },
    Text,
    PageInfo,
    Inspect {
        #[arg(long)]
        json: bool,
        #[arg(short, long)]
        query: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long, default_value_t = 2500)]
        text: usize,
    },
    Snapshot {
        #[arg(long)]
        json: bool,
    },
    Observe {
        #[arg(long)]
        json: bool,
    },
    Wait {
        #[arg(default_value = "load")]
        mode: String,
        #[arg(default_value_t = 10)]
        seconds: u64,
    },
    WaitElement {
        selector: String,
        #[arg(default_value_t = 10)]
        seconds: u64,
        #[arg(long)]
        visible: bool,
    },
    Click {
        target: String,
        #[arg(long)]
        frame: Option<String>,
        #[arg(long, default_value = "left")]
        button: String,
        #[arg(long, default_value_t = 1)]
        clicks: u64,
        #[arg(long, default_value_t = 0)]
        wait: u64,
    },
    Fill {
        target: String,
        text: String,
        #[arg(long)]
        frame: Option<String>,
        #[arg(long, default_value_t = 0)]
        wait: u64,
    },
    Type {
        text: String,
    },
    Press {
        key: String,
    },
    Scroll {
        #[arg(default_value_t=-300)]
        dy: i64,
        #[arg(default_value_t = 0)]
        dx: i64,
        #[arg(long)]
        at: Option<String>,
    },
    Screenshot {
        path: Option<String>,
        #[arg(long)]
        full: bool,
    },
    Pdf {
        path: Option<String>,
    },
    Upload {
        selector: String,
        #[arg(required = true)]
        paths: Vec<String>,
    },
    Select {
        target: String,
        value: String,
    },
    Drag {
        from: String,
        to: String,
    },
    DownloadPath {
        path: String,
    },
    HttpGet {
        url: String,
    },
    Tabs,
    CurrentTab,
    NewTab {
        url: Option<String>,
    },
    SwitchTab {
        target: String,
    },
    CloseTab {
        target: Option<String>,
    },
    Frames,
    Cookies,
    Dialog {
        #[arg(default_value = "accept")]
        action: String,
    },
    Viewport {
        width: u64,
        height: u64,
    },
    LocalProfiles,
    SyncProfile {
        profile: String,
        #[arg(long)]
        browser: Option<String>,
        #[arg(long)]
        cloud_profile_id: Option<String>,
    },
    CloudProfiles,
    CloudStart {
        body: Option<String>,
    },
    CloudStop {
        id: String,
    },
    Run {
        script: String,
        args: Vec<String>,
    },
    #[command(hide = true)]
    Daemon,
}

#[tokio::main]
async fn main() -> Result<()> {
    if let Err(e) = run().await {
        let _ = workspace::write_failure_trace(&e).await;
        return Err(e);
    }
    Ok(())
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let Some(cmd) = cli.cmd else {
        if !cli.from_stdin {
            bail!("no command (use --from-stdin to execute stdin as a shell script)");
        }
        return workspace::run_stdin();
    };
    match &cmd {
        Cmd::Run { script, args } => return workspace::run_script(script, args),
        Cmd::Daemon => return daemon::daemon_loop().await,
        Cmd::Init => return workspace::init(),
        Cmd::Launch {
            url,
            port,
            headless,
        } => return lifecycle::launch(url.as_deref(), *port, *headless),
        Cmd::Stop | Cmd::Reload => return lifecycle::stop(),
        Cmd::Doctor => return lifecycle::doctor().await,
        Cmd::LocalProfiles => return cloud::passthrough("profile-use", &["list"]),
        Cmd::SyncProfile {
            profile,
            browser,
            cloud_profile_id,
        } => return cloud::sync_profile(profile, browser.as_deref(), cloud_profile_id.as_deref()),
        Cmd::CloudProfiles => return cloud::cloud_list().await,
        Cmd::CloudStart { body } => return cloud::cloud_start(body.as_deref()).await,
        Cmd::CloudStop { id } => return cloud::cloud_stop(id).await,
        Cmd::Dialog { action } => return daemon::dialog_action(action).await,
        Cmd::Events => return println_json(daemon::daemon_command(json!({"cmd":"events"})).await?),
        Cmd::Network => {
            return println_json(daemon::daemon_command(json!({"cmd":"network"})).await?)
        }
        Cmd::Console => {
            return println_json(daemon::daemon_command(json!({"cmd":"console"})).await?)
        }
        Cmd::DaemonState => {
            return println_json(daemon::daemon_command(json!({"cmd":"state"})).await?)
        }
        Cmd::HttpGet { url } => {
            println!("{}", reqwest::get(url).await?.text().await?);
            return Ok(());
        }
        _ => {}
    }
    if primes_daemon(&cmd) {
        let _ = daemon::ensure_daemon();
    }
    let mut c = Cdp::connect().await?;
    match cmd {
        Cmd::Init
        | Cmd::Launch { .. }
        | Cmd::Stop
        | Cmd::Reload
        | Cmd::Doctor
        | Cmd::Run { .. }
        | Cmd::LocalProfiles
        | Cmd::SyncProfile { .. }
        | Cmd::CloudProfiles
        | Cmd::CloudStart { .. }
        | Cmd::CloudStop { .. }
        | Cmd::Dialog { .. }
        | Cmd::Events
        | Cmd::Network
        | Cmd::Console
        | Cmd::DaemonState
        | Cmd::HttpGet { .. }
        | Cmd::Daemon => unreachable!(),
        Cmd::Status => {
            println!("cdp: ok");
            println_json(c.call("Browser.getVersion", json!({}), false).await?)?;
        }
        Cmd::Cdp { method, params } => {
            let attach = !(method.starts_with("Browser.") || method.starts_with("Target."));
            let params = parse_params(params)?;
            let method_for_daemon = method.clone();
            let r = match daemon::daemon_command(
                json!({"cmd":"cdp","method":method_for_daemon,"params":params.clone(),"attach":attach}),
            )
            .await
            {
                Ok(v) => v,
                Err(_) => c.call(&method, params, attach).await?,
            };
            println_json(r)?;
        }
        Cmd::Open { url } => {
            c.call("Page.navigate", json!({"url":url}), true).await?;
            wait_load(&mut c, cdp_timeout().as_secs()).await?;
            println!("ok");
        }
        Cmd::Eval { js, frame } => {
            if let Some(frame) = frame {
                c.attach_frame(&frame).await?;
            }
            println!("{}", render(c.eval(&js).await?))
        }
        Cmd::Text => println!("{}", render(c.eval("document.body.innerText").await?)),
        Cmd::PageInfo => {
            // An open alert/confirm/prompt freezes the page's JS thread, so an
            // eval would hang until timeout; report the dialog instead.
            if let Some(dialog) = daemon::pending_dialog().await {
                println_json(json!({"dialog": dialog}))?;
            } else {
                println_json(c.eval(PAGE_INFO_JS).await?)?;
            }
        }
        Cmd::Inspect {
            json,
            query,
            limit,
            text,
        } => {
            let v = c.eval(&inspect_js(query.as_deref(), limit, text)?).await?;
            if json {
                println_json(v)?
            } else {
                print_inspect(&v)
            }
        }
        Cmd::Snapshot { json } | Cmd::Observe { json } => {
            let v = c.eval(SNAPSHOT_JS).await?;
            if json {
                println_json(v)?
            } else {
                print_snapshot(&v)
            }
        }
        Cmd::Wait { mode, seconds } => {
            if mode.starts_with("network") {
                if !daemon::wait_network_idle_daemon(
                    Duration::from_secs(seconds),
                    Duration::from_millis(500),
                )
                .await
                .unwrap_or(false)
                    && !c
                        .wait_network_idle(Duration::from_secs(seconds), Duration::from_millis(500))
                        .await?
                {
                    bail!("network idle timeout after {seconds}s")
                }
            } else {
                wait_load(&mut c, seconds).await?;
            }
            println!("ok");
        }
        Cmd::WaitElement {
            selector,
            seconds,
            visible,
        } => {
            if !wait_element(&mut c, &selector, seconds, visible).await? {
                bail!("element not found after {seconds}s: {selector}")
            }
            println!("ok");
        }
        Cmd::Click {
            target,
            frame,
            button,
            clicks,
            wait,
        } => {
            if let Some(frame) = frame {
                c.attach_frame(&frame).await?;
            }
            if let Some((x, y)) = xy(&target) {
                mouse_click(&mut c, x, y, &button, clicks).await?
            } else {
                if wait > 0 && !wait_element(&mut c, &ref_selector(&target), wait, false).await? {
                    bail!("element not found after {wait}s: {target}")
                }
                let (x, y) = element_center(&mut c, &target).await?;
                mouse_click(&mut c, x, y, &button, clicks).await?
            }
            println!("ok");
        }
        Cmd::Fill {
            target,
            text,
            frame,
            wait,
        } => {
            if let Some(frame) = frame {
                c.attach_frame(&frame).await?;
            }
            if wait > 0 && !wait_element(&mut c, &ref_selector(&target), wait, false).await? {
                bail!("element not found after {wait}s: {target}")
            }
            c.eval(&format!(
                "({}).fill({},{})",
                js::ACTION_JS,
                serde_json::to_string(&target)?,
                serde_json::to_string(&text)?
            ))
            .await?;
            println!("ok");
        }
        Cmd::Type { text } => {
            c.call("Input.insertText", json!({"text":text}), true)
                .await?;
            println!("ok");
        }
        Cmd::Press { key } => {
            press(&mut c, &key).await?;
            println!("ok");
        }
        Cmd::Scroll { dy, dx, at } => {
            if let Some(at) = at {
                let (x, y) = xy(&at).context("--at must be x,y")?;
                wheel(&mut c, x, y, dx, dy).await?;
            } else {
                c.eval(&format!("window.scrollBy({},{}); true", dx, dy))
                    .await?;
            }
            println!("ok");
        }
        Cmd::Screenshot { path, full } => write_b64(
            path.unwrap_or("screenshot.png".into()),
            c.call(
                "Page.captureScreenshot",
                json!({"format":"png","captureBeyondViewport":full}),
                true,
            )
            .await?,
            "data",
        )?,
        Cmd::Pdf { path } => write_b64(
            path.unwrap_or("page.pdf".into()),
            c.call("Page.printToPDF", json!({"printBackground":true}), true)
                .await?,
            "data",
        )?,
        Cmd::Upload { selector, paths } => {
            upload(&mut c, &selector, &paths).await?;
            println!("ok");
        }
        Cmd::Select { target, value } => {
            c.eval(&format!(
                "({}).select({},{})",
                js::ACTION_JS,
                serde_json::to_string(&target)?,
                serde_json::to_string(&value)?
            ))
            .await?;
            println!("ok");
        }
        Cmd::Drag { from, to } => {
            let (x1, y1) = xy(&from).context("from must be x,y")?;
            let (x2, y2) = xy(&to).context("to must be x,y")?;
            drag(&mut c, x1, y1, x2, y2).await?;
            println!("ok");
        }
        Cmd::DownloadPath { path } => {
            fs::create_dir_all(&path)?;
            c.call("Browser.setDownloadBehavior", json!({"behavior":"allow","downloadPath":fs::canonicalize(path)?.to_string_lossy()}), false).await?;
            println!("ok");
        }
        Cmd::Tabs => print_tabs(c.call("Target.getTargets", json!({}), false).await?),
        Cmd::CurrentTab => {
            let id = c.current_target().await?;
            println_json(
                c.call("Target.getTargetInfo", json!({"targetId":id}), false)
                    .await?["targetInfo"]
                    .clone(),
            )?
        }
        Cmd::NewTab { url } => {
            // Create blank, attach, then navigate: passing url to createTarget
            // races with attach, so the brief about:blank is "complete" before
            // navigation starts and `wait load` returns too early.
            let r = c
                .call("Target.createTarget", json!({"url":"about:blank"}), false)
                .await?;
            let id = r["targetId"].as_str().context("targetId")?.to_string();
            write_current_tab(&id)?;
            c.attach_page(Some(id.clone())).await?;
            if let Some(url) = url.filter(|u| u != "about:blank") {
                c.call("Page.navigate", json!({"url":url}), true).await?;
                wait_load(&mut c, cdp_timeout().as_secs()).await?;
            }
            println_json(r)?;
        }
        Cmd::SwitchTab { target } => {
            let id = c.resolve_tab(&target).await?;
            c.call(
                "Target.activateTarget",
                json!({"targetId":id.clone()}),
                false,
            )
            .await?;
            write_current_tab(&id)?;
            println!("ok");
        }
        Cmd::CloseTab { target } => {
            let id = match target {
                Some(t) => c.resolve_tab(&t).await?,
                None => c.current_target().await?,
            };
            println_json(
                c.call("Target.closeTarget", json!({"targetId":id.clone()}), false)
                    .await?,
            )?;
            clear_current_tab()?;
        }
        Cmd::Frames => println_json(c.call("Page.getFrameTree", json!({}), true).await?)?,
        Cmd::Cookies => println_json(c.call("Storage.getCookies", json!({}), false).await?)?,
        Cmd::Viewport { width, height } => {
            c.call(
                "Emulation.setDeviceMetricsOverride",
                json!({"width":width,"height":height,"deviceScaleFactor":1,"mobile":false}),
                true,
            )
            .await?;
            println!("ok");
        }
    }
    Ok(())
}

async fn wait_load(c: &mut Cdp, seconds: u64) -> Result<()> {
    c.call(
        "Runtime.evaluate",
        json!({"expression":WAIT_LOAD_JS,"awaitPromise":true,"timeout":seconds*1000}),
        true,
    )
    .await?;
    Ok(())
}

fn primes_daemon(cmd: &Cmd) -> bool {
    matches!(
        cmd,
        Cmd::Open { .. }
            | Cmd::Click { .. }
            | Cmd::Fill { .. }
            | Cmd::Type { .. }
            | Cmd::Press { .. }
            | Cmd::Select { .. }
            | Cmd::Drag { .. }
            | Cmd::Upload { .. }
            | Cmd::Wait { .. }
            | Cmd::WaitElement { .. }
            | Cmd::SwitchTab { .. }
            | Cmd::NewTab { .. }
    )
}

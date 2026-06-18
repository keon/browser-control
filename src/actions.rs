use anyhow::{bail, Context, Result};
use serde_json::json;
use std::{
    fs,
    time::{Duration, Instant},
};

use crate::cdp::Cdp;
use crate::js::ACTION_JS;

pub fn xy(s: &str) -> Option<(i64, i64)> {
    let (a, b) = s.split_once(',')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

/// `@e3` snapshot refs become attribute selectors; everything else is CSS.
pub fn ref_selector(target: &str) -> String {
    match target.strip_prefix('@') {
        Some(r) => format!(r#"[data-browser-control-ref="{r}"]"#),
        None => target.to_string(),
    }
}

pub async fn mouse_click(c: &mut Cdp, x: i64, y: i64, button: &str, clicks: u64) -> Result<()> {
    if !["left", "right", "middle", "back", "forward"].contains(&button) {
        bail!("bad button {button:?}; use left/right/middle/back/forward")
    }
    for t in ["mousePressed", "mouseReleased"] {
        c.call(
            "Input.dispatchMouseEvent",
            json!({"type":t,"x":x,"y":y,"button":button,"clickCount":clicks}),
            true,
        )
        .await?;
    }
    Ok(())
}

pub async fn wheel(c: &mut Cdp, x: i64, y: i64, dx: i64, dy: i64) -> Result<()> {
    c.call(
        "Input.dispatchMouseEvent",
        json!({"type":"mouseWheel","x":x,"y":y,"deltaX":dx,"deltaY":dy}),
        true,
    )
    .await?;
    Ok(())
}

pub async fn element_center(c: &mut Cdp, target: &str) -> Result<(i64, i64)> {
    let v = c
        .eval(&format!(
            "({ACTION_JS}).point({})",
            serde_json::to_string(target)?
        ))
        .await?;
    let x = v["x"].as_i64().context("element x")?;
    let y = v["y"].as_i64().context("element y")?;
    Ok((x, y))
}

pub async fn drag(c: &mut Cdp, x1: i64, y1: i64, x2: i64, y2: i64) -> Result<()> {
    c.call(
        "Input.dispatchMouseEvent",
        json!({"type":"mousePressed","x":x1,"y":y1,"button":"left","clickCount":1}),
        true,
    )
    .await?;
    c.call(
        "Input.dispatchMouseEvent",
        json!({"type":"mouseMoved","x":x2,"y":y2,"button":"left"}),
        true,
    )
    .await?;
    c.call(
        "Input.dispatchMouseEvent",
        json!({"type":"mouseReleased","x":x2,"y":y2,"button":"left","clickCount":1}),
        true,
    )
    .await?;
    Ok(())
}

const MOD_ALT: u64 = 1;
const MOD_CTRL: u64 = 2;
const MOD_META: u64 = 4;
const MOD_SHIFT: u64 = 8;

/// `Enter`, `a`, `ctrl+a`, `cmd+shift+t`. Modifier names: alt/ctrl/meta(cmd)/shift.
pub fn parse_key_combo(spec: &str) -> Result<(u64, String)> {
    if !spec.contains('+') || spec.chars().count() == 1 {
        return Ok((0, spec.to_string()));
    }
    let (mods, key) = spec.rsplit_once('+').unwrap();
    // "ctrl++" means the key is the plus sign itself.
    let (mods, key) = if key.is_empty() {
        (mods.trim_end_matches('+'), "+")
    } else {
        (mods, key)
    };
    let mut bits = 0u64;
    for m in mods.split('+').filter(|s| !s.is_empty()) {
        bits |= match m.to_ascii_lowercase().as_str() {
            "alt" | "option" | "opt" => MOD_ALT,
            "ctrl" | "control" => MOD_CTRL,
            "meta" | "cmd" | "command" | "super" | "win" => MOD_META,
            "shift" => MOD_SHIFT,
            other => bail!("unknown modifier {other:?}; use alt/ctrl/meta/shift"),
        };
    }
    Ok((bits, key.to_string()))
}

/// key -> (windowsVirtualKeyCode, code, text). Special keys carry their codes
/// so listeners checking e.keyCode / e.code / e.key all fire.
pub fn key_info(key: &str) -> Option<(u64, String, Option<String>)> {
    let named = |vk: u64, code: &str, text: &str| {
        Some((
            vk,
            code.to_string(),
            (!text.is_empty()).then(|| text.to_string()),
        ))
    };
    match key {
        "Enter" => named(13, "Enter", "\r"),
        "Tab" => named(9, "Tab", "\t"),
        "Escape" | "Esc" => named(27, "Escape", ""),
        "Backspace" => named(8, "Backspace", ""),
        "Delete" => named(46, "Delete", ""),
        "Home" => named(36, "Home", ""),
        "End" => named(35, "End", ""),
        "PageUp" => named(33, "PageUp", ""),
        "PageDown" => named(34, "PageDown", ""),
        "ArrowLeft" => named(37, "ArrowLeft", ""),
        "ArrowUp" => named(38, "ArrowUp", ""),
        "ArrowRight" => named(39, "ArrowRight", ""),
        "ArrowDown" => named(40, "ArrowDown", ""),
        " " | "Space" => named(32, "Space", " "),
        _ if key.chars().count() == 1 => {
            let ch = key.chars().next()?;
            // CDP's windowsVirtualKeyCode for letter keys is the uppercase ASCII value.
            let vk = if ch.is_ascii_alphabetic() {
                ch.to_ascii_uppercase() as u64
            } else {
                ch as u64
            };
            let code = if ch.is_ascii_alphabetic() {
                format!("Key{}", ch.to_ascii_uppercase())
            } else if ch.is_ascii_digit() {
                format!("Digit{ch}")
            } else {
                key.to_string()
            };
            Some((vk, code, Some(key.to_string())))
        }
        _ => None,
    }
}

/// Editing command for a shortcut combo. Renderer-level: works headless and on
/// macOS, where e.g. Cmd+A is otherwise swallowed by the application menu.
pub fn editing_command(modifiers: u64, key: &str) -> Option<&'static str> {
    if modifiers & (MOD_CTRL | MOD_META) == 0 {
        return None;
    }
    match (
        key.to_ascii_lowercase().as_str(),
        modifiers & MOD_SHIFT != 0,
    ) {
        ("a", false) => Some("selectAll"),
        ("c", false) => Some("copy"),
        ("x", false) => Some("cut"),
        ("v", false) => Some("paste"),
        ("z", false) => Some("undo"),
        ("z", true) | ("y", false) => Some("redo"),
        _ => None,
    }
}

pub async fn press(c: &mut Cdp, spec: &str) -> Result<()> {
    let (modifiers, key) = parse_key_combo(spec)?;
    let (vk, code, text) = key_info(&key).with_context(|| format!("unknown key {key:?}"))?;
    // With Ctrl/Cmd/Alt held this is a shortcut, not typing: rawKeyDown with no
    // text, otherwise Chrome treats e.g. ctrl+a as a printable "a". Plain keys
    // carry text on keyDown only — a separate char event double-inserts.
    let shortcut = modifiers & (MOD_ALT | MOD_CTRL | MOD_META) != 0;
    let base = json!({"key":key,"code":code,"modifiers":modifiers,"windowsVirtualKeyCode":vk,"nativeVirtualKeyCode":vk});
    let mut down = base.clone();
    down["type"] = json!(if shortcut { "rawKeyDown" } else { "keyDown" });
    if shortcut {
        if let Some(cmd) = editing_command(modifiers, &key) {
            down["commands"] = json!([cmd]);
        }
    } else if let Some(t) = &text {
        down["text"] = json!(t);
    }
    c.call("Input.dispatchKeyEvent", down, true).await?;
    let mut up = base;
    up["type"] = json!("keyUp");
    c.call("Input.dispatchKeyEvent", up, true).await?;
    Ok(())
}

pub async fn upload(c: &mut Cdp, selector: &str, paths: &[String]) -> Result<()> {
    let doc = c.call("DOM.getDocument", json!({}), true).await?["root"]["nodeId"]
        .as_i64()
        .context("nodeId")?;
    let node = c
        .call(
            "DOM.querySelector",
            json!({"nodeId":doc,"selector":selector}),
            true,
        )
        .await?["nodeId"]
        .as_i64()
        .context("file input not found")?;
    let files = paths
        .iter()
        .map(|p| Ok(fs::canonicalize(p)?.to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>>>()?;
    c.call(
        "DOM.setFileInputFiles",
        json!({"nodeId":node,"files":files}),
        true,
    )
    .await?;
    Ok(())
}

pub async fn wait_element(
    c: &mut Cdp,
    selector: &str,
    seconds: u64,
    visible: bool,
) -> Result<bool> {
    let check = if visible {
        format!(
            "(()=>{{const e=document.querySelector({});if(!e)return false;\
             if(typeof e.checkVisibility==='function')return e.checkVisibility({{checkOpacity:true,checkVisibilityCSS:true}});\
             const s=getComputedStyle(e),r=e.getBoundingClientRect();return r.width>0&&r.height>0&&s.display!=='none'&&s.visibility!=='hidden'&&s.opacity!=='0'}})()",
            serde_json::to_string(selector)?
        )
    } else {
        format!(
            "!!document.querySelector({})",
            serde_json::to_string(selector)?
        )
    };
    let deadline = Instant::now() + Duration::from_secs(seconds);
    loop {
        // Eval can transiently fail mid-navigation; keep polling rather than aborting.
        let ok = c
            .eval(&check)
            .await
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if ok {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_coordinates() {
        assert_eq!(xy("10,20"), Some((10, 20)));
        assert_eq!(xy(" 1 , -2 "), Some((1, -2)));
        assert_eq!(xy("nope"), None);
    }

    #[test]
    fn maps_common_keys_without_zero_fallback() {
        assert_eq!(
            key_info("Enter"),
            Some((13, "Enter".into(), Some("\r".into())))
        );
        assert_eq!(key_info("ArrowDown"), Some((40, "ArrowDown".into(), None)));
        assert_eq!(
            key_info("a"),
            Some(('A' as u64, "KeyA".into(), Some("a".into())))
        );
        assert_eq!(
            key_info("Z"),
            Some(('Z' as u64, "KeyZ".into(), Some("Z".into())))
        );
        assert_eq!(
            key_info("1"),
            Some(('1' as u64, "Digit1".into(), Some("1".into())))
        );
        assert_eq!(key_info("DefinitelyNotAKey"), None);
    }

    #[test]
    fn parses_key_combos() {
        assert_eq!(parse_key_combo("a").unwrap(), (0, "a".into()));
        assert_eq!(parse_key_combo("Enter").unwrap(), (0, "Enter".into()));
        assert_eq!(parse_key_combo("+").unwrap(), (0, "+".into()));
        assert_eq!(parse_key_combo("ctrl+a").unwrap(), (2, "a".into()));
        assert_eq!(parse_key_combo("cmd+shift+t").unwrap(), (12, "t".into()));
        assert_eq!(
            parse_key_combo("alt+ArrowLeft").unwrap(),
            (1, "ArrowLeft".into())
        );
        assert_eq!(parse_key_combo("ctrl++").unwrap(), (2, "+".into()));
        assert!(parse_key_combo("hyper+a").is_err());
    }

    #[test]
    fn maps_editing_commands() {
        assert_eq!(editing_command(2, "a"), Some("selectAll"));
        assert_eq!(editing_command(4, "A"), Some("selectAll"));
        assert_eq!(editing_command(4 | 8, "z"), Some("redo"));
        assert_eq!(editing_command(2, "z"), Some("undo"));
        assert_eq!(editing_command(8, "a"), None); // shift alone is typing
        assert_eq!(editing_command(2, "q"), None);
    }

    #[test]
    fn ref_targets_become_attribute_selectors() {
        assert_eq!(ref_selector("@e3"), r#"[data-browser-control-ref="e3"]"#);
        assert_eq!(ref_selector("#submit"), "#submit");
    }
}

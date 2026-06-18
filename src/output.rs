use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine};
use serde_json::Value;
use std::fs;

pub fn parse_params(s: Option<String>) -> Result<Value> {
    Ok(match s {
        Some(x) => serde_json::from_str(&x)?,
        None => serde_json::json!({}),
    })
}

pub fn println_json(v: Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

pub fn render(v: Value) -> String {
    match v {
        Value::String(s) => s,
        Value::Null => String::new(),
        x => x.to_string(),
    }
}

pub fn write_b64(path: String, v: Value, key: &str) -> Result<()> {
    let data = v[key].as_str().context("missing data")?;
    fs::write(&path, general_purpose::STANDARD.decode(data)?)?;
    println!("{path}");
    Ok(())
}

pub fn print_snapshot(v: &Value) {
    println!("url: {}", v["url"].as_str().unwrap_or(""));
    println!("title: {}\n", v["title"].as_str().unwrap_or(""));
    for e in v["elements"].as_array().into_iter().flatten() {
        println!(
            "@{} {} {:?}",
            e["ref"].as_str().unwrap_or("?"),
            e["kind"].as_str().unwrap_or("el"),
            e["text"].as_str().unwrap_or("")
        );
    }
}

pub fn print_inspect(v: &Value) {
    println!("url: {}", v["url"].as_str().unwrap_or(""));
    println!("title: {}", v["title"].as_str().unwrap_or(""));
    println!("readyState: {}\n", v["readyState"].as_str().unwrap_or(""));
    if let Some(q) = v["query"].as_str().filter(|s| !s.is_empty()) {
        println!("query: {q}\n");
    }
    if let Some(t) = v["text"].as_str().filter(|s| !s.is_empty()) {
        println!("text:\n{t}\n");
    }
    let headings = v["headings"].as_array().into_iter().flatten();
    let mut any_heading = false;
    for h in headings {
        if !any_heading {
            println!("headings:");
            any_heading = true;
        }
        println!("- {}", h.as_str().unwrap_or(""));
    }
    if any_heading {
        println!();
    }
    if let Some(elements) = v["elements"].as_array() {
        if !elements.is_empty() {
            println!("elements:");
        }
        for e in elements {
            let href = e["href"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(|s| format!(" -> {s}"))
                .unwrap_or_default();
            println!(
                "@{} {} {:?}{}",
                e["ref"].as_str().unwrap_or("?"),
                e["kind"].as_str().unwrap_or("el"),
                e["text"].as_str().unwrap_or(""),
                href
            );
        }
    }
}

pub fn print_tabs(v: Value) {
    for (i, t) in v["targetInfos"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|t| t["type"] == "page")
        .enumerate()
    {
        println!(
            "{} {} {:?} {:?}",
            i,
            t["targetId"].as_str().unwrap_or(""),
            t["title"].as_str().unwrap_or(""),
            t["url"].as_str().unwrap_or("")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_json_params() {
        assert_eq!(parse_params(None).unwrap(), json!({}));
        assert_eq!(
            parse_params(Some(r#"{"x":1}"#.into())).unwrap(),
            json!({"x":1})
        );
        assert!(parse_params(Some("{".into())).is_err());
    }
}

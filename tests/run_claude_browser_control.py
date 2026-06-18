#!/usr/bin/env python3
"""Run a real Claude Code + browser-control smoke task.

This mirrors the shape of WebVoyager's Claude Code runner, but replaces MCP
browser tools with the small shell-native browser-control binary:

1. build browser-control
2. start a local fixture page and isolated headless Chrome/CDP endpoint
3. run Claude Code non-interactively with only Bash available
4. require Claude to drive the page through browser-control commands
5. independently verify the browser state and persist artifacts
"""

from __future__ import annotations

import argparse
import http.server
import json
import os
import pathlib
import select
import signal
import socket
import socketserver
import subprocess
import sys
import tempfile
import threading
import time
import urllib.request

PROJECT = pathlib.Path(__file__).resolve().parents[1]
ROOT = PROJECT.parent
BC = PROJECT / "target" / "debug" / "browser-control"
RESULTS = PROJECT / ".browser-control" / "claude-runs"
CHROME = os.environ.get(
    "BROWSER_CONTROL_CHROME",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
)

HTML = b"""<!doctype html>
<title>Claude browser-control smoke</title>
<script>console.log('claude-smoke-ready')</script>
<h1>Claude browser-control smoke</h1>
<button id="load" onclick="fetch('/secret').then(r=>r.text()).then(t=>{document.body.dataset.secret=t;document.querySelector('#secret').textContent=t})">Load secret</button>
<div id="secret"></div>
<label>Name <input id="answer"></label>
<label>Choice <select id="choice"><option value="a">A</option><option value="b">B</option></select></label>
<button id="submit" onclick="document.body.dataset.result=(document.querySelector('#answer').value==='Ada Lovelace'&&document.querySelector('#choice').value==='b'&&document.body.dataset.secret==='42')?'pass':'fail';document.querySelector('#result').textContent=document.body.dataset.result">Submit</button>
<div id="result"></div>
"""


class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *_: object) -> None:
        pass

    def do_GET(self) -> None:
        if self.path.startswith("/secret"):
            time.sleep(0.2)
            body = b"42"
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Content-Length", str(len(HTML)))
        self.end_headers()
        self.wfile.write(HTML)


def free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


def wait_cdp(port: int) -> None:
    for _ in range(80):
        try:
            urllib.request.urlopen(
                f"http://127.0.0.1:{port}/json/version", timeout=0.5
            ).read()
            return
        except Exception:
            time.sleep(0.1)
    raise RuntimeError("Chrome CDP did not start")


def run(cmd: list[str], *, env: dict[str, str], cwd: pathlib.Path, timeout: int = 60) -> str:
    p = subprocess.run(
        cmd, text=True, capture_output=True, env=env, cwd=cwd, timeout=timeout
    )
    if p.returncode:
        raise RuntimeError(
            f"command failed: {cmd}\nSTDOUT:\n{p.stdout}\nSTDERR:\n{p.stderr}"
        )
    return p.stdout.strip()


def format_stream_event(data: dict, seen: set[str]) -> str | None:
    msg_type = data.get("type")
    if msg_type == "system" and data.get("subtype") == "init":
        return f"[init] model={data.get('model', 'unknown')}"
    if msg_type == "assistant":
        message = data.get("message", {})
        content = message.get("content", [])
        lines: list[str] = []
        for item in content:
            if item.get("type") == "tool_use":
                key = f"tool:{item.get('id')}"
                if key in seen:
                    continue
                seen.add(key)
                lines.append(f"[tool] {item.get('name')}({json.dumps(item.get('input', {}))})")
            elif item.get("type") == "text" and item.get("text"):
                key = f"text:{hash(item.get('text'))}"
                if key in seen:
                    continue
                seen.add(key)
                lines.append(item["text"])
        return "\n".join(lines) if lines else None
    if msg_type == "user":
        out: list[str] = []
        for item in data.get("message", {}).get("content", []):
            if item.get("type") != "tool_result":
                continue
            result = item.get("content", "")
            if isinstance(result, list):
                result = "\n".join(
                    x.get("text", "") for x in result if x.get("type") == "text"
                )
            text = str(result).strip().replace("\n", "\\n")
            out.append(("[error] " if item.get("is_error") else "[result] ") + text[:500])
        return "\n".join(out) if out else None
    if msg_type == "result":
        return f"[result-summary] {data.get('subtype', '')} cost={data.get('total_cost_usd')}"
    return None


def run_claude(prompt: str, env: dict[str, str], cwd: pathlib.Path, out_dir: pathlib.Path, args: argparse.Namespace) -> dict:
    cmd = [
        "claude",
        "--model",
        args.model,
        "--print",
        "--output-format",
        "stream-json",
        "--verbose",
        "--no-session-persistence",
        "--tools",
        "Bash",
        "--allowed-tools",
        "Bash",
        "--permission-mode",
        args.permission_mode,
        "--max-budget-usd",
        str(args.max_budget_usd),
        prompt,
    ]
    (out_dir / "claude-command.json").write_text(json.dumps(cmd, indent=2))
    raw_path = out_dir / "claude-stream.jsonl"
    seen: set[str] = set()
    start = time.time()
    timed_out = False
    process = subprocess.Popen(
        cmd,
        env=env,
        cwd=cwd,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
        start_new_session=True,
    )
    assert process.stdout is not None
    with raw_path.open("w") as raw:
        while True:
            if time.time() - start > args.timeout:
                os.killpg(os.getpgid(process.pid), signal.SIGINT)
                timed_out = True
                break
            ready, _, _ = select.select([process.stdout], [], [], 1.0)
            if ready:
                line = process.stdout.readline()
                if not line:
                    break
                raw.write(line)
                raw.flush()
                line = line.rstrip()
                if not line:
                    continue
                try:
                    formatted = format_stream_event(json.loads(line), seen)
                except json.JSONDecodeError:
                    formatted = line
                if formatted:
                    print(formatted, flush=True)
            elif process.poll() is not None:
                break
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        os.killpg(os.getpgid(process.pid), signal.SIGTERM)
        process.wait()
    return {
        "returncode": process.returncode,
        "timed_out": timed_out,
        "duration_seconds": round(time.time() - start, 2),
        "stream": str(raw_path),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--timeout", type=int, default=300)
    parser.add_argument("--model", default="sonnet")
    parser.add_argument("--max-budget-usd", type=float, default=1.0)
    parser.add_argument("--permission-mode", default="dontAsk")
    parser.add_argument("--keep", action="store_true", help="leave temp workspace/chrome running")
    args = parser.parse_args()

    if not pathlib.Path(CHROME).exists():
        raise SystemExit(f"Chrome not found: {CHROME}")
    run(["cargo", "build", "--manifest-path", str(PROJECT / "Cargo.toml")], env=os.environ.copy(), cwd=ROOT, timeout=120)

    run_id = time.strftime("%Y%m%dT%H%M%S")
    out_dir = RESULTS / run_id
    out_dir.mkdir(parents=True, exist_ok=True)
    work = pathlib.Path(tempfile.mkdtemp(prefix="bc-claude-", dir=str(out_dir)))
    (work / "CLAUDE.md").write_text(
        "Use browser-control as the only browser automation surface. "
        "Do not edit files. Verify before final answer.\n"
    )

    with socketserver.ThreadingTCPServer(("127.0.0.1", 0), Handler) as srv:
        threading.Thread(target=srv.serve_forever, daemon=True).start()
        cdp_port = free_port()
        chrome = subprocess.Popen(
            [
                CHROME,
                "--headless=new",
                f"--remote-debugging-port={cdp_port}",
                "--no-first-run",
                "--no-default-browser-check",
                f"--user-data-dir={work / 'profile'}",
                "about:blank",
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        try:
            wait_cdp(cdp_port)
            url = f"http://127.0.0.1:{srv.server_address[1]}/"
            env = os.environ.copy()
            env["PATH"] = f"{BC.parent}:{env.get('PATH', '')}"
            env["BROWSER_CONTROL_BIN"] = str(BC)
            env["BROWSER_CONTROL_CDP_URL"] = f"http://127.0.0.1:{cdp_port}"
            env["BU_CDP_URL"] = env["BROWSER_CONTROL_CDP_URL"]
            env["BROWSER_CONTROL_TIMEOUT"] = "8"

            prompt = f"""You are testing browser-control from Claude Code.

Starting URL: {url}

Use Bash to run browser-control commands. Do not use Python, curl, Playwright,
Selenium, or direct HTTP to solve the page. Do not edit files.

Task:
1. Open the starting URL with browser-control.
2. Use browser-control observation/action commands to click "Load secret".
3. Wait for network idle.
4. Fill the Name input with exactly: Ada Lovelace
5. Select choice B.
6. Click Submit.
7. Verify with browser-control eval that document.body.dataset.result is "pass".

When verified, respond exactly: FINAL ANSWER: pass
"""
            (out_dir / "prompt.txt").write_text(prompt)
            print(f"Output: {out_dir}")
            print(f"Workspace: {work}")
            print(f"URL: {url}")
            claude_result = run_claude(prompt, env, work, out_dir, args)

            bc = lambda *a: run([str(BC), *a], env=env, cwd=work, timeout=30)
            state = json.loads(
                bc(
                    "eval",
                    "({result:document.body.dataset.result||'',secret:document.body.dataset.secret||'',answer:document.querySelector('#answer')?.value||'',choice:document.querySelector('#choice')?.value||''})",
                )
            )
            events = json.loads(bc("events"))
            network = json.loads(bc("network"))
            console = json.loads(bc("console"))
            verified = state == {
                "result": "pass",
                "secret": "42",
                "answer": "Ada Lovelace",
                "choice": "b",
            }
            result = {
                "ok": verified and claude_result["returncode"] == 0 and not claude_result["timed_out"],
                "verified": verified,
                "state": state,
                "events": len(events),
                "network": len(network),
                "console": len(console),
                "claude": claude_result,
                "output_dir": str(out_dir),
                "workspace": str(work),
                "url": url,
            }
            (out_dir / "result.json").write_text(json.dumps(result, indent=2, sort_keys=True))
            print(json.dumps(result, indent=2, sort_keys=True))
            return 0 if result["ok"] else 1
        finally:
            if not args.keep:
                chrome.terminate()
                try:
                    chrome.wait(timeout=3)
                except subprocess.TimeoutExpired:
                    chrome.kill()


if __name__ == "__main__":
    sys.exit(main())

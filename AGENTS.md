# browser-control agent guide

Use this project as a tiny browser pipe, not an agent framework.

Quick loop:

```bash
browser-control launch about:blank
export BROWSER_CONTROL_CDP_URL=http://127.0.0.1:9222
browser-control open https://example.com
browser-control inspect    # compact title/url/text/headings/actions/links
browser-control snapshot   # observe also works
browser-control click @e1
browser-control eval 'document.title'
browser-control events
browser-control network
browser-control console
browser-control cdp Browser.getVersion
```

Rules:
- Use `snapshot` before ref actions.
- Use `inspect` first when you need compact page text + candidate links/actions.
- Use refs for simple actions, selectors/coordinates when needed.
- Use `--wait N` on `click`/`fill` for late-rendered elements; `wait-element` for explicit polling.
- Use `press ctrl+a` / `press cmd+shift+t` style combos; editing combos (select-all/copy/paste/undo) work headless and on macOS.
- Use `scroll --at x,y` for inner scrollable containers; plain `scroll` for the window.
- Use `screenshot --full` for full-page captures; `click --clicks 2` / `--button right` for double/context clicks.
- `page-info` reports a pending `{dialog}` instead of hanging; clear it with `dialog accept|dismiss`.
- `eval` accepts top-level `return` (auto-wrapped in an IIFE).
- Use `eval` or `cdp` as the raw escape hatch.
- Use `events`, `network`, and `console` when diagnosing flaky waits/pages.
- Use `--frame <target-id|url-substring>` on `eval`, `click`, or `fill` for OOPIF/frame targets.
- Put reusable scripts in `.browser-control/scripts`.
- Verify with `text`, `eval`, `screenshot`, or raw CDP before saying done.
- Check `.browser-control/traces/*` after failures.
- Short env aliases `BU_CDP_WS` and `BU_CDP_URL` are accepted.

Lifecycle:

```bash
browser-control stop     # stop browser launched by browser-control
browser-control reload   # same minimal lifecycle reset
```

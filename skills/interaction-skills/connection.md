# Connection & Efficient CLI Use

`browser-control` is a CLI surface over one runner-owned browser. In eval/workspaces,
do **not** run lifecycle commands such as `launch`, `stop`, `reload`, `cloud-start`, or
`cloud-stop`; the runner has already connected Chrome/CDP for you.

## First moves

```sh
browser-control open "https://example.com"
browser-control inspect | head -120
browser-control observe | head -80   # when you need fresh clickable refs only
browser-control page-info
```

Use the current tab by default. Do not create extra tabs unless the task truly needs
parallel pages.

```sh
browser-control tabs
browser-control current-tab
browser-control open "https://example.com/next"   # navigates current tab
```

## Batch related actions

Every Claude Bash turn is expensive. Prefer one small shell batch for related browser
steps instead of many separate Bash calls.

```sh
browser-control --from-stdin <<'SH'
set -e
browser-control open "https://example.com"
browser-control wait load 10
browser-control eval "JSON.stringify({title: document.title, text: document.body.innerText.slice(0,1200)})"
SH
```

Keep batch output concise with `head`, `grep`, or compact JSON. Avoid printing whole
pages unless necessary.

## Prefer compact extraction over UI wandering

- Prefer `browser-control inspect` for the first read: it combines title, URL, text,
  headings, links, controls, and stable `@eN` refs.
- Use `browser-control inspect --query "phrase"` to get local text snippets and
  matching links/controls without dumping the full page.
- If a page already contains the needed data, use one `eval` that returns compact JSON.
- If the page is static or the URL was discovered in-browser, use
  `browser-control http-get <url>` for source/text extraction.
- Avoid opening many guessed URLs. First extract candidate links with one `eval`, then
  navigate only the best candidate.

Useful commands:

```sh
browser-control inspect --query "iPad"
browser-control observe --json
browser-control text | head -120
browser-control eval "Array.from(document.querySelectorAll('a')).slice(0,30).map(a=>a.textContent.trim()+' | '+a.href).join('\n')"
browser-control wait 2
browser-control wait-element "[data-testid='result']" 10 --visible
```

## Browser-control failure patterns

Refs (`@e12`) are snapshots, not durable IDs. Dynamic SPAs frequently re-render
after clicks, typing, date pickers, or result loads. If a click/fill says
`not found: eNN`, do **one** fresh `inspect`/`observe` and use the new ref; do
not retry the stale ref.

`press` takes only a key name, not a target:

```sh
browser-control click "@e12"
browser-control type "Chicago"
browser-control press Enter
```

Supported named keys are `Enter`, `Tab`, `Escape`/`Esc`, `Backspace`, `Delete`,
`Home`, `End`, `PageUp`, `PageDown`, and arrow keys. Do not use `Control+A`,
`KeyA`, or `press @e12 Enter`. To clear a custom input, prefer `fill`, site
remove buttons/chips, or a small DOM `eval`.

Eval snippets share the page's global JS context. Avoid top-level `const`/`let`
names that may be reused in later evals; wrap nontrivial snippets in an IIFE:

```sh
browser-control eval "(()=>{const links=[...document.querySelectorAll('a')];return JSON.stringify(links.slice(0,20).map(a=>({text:a.innerText,href:a.href})))})()"
```

CSS attribute values containing `/`, spaces, `*`, or punctuation must be quoted:
use `a[href*=\"/recipe/\"]`, not `a[href*=/recipe/]`.

Use only the browser-control surface for parsing too. Do not pipe page/API output
to Python/Node/Perl; if structured parsing is needed, use `browser-control eval`
on the page or keep shell filters simple (`grep`, `sed`, `head`).

# Scrolling

Prefer deterministic scroll commands and compact checks.

```sh
browser-control scroll -600     # down; negative dy matches mouse-wheel convention
browser-control scroll 600      # up
browser-control eval "window.scrollTo(0, document.body.scrollHeight); 'bottom'"
browser-control eval "JSON.stringify({y: scrollY, h: innerHeight, total: document.body.scrollHeight})"
```

After scrolling, verify with `observe | head -80` or a targeted `eval`; avoid repeated
full-page dumps.

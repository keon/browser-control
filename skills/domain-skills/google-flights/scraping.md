# Google Flights — CLI interaction guide

Google Flights is an SPA. Use `inspect`/`observe` for refs, then batch clicks and
small DOM checks.

Useful pattern:

```sh
browser-control open "https://www.google.com/travel/flights/"
browser-control inspect | head -120
```

For form state verification, query visible buttons/inputs rather than dumping the
whole page:

```sh
browser-control eval "JSON.stringify(Array.from(document.querySelectorAll('button,input,[role=button],[role=combobox]')).slice(0,80).map(e=>({text:(e.innerText||e.value||e.ariaLabel||e.placeholder||'').trim(), aria:e.getAttribute('aria-label'), role:e.getAttribute('role')})))"
```

Batch related actions with `--from-stdin`; verify the final state with one compact
`eval` or `inspect --query` before answering.

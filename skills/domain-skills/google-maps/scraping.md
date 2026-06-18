# Google Maps — CLI extraction guide

Maps is an SPA. Prefer direct `/maps/search/...` or `/maps/place/...` URLs, then
compact DOM/ARIA extraction. Do **not** use external search or local Python.

```sh
browser-control open "https://www.google.com/maps/search/QUERY"
browser-control wait 5
browser-control inspect --query "Reviews"
```

Useful compact extraction:

```sh
browser-control eval "JSON.stringify(Array.from(document.querySelectorAll('[aria-label],button,[role=tab]')).slice(0,160).map(e=>({text:(e.innerText||'').trim().slice(0,80), aria:e.getAttribute('aria-label'), role:e.getAttribute('role')})))"
```

For review distributions/accessibility/amenities, click the relevant tab once,
then extract visible text and `aria-label` values containing task keywords such
as `star`, `review`, `%`, `wheelchair`, `accessible`, `amenities`, `restroom`,
`parking`, or `Wi‑Fi`.

If the UI hides a panel after two attempts, switch strategy: use the current
visible DOM/ARIA labels and answer with the best official Maps evidence instead
of repeatedly scraping preview blobs or external search pages.

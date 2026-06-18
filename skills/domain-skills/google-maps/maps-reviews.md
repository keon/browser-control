# Maps Reviews

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. Maps search loaded a limited non-signed-in view with result list links, ratings, filters, and place pages. Place detail exposed name, rating, category, address, hours, website, phone, plus Overview/About tabs in visible text/ARIA.

Maps is an SPA. Prefer direct `/maps/search/...` or `/maps/dir/...` URLs, then
extract visible DOM/ARIA text. Keep one tab and avoid external sites.

## Search and routes

```sh
browser-control open "https://www.google.com/maps/search/QUERY"
browser-control inspect --query "rating"
```

Routes:

```sh
browser-control open "https://www.google.com/maps/dir/ORIGIN/DESTINATION"
browser-control inspect --query "Directions"
```

For route tasks, extract the route summary and then open/expand the step list
when step-by-step directions are requested. A single distance/time/highway
summary is not enough for a detailed-directions prompt.

## Compact extraction

```sh
browser-control eval "JSON.stringify([...document.querySelectorAll('[aria-label],a,button,[role=article],[role=feed]')].slice(0,200).map(e=>({text:(e.innerText||'').trim().slice(0,180),aria:e.getAttribute('aria-label'),href:e.href,role:e.getAttribute('role')})).filter(x=>x.text||x.aria||x.href))"
```

For place details, click the candidate once, then query fields:

```sh
browser-control inspect --query "Reviews"
browser-control inspect --query "Accessibility"
browser-control inspect --query "Amenities"
```

For reviews, open the Reviews tab and filter visible text for star count/one-star
comments. For share/print/settings tasks, use refs and stop after the visible
Maps-generated link/PDF/settings options are observed.

For address/location tasks, open the place detail pane and read the address field
or ARIA label. Do not answer with only "near <street>" when the task asks for an
exact address or location.

## Browser-control friction fixes

Maps frequently re-renders the result pane. Treat refs as temporary; after a
result click, filter change, or scroll, re-run a targeted `inspect`/`observe`
instead of retrying stale refs.

When using DOM selectors in `eval`, quote attribute values:

```sh
browser-control eval "JSON.stringify([...document.querySelectorAll('a[href*=\"/maps/place\"]')].slice(0,20).map(a=>({text:a.innerText,href:a.href})))"
```

Avoid bare `return` statements at top level in `eval`; wrap logic in an IIFE.
For result details, direct DOM/ARIA text extraction is usually faster than
clicking through multiple result cards.

For review-content tasks, the full review text matters. After sorting/filtering
reviews, click visible `More`/`See more` controls once and re-extract the review
card. If the requested review stays truncated, try another qualifying place or
state that the current Maps view only exposes truncated text; do not paraphrase
hidden text as if it were observed.

If a task only asks what a review says, prefer a qualifying place where a full
one-star review is visible. A truncated first sentence is not enough unless the
prompt allows a summary and the final answer clearly marks it as truncated.

When a task asks for a rating threshold and a specific food/service/category,
verify the place rating and the category/menu/search evidence on the place page
before reading reviews. A nearby qualifying alternative is acceptable only if it
meets every explicit constraint.

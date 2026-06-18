# Network Requests & Static Fetch

Use this when the UI is ambiguous, an SPA updates silently, or the answer is available
from HTML/API responses.

## Inspect browser-observed traffic

```sh
browser-control network | tail -80
browser-control console | tail -80
browser-control events | tail -80
```

Filter aggressively; raw network logs are noisy.

```sh
browser-control network | grep -Ei 'api|search|json|graphql|status' | tail -40
```

## Static fetch through browser-control

`browser-control http-get <url>` is the static-fetch path. Use it for
official/static pages or URLs discovered while browsing; do not use external
`curl`, `wget`, Python, Node, Selenium, or Playwright.

```sh
browser-control http-get "https://example.com/page" | grep -i "target phrase" | head -20
```

If browser navigation is blocked by a challenge but static HTML is available via
`http-get`, use `http-get` and cite the extracted page text in the final answer.

## Parsing without leaving browser-control

Do not pipe JSON/HTML into Python, Node, Perl, or local scripts during eval runs.
For official APIs on the same site, open that site first and parse with page JS:

```sh
browser-control open "https://huggingface.co/"
browser-control eval "fetch('/api/models?search=sentiment&sort=downloads&direction=-1&limit=10').then(r=>r.json()).then(d=>JSON.stringify(d.map(m=>({id:m.modelId,downloads:m.downloads,modified:m.lastModified,tags:m.tags?.slice(0,5)}))))"
```

For static HTML fetched through `http-get`, keep extraction simple and bounded:

```sh
browser-control http-get "https://example.com/page" | grep -i "target phrase" | head -20
browser-control http-get "https://example.com/page" | sed 's/<[^>]*>/ /g' | tr -s ' ' | head -c 3000
```

If an official API/page returns 5xx, retry once without a long sleep. If it still
fails, switch to the official browser-visible page or answer from the best
official evidence already observed; do not chain long `sleep` retries.

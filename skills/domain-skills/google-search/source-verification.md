# Source Verification

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. A headless Google Search request returned an “unusual traffic” page, while Google Maps/Flights loaded. Detect this blocker quickly; do not retry loops. If blocked, use official/authoritative target pages via browser-control rather than grinding Google.

Use Google as a discovery surface, then prefer the official/authoritative result.
Do not wander across many unofficial pages once a reliable snippet or source is
visible.

## Search

```sh
browser-control open "https://www.google.com/search?q=QUERY+WORDS"
browser-control inspect --query "KEY PHRASE"
```

For current/latest facts, include the date or season in the query and verify with
an official source when practical. For source-specific tasks, add `site:domain`.
Examples: `site:github.com`, `site:support.apple.com`, `site:imdb.com`,
`site:reddit.com/r/...`, `site:spotifycharts.com`.

For trend/time-window questions, apply `freshness.md` and use the requested
geography and time range exactly. Do not substitute a state for a city, past
7 days for monthly, or global results for a regional prompt unless the final
answer explicitly says the requested official surface was unavailable.

## Results extraction

```sh
browser-control eval "JSON.stringify([...document.querySelectorAll('a')].map(a=>({text:a.innerText.trim(),href:a.href})).filter(x=>x.text&&x.href).slice(0,40))"
```

If the answer appears in a knowledge panel/snippet, `inspect --query` can be
enough. If a target page is static/official, `browser-control http-get URL` is
allowed; otherwise open it and extract visible text. For login tasks, attempt the
provided form once and report the observed login result, without bypassing auth.

For event questions, distinguish completed events from scheduled future events.
If the prompt asks for "most recent" or "last", verify the event date is not in
the future relative to the run date before reporting a winner/result.

For sports/tournament finals, verify the winner/result on an official competition
or reputable event page and compare the final date to today's run date. Scheduled
venues for future finals do not answer "who won".

For current vs historical ambiguity, prefer official source pages and include the
observed date. If search snippets conflict with official pages, trust the
official page.

For ranked media/search questions, identify the ranking source and sort criterion
before listing results. If the task asks for user ratings, do not use popularity,
editorial picks, alphabetical lists, or arbitrary 100%-rated obscure items unless
the source page clearly sorts by that user-rating metric.

For astronomy/weather/visibility questions, define the requested geography from
the prompt before choosing dates. If the prompt says visible in a continent or
region, verify that each event is actually visible somewhere in that region, not
merely a global event of the same type.

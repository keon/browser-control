# Flight Search

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. Google Flights loaded without CAPTCHA and exposed visible controls for trip type, passenger count, cabin, origin (`Where from?`), and destination/date workflow. Results require SPA interaction and state verification.

Google Flights is an SPA. Use one tab, `inspect` for refs, batch actions, and
compact DOM checks. Do not retry the same picker path more than twice.

## Setup/search flow

```sh
browser-control open "https://www.google.com/travel/flights/"
browser-control inspect --query "Where from"
```

Use refs from `inspect`/`snapshot` for trip type, origin, destination, dates,
passengers, cabin, stops, price, emissions, duration, and sort. After entering a
city/date, verify visible chips/text before continuing.

## Compact state/result extraction

```sh
browser-control eval "JSON.stringify([...document.querySelectorAll('button,input,[role=button],[role=combobox],[aria-label]')].slice(0,160).map(e=>({text:(e.innerText||e.value||e.getAttribute('aria-label')||e.placeholder||'').trim().slice(0,120),aria:e.getAttribute('aria-label'),role:e.getAttribute('role')})))"
```

After results load:

```sh
browser-control inspect --query "Best departing flights"
browser-control inspect --query "Price"
browser-control eval "JSON.stringify([...document.querySelectorAll('[role=listitem], li, div')].map(x=>x.innerText.trim()).filter(t=>/\$|stops?|Nonstop|CO₂|emissions|duration|Airlines?/i.test(t)).slice(0,40))"
```

For booking-options tasks, select one flight and read the official booking panel.
For price graph/explore-map tasks, use visible Google Flights text and summarize
observed trends/options rather than scraping hidden state.

## Browser-control friction fixes

Google Flights re-renders aggressively. Refs go stale after almost every picker
interaction. If `click @eNN` or `fill @eNN` fails, run one fresh `observe` or use
a text/ARIA DOM query; never keep retrying the old ref.

For custom comboboxes, a reliable pattern is: click/focus, `type`, then choose
the visible option with DOM text. Do not use `Control+A` or `press @ref Enter`.

```sh
browser-control eval "(()=>{const x=[...document.querySelectorAll('input[aria-label=\"Where from?\"]')].find(e=>e.offsetParent);x.focus();x.click();return 'focused'})()"
browser-control type "Chicago"
browser-control wait 1
browser-control eval "(()=>{const o=[...document.querySelectorAll('[role=option]')].find(e=>/Chicago/.test(e.innerText)); if(o){o.click(); return o.innerText.slice(0,80)} return 'no option'})()"
```

Wrap all multi-step JS in IIFEs. Top-level `const d = ...` or `const els = ...`
will collide with earlier evals in the page context and cause "Identifier has
already been declared".

Dates can be hard when a run crosses midnight. If the prompt says "today", use
the runner's exact prompt date as the requested date, but if Google Flights says
that date is no longer selectable, state that and report the earliest available
same-route option instead of spending many turns fighting the date picker.

For direct search URLs, they are useful as a shortcut but brittle. Verify route,
trip type, class, and dates from visible text before answering.

For price graph tasks, keep the requested flight date separate from the graph
analysis window. If the prompt asks for "next two months", inspect prices across
roughly the next two calendar months from the requested departure context rather
than only adjacent days around the chosen flight date.

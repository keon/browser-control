# Computation Results

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. Wolfram Alpha query pages rendered pod headings as text, but mathematical answers were often `img[alt]` values, e.g. input interpretation, numeric result, derivative, and plot. Always extract image alt text in addition to visible text.

Use Wolfram Alpha official query pages. Many answers are rendered as pods with
text, images, or alt text; extract both visible text and image alt attributes.

## Query

```sh
browser-control open "https://www.wolframalpha.com/input?i=URL_ENCODED_QUERY"
browser-control wait 8
browser-control inspect --query "Result"
```

If the direct URL does not populate, use the homepage input with `fill`/`press`.
Keep the query close to the task wording, including units, dates, assumptions,
or “show steps/general solution” when requested.

## Pod extraction

```sh
browser-control eval "JSON.stringify([...document.querySelectorAll('section, [data-testid], img[alt], table')].map(e=>({text:(e.innerText||'').trim().slice(0,500),alt:e.getAttribute('alt')})).filter(x=>x.text||x.alt).slice(0,80))"
```

Run targeted reads for pod names if needed:

```sh
browser-control inspect --query "Input interpretation"
browser-control inspect --query "Result"
browser-control inspect --query "Solution"
browser-control inspect --query "Decimal approximation"
```

For plots/geometry/packing, use screenshot only after pods are visible and report
Wolfram's displayed values/labels. If a pod is hidden behind “more”, click it once
and re-extract; otherwise answer from visible official pods only.

## Failure guardrails

Answer the full expression the task requests, not just an intermediate value, and
extract `img[alt]` from Result/Solution pods when answers render as images.

Check directionality and units against the task wording before finalizing. Many
quantitative prompts have an inverse reading, so query the explicit phrasing,
include the interpreted direction in the answer, and use a magnitude sanity check
to confirm the result's size matches that direction.

When the same value has several official pods or source editions, prefer the pod
whose input interpretation exactly names the requested entity, date, and units,
and state that source/edition rather than mixing or averaging across editions.
Preserve any assumptions Wolfram used (units, date basis, data source); if the
task omits one, report the assumption shown.

For equation/ODE solutions, verify the returned expression by substituting the
given conditions back into the displayed formula. If alt text is hard to read,
derive the simplified form and re-check it before finalizing.

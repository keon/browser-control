# Model Hub

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. Model search rendered article-style cards with model IDs, task tags, updated dates, downloads, and likes. Model pages expose README/model card, tags/license, files/community tabs, spaces using the model, and downloads in visible text.

Use Hugging Face official pages/APIs. The public API is best for models/datasets;
use browser pages for Spaces, docs, pricing, blog, daily papers, and dataset
viewer UI tasks.

## Models and datasets API

```sh
browser-control http-get "https://huggingface.co/api/models?search=sentiment&sort=downloads&direction=-1&limit=10"
browser-control http-get "https://huggingface.co/api/models?pipeline_tag=translation&sort=likes&direction=-1&limit=10"
browser-control http-get "https://huggingface.co/api/datasets?search=text+retrieval&sort=downloads&direction=-1&limit=10"
```

Model detail (URL-encode `/` as `%2F` if needed):

```sh
browser-control http-get "https://huggingface.co/api/models/OWNER%2FREPO"
```

Read `modelId`, `author`, `lastModified`, `downloads`, `likes`, `pipeline_tag`,
`tags`, `cardData`, `siblings`, and `spaces` from output. Open the model page
when you need visible README usage, files, tensor type, or Inference widget.

## Browser pages

```sh
browser-control open "https://huggingface.co/models?search=QUERY"
browser-control inspect --query "Downloads"
```

Docs/search/blog/pricing: use `inspect --query` and official links. Dataset first
rows often have an official endpoint:

```sh
browser-control http-get "https://datasets-server.huggingface.co/first-rows?dataset=OWNER/NAME&config=default&split=train"
```

For Spaces, wait for the iframe/app to load, interact once, then extract visible
answer text. Do not need API tokens unless the page explicitly blocks public use.

## Browser-control friction fixes

Avoid local JSON parsing helpers. If you need to rank/filter public API results,
open Hugging Face and parse through page JS:

```sh
browser-control open "https://huggingface.co/"
browser-control eval "fetch('/api/models?search=QUERY&sort=downloads&direction=-1&limit=10').then(r=>r.json()).then(d=>JSON.stringify(d.map(m=>({id:m.modelId,downloads:m.downloads,likes:m.likes,modified:m.lastModified,tags:m.tags?.slice(0,8)}))))"
```

Then open the selected model page and use `inspect --query` or a small text slice
from `document.body.innerText` for README/model-card features. Do not spend many
turns on hidden Next.js data or network-log parsing unless the visible model card
is empty.

For conversational/chatbot tasks, search the official hub for conversational or
text-generation models (e.g. by `pipeline_tag` or a text query), rank by the
requested metric, and verify the chosen model on its official page before final
answering.

## Failure guardrails

For "most downloaded/liked/recent" tasks, rank with official `/api/models` or
`/api/datasets` using the requested task/license/tag and state the chosen id plus
metric. Do not substitute a famous model for "most recent". For parameter/default
questions, open model files/README or API `siblings` and quote the exact default.
For Spaces chat tasks, report the visible app response, not the model owner guess.

For model metadata constraints, distinguish `createdAt`, `lastModified`, commit
history, downloads, likes, task tag, and pipeline tag. A model that satisfies only
some constraints is not a match; continue filtering or state which constraint is
not visible.

When searching for models with multiple constraints, use the API filters and then
inspect several candidates rather than choosing the most famous model. For date
constraints, compare the exact metadata field the task names; `createdAt` cannot
stand in for `last updated`.

For metadata constraints that combine task type, update date, and download
threshold, do not only sort by current monthly downloads. Search multiple
official surfaces: API `pipeline_tag`, text search for the task phrase, model
cards/tags, and visible model-page download/update badges. A qualifying model may
rank lower by current monthly downloads while still satisfying the visible task
constraints.

If the prompt says "has 1M+ downloads", verify which download surface is visible
on the official page. Distinguish monthly API `downloads` from page-visible
aggregate or badge counts when the site exposes both, and state the surface used.

For Inference widget/API tasks, first try the on-page widget with real `fill` or
React-compatible input events, then inspect console/network and visible widget
state. If the official widget requires login, returns no result, or fails due to
site/API policy, report that blocker. If an official public API endpoint is
available through browser-control and returns the requested output, use that
official output and state the endpoint surface.

For deterministic inference tasks such as embeddings or sentence similarity, try
the model page widget, then the official Hugging Face Inference/Router endpoint
from the browser context if it is publicly callable. If both are blocked, flag an
API/login blocker; do not estimate the score from memory.

For generation tasks, check that the generated text includes every required
prompt element before finalizing. If the model output omits a required character,
object, topic, or format, try one more prompt or parameter adjustment through the
same official surface before answering.

For parameter defaults, do not use a random sample snippet unless it is clearly
the model's default. Check README, config files, widget/view-code settings, and
official file contents when visible.

For generation parameters, distinguish provider example defaults from the model
or widget defaults. Prefer `generation_config.json`, README default settings, or
the widget/View Code configuration over third-party serving snippets.

For deleted or errored Spaces, open the official Space page and related model
card if available. Report the live-space blocker separately from any inferred
model-card answer.

If a Space chat task asks an identity/provenance question and the live Space is
deleted, private, or in runtime error, follow official links or search the Space
name to the underlying model card. Distinguish the base model developer/trainer
from the fine-tuning or Space author, and include both when the model card makes
that split visible. Do not present a model-card inference as an observed chat
response; label it as the best official fallback when the Space cannot run.

For model discovery with multiple metadata constraints, keep filtering until a
single official model satisfies all named constraints. A model that is correct
for the task type but misses the requested update date, download threshold, or
language/domain is only a fallback and should not be the primary final answer.

When a model satisfies one metadata constraint but not another, say which field
failed using official metadata (`lastModified`, page update badge, API
downloads, page-visible downloads). Do not relax a date or download threshold
silently just because the model is famous or top-ranked.

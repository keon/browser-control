# Freshness Checks

Use this when a task contains relative or live words such as `today`, `yesterday`,
`latest`, `current`, `recent`, `last`, `past`, `next`, `upcoming`, `available`,
or asks about live prices, inventory, reviews, rankings, schedules, flights,
hotels, sports, news, repositories, papers, or search trends.

## Date anchor

- The task text is authoritative. If it gives an explicit date, use that date
  even when the run date differs.
- If the task uses a relative date with no explicit date, anchor it to the
  runner's evaluation date from the prompt.
- Convert `yesterday`, `last two days`, `past week`, and similar windows into
  concrete dates before searching.
- State dates in the final answer when recency is part of the task.

## Live answer rules

- Prefer official pages, official APIs, RSS feeds, structured data, or page text
  from the named site.
- Sort or filter by the requested freshness dimension when the site supports it:
  submitted date, published date, updated date, newest, latest, upcoming, or
  schedule date.
- Do not report an old result as latest/current unless the official page labels
  it as the latest visible result or the final answer clearly says no newer
  official item was visible.
- Do not use a future scheduled item as a completed result.
- For live inventory or availability, verify that the visible form/results still
  show the requested route, dates, location, guests, product variant, or filters.

## Cross-checks

- If a result's date is outside the requested window, keep searching or report no
  qualifying result. Do not silently widen the window.
- If a direct URL or search page drops query parameters, refill or restate the
  fields once and verify them again before trusting results.
- If a count comes from a sample page, visible cards, or first page only, do not
  present it as the total count.
- If a current official page contradicts an old remembered/golden value, answer
  from the current official evidence and include the observed date/source label.

## Final gate

Before `FINAL ANSWER:`, check:

1. requested date window converted to concrete dates;
2. returned item date/time is inside that window;
3. requested source/site is the source of the answer;
4. requested filters and sort are visible or otherwise officially evidenced;
5. final wording distinguishes current/live answer from historical/static answer.

# Evidence Page

Before finalizing an answer, leave the browser on the strongest official visible
evidence you can reasonably preserve.

## What to preserve

Prefer a page or state that shows the exact entity, filter, date, count, price,
rating, location, or action result used in the final answer.

Good final states include:

- A detail page for the selected item, article, recipe, course, repository, hotel,
  flight, or venue.
- A filtered result page that still visibly shows the requested query, dates,
  passengers, guests, location, sort order, or constraints.
- A confirmation/cart/checkout/search state that proves an action was completed
  or blocked.
- An official page matching the same item or fact when the answer was extracted
  through `eval`, `inspect`, or `http-get`.

## Avoid weak final states

Do not end on a generic home page, broad search page, unrelated tab, cookie
banner, blank page, stale route, or page whose visible state contradicts the
answer.

For single-item answers, open the item/detail page when possible. For comparison
or top-N tasks, keep the result list or multiple visible result cards when that is
stronger than a single detail page.

## When evidence is blocked

If login, payment, bot protection, regional availability, or site policy prevents
the exact final state, leave the browser on the official blocker page or the last
official page that proves the boundary. State that blocker in the final answer.

## Final check

Before `FINAL ANSWER:`, confirm the current tab title/URL and visible text still
support the answer. Use `page-info`, `inspect`, `text`, or `screenshot` when the
state is uncertain.

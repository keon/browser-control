# Final Answer Contract

Before emitting `FINAL ANSWER:`, do a quick contract check. If any item fails,
continue browsing/extracting instead of answering.

## Required checks

1. **Exact task object**: confirm the answer is about the requested site/page,
   category, filter, product type, date/season, location, and sort order.
   If the task names a subtype, filter, or surface, make that subtype explicit in
   the answer.
2. **All requested fields**: include every requested name/count/price/date/rating,
   review count, duration, source, author, list length, or action result.
3. **Observed evidence only**: do not invent product/model/course/page names or
   fill gaps from memory. Use values seen in official page text/API output.
4. **Dynamic dates**: apply `freshness.md`. Treat the task prompt date as
   authoritative. Convert relative windows into concrete dates and verify the
   returned item is inside that window before finalizing.
5. **Lists and counts**: for “top N”, “list N”, “how many”, filters, or sorted
   results, state the count/list and the applied filter/sort evidence.
6. **Action tasks**: say what was completed. If blocked by login/payment/site
   policy, state the blocker and the last official page evidence; do not merely
   explain how to do the action.
7. **Constraint completeness**: verify every hard constraint before finalizing.
   If the chosen item fails any required threshold, type, field, date, or action,
   continue searching or clearly state that no fully matching item was visible.
8. **No silent substitutions**: do not replace a requested product type, source,
   date window, page section, action, or metric with a nearby alternative unless
   the requested object is unavailable and the final answer says so.
9. **Blocker classification**: distinguish a factual answer from an action/API
   boundary. Reaching checkout, opening a sign-in wall, or seeing a disabled/no-
   result widget is not the same as completing the action.
10. **Reference surface**: when several official surfaces disagree, prefer the
   surface named by the task. Examples: a section page beats search snippets for
   section counts, a product Tech Specs page beats marketing/accessory copy for
   specifications, and a detail page beats result cards for thresholds.
11. **Temporal sanity**: for most recent/next/current tasks, compare the event or
   inventory date to today's run date and to the task's explicit date. Do not
   report a scheduled future event as completed, and do not use a current page
   for a historical season/date unless the prompt asks for live current data.
12. **Threshold evidence**: for "more than", "at least", "over", price bands,
   ratings, reviews, downloads, and dates, quote or include the observed value
   that satisfies the threshold. If the value only appears on a card, open the
   detail page or another official source when practical.
13. **No guessed official URLs**: if an official page URL was guessed, verify the
   page title/body confirms the entity. A successful navigation alone is not
   evidence that the entity exists.
14. **State retention**: for SPAs and booking/search flows, verify the final
   visible state still contains the requested route, date, guests/passengers,
   location, filters, and sort. If the page dropped parameters, do not answer
   from those results.
15. **Subtype lock**: when the task asks for a specific subtype or surface, the
   selected answer must come from that subtype/surface.

## Red flags: do not finalize yet

- Answer is only “results page loaded”, only a bare price/count for a detailed
  task, or omits the selected item name.
- Task asks for a specific subtype, filter, or date range, but the answer
  does not mention that subtype/filter.
- A value came from memory or inference rather than official visible text/API.
- Latest/current/today answer uses an old article/page without saying why it is
  the latest visible official result.
- Relative-date answer does not state or imply the concrete date window used.
- A direct search or booking URL dropped the requested date/location/person
  fields and you have not corrected the visible form state.
- The answer reports a count from only the first page/sample when the task asks
  for a total count.
- A review, article, recipe, course, model, or product is truncated and the task
  asks for the full content or exact value.
- A numerical answer has an inverse-direction or unit ambiguity that has not
  been checked against the task wording.
- The answer says "no match" after only one query/result page and no alternate
  official query, filter spelling, archive/search, or detail page was checked.
- The final answer names a fallback item as the primary answer even though it
  misses a hard constraint.
- The final answer names a model, product, course, repository, or page that was
  not visible in official page text, official API output, or a verified official
  search result.

## Compact final shape

Use one line:

```text
FINAL ANSWER: <direct answer with requested fields; include blocker only if truly blocked by official page>
```

# Paper Search

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. arXiv HTML search displayed `Showing 1–25 of 55,412 results for all: quantum computing`, advanced-search links, pagination, and static result pages. Use official Atom API for counts/metadata when possible; use browser pages for Help/Store/HTML article tasks.

Prefer arXiv official APIs/static pages through `browser-control http-get`; use
the browser for Help, Store, HTML article pages, and page navigation tasks.

## Search papers with Atom API

URL-encode the query yourself and keep `max_results` small.

```sh
browser-control http-get "https://export.arxiv.org/api/query?search_query=all:quantum+computing&max_results=5&sortBy=submittedDate&sortOrder=descending"
```

Useful query fields: `all:`, `ti:` title, `abs:` abstract, `au:` author,
`cat:cs.CL`, `cat:quant-ph`, `jr:` journal ref. Add date logic by reading
`published`/`updated` in each `<entry>` and the `<opensearch:totalResults>` count.
Known-paper fetch:

```sh
browser-control http-get "https://export.arxiv.org/api/query?id_list=1706.03762"
```

## Paper/page extraction

```sh
browser-control open "https://arxiv.org/abs/ARXIV_ID"
browser-control inspect --query "Abstract"
browser-control eval "JSON.stringify(({title:document.querySelector('h1.title')?.innerText,authors:[...document.querySelectorAll('.authors a')].map(a=>a.innerText),abstract:document.querySelector('.abstract')?.innerText,dates:document.querySelector('.dateline')?.innerText,subjects:document.querySelector('.subjects')?.innerText,links:[...document.querySelectorAll('a')].map(a=>a.innerText+' | '+a.href).filter(x=>/html|pdf|help|store/i.test(x)).slice(0,20)}))"
```

Use `/html/ARXIV_ID` when available for article text, figures, tables, equations,
and Introduction questions. For Help/category/about/store tasks, navigate the
official arXiv page and use `inspect --query` before clicking.

## Browser-control friction fixes

The Atom API occasionally returns `503` under load. Do not chain long sleeps.
Retry the same small API request once; if it still fails, switch to the official
HTML search/abs page or answer from the latest official evidence already visible.

For "latest" tasks, this is usually enough:

```sh
browser-control http-get "https://export.arxiv.org/api/query?search_query=all:%22neural+networks%22&max_results=10&sortBy=submittedDate&sortOrder=descending" | head -220
```

Avoid opening many arXiv advanced-search URLs when the Atom API can provide
`published`, `updated`, title, authors, categories, abstract, and total count.
If you must parse XML, keep output small and read the visible tags directly; do
not pipe into Python.

## Failure guardrails

For historical/count tasks, do not answer from a small `max_results` sample; read
`opensearch:totalResults`, page result counts, or enough pages to establish the
count. For article questions, open `/abs/ID` then `/html/ID` or the PDF text and
count only the requested object (figures/tables/formulas) using article content.
For Store/Leadership/Help tasks, inspect the specific official page and list all
visible requested entries before finalizing.

For title/abstract keyword tasks, match the wording precisely:

- `exact phrase in title` means phrase query against title.
- `words/term in title`, a quoted multi-word topic, or a quoted search string
  without the words "exact phrase" means use broad title-search semantics first.
  Cross-check phrase semantics, but do not choose the smaller phrase count unless
  the prompt explicitly asks for exact phrase containment.
- `originally announced/submitted between dates` means use the originally
  announced or submitted date field, not updated date.

For counts that are surprisingly small, cross-check the Atom API with arXiv's
HTML advanced search result count using the same field/date semantics. If the
counts disagree, inspect the query interpretation and prefer the surface whose
field/date wording matches the task.

For fixed historical counts, use an explicit date-range query when possible
(for example an Atom `submittedDate:[YYYYMMDD0000 TO YYYYMMDD2359]` range or the
matching HTML advanced-search date controls) and list the candidate IDs/titles
when the count is small. If official current arXiv surfaces disagree with an
older expected count, do not backfit the older count; answer with the official
count and enough IDs for verification.

For inclusive date windows, make the end date inclusive through the full day
(`YYYYMMDD2359` in API-style filters or the HTML advanced-search inclusive date
controls). Count unique arXiv entries, not visible snippets, and check whether
the prompt says originally announced, submitted, published, or updated.

When the prompt says a term is "in the article", use an all-fields/article
search unless it names title, abstract, author, or exact phrase. If a month/year
count is requested, filter by the originally announced/submitted date after the
field query and count unique entries.

For "most recent" tasks, sort descending by submitted/published date and inspect
the top relevant entries. Do not stop at the first older match from a broad
keyword search.

For author-name filters, search a broad relevant query sorted newest first, then
filter entries whose author first name exactly matches the prompt. Do not switch
to relevance or stop at an older title simply because it contains the topical
phrase.

For "related to" topical searches, try common spelling and punctuation variants
of the topic and any obvious arXiv subject/category terms, always sorted by
submitted date descending. Merge candidates, then apply the author/name filter.
Do not answer from a relevance-sorted or exact-phrase-only result if a newer
topic-equivalent result exists.

For Help/FAQ tasks, prefer the current official Help text. If current policy
contradicts older remembered behavior, answer the current policy and include the
observed page/source rather than inventing legacy instructions.

For submission-form or abstract-format questions, inspect the Help/FAQ text that
describes the exact field being asked about. If the task asks for a separator
between multiple abstracts, report the literal separator shown by arXiv's
instructions rather than answering from broader language-policy prose.

arXiv's legacy multi-abstract separator convention is five hyphens on their own
line: `-----`. If a question specifically asks for the separator between multiple
abstracts and the current page does not expose a newer literal separator, give
that delimiter with a note that broader multilingual-submission policy may have
changed.

For article/PDF formula counts, count only display equations/formulas in the main
article body when the prompt asks "how many formulas are in the article." Do not
include inline symbols, appendix equations, references, or duplicated MathJax
rendering unless the task explicitly asks for all equations.

If a paper has many numbered equations but the task asks for "formulas" in a
loose way, cross-check the PDF/HTML around each display equation and identify
which displays are actual formula definitions versus probability/objective
expressions. State the counting rule in the answer.

Prefer arXiv HTML/ar5iv-style equation blocks for formula counts when available;
PDF text extraction can duplicate or split math and overcount. If HTML is not
available, inspect nearby PDF text around each numbered equation and exclude
appendix-only, reference-only, or repeated rendering artifacts from the main
article count.

# Course Catalog

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. Search pages may show an AI Overview before All Results; use it only as navigation context. Opening a course page exposed rating, review count, level, duration, modules, assessments, outcomes, instructor, and provider in visible text.

Use Coursera pages for search/course details; use Coursera public APIs only for
official catalog metadata when it is clearly faster. No login is needed for most
browser tasks.

## Search pages

```sh
browser-control open "https://www.coursera.org/search?query=QUERY+WORDS"
browser-control inspect --query "Beginner"
```

Add visible filters with refs for level, duration, language, product type, sort,
or partner. Direct URL search plus one card extraction is usually enough.

```sh
browser-control eval "JSON.stringify([...document.querySelectorAll('a[href*=\"/learn/\"], a[href*=\"/specializations/\"], a[href*=\"/degrees/\"]')].slice(0,30).map(a=>{let c=a.closest('li,article,div')||a;return {title:a.innerText.trim(),url:a.href,text:c.innerText.slice(0,600)}}))"
```

## Course/specialization extraction

```sh
browser-control eval "JSON.stringify(({title:document.querySelector('h1')?.innerText,provider:[...document.querySelectorAll('a,span')].map(x=>x.innerText.trim()).find(t=>/University|Google|IBM|Meta|DeepLearning|Institute/i.test(t)),headings:[...document.querySelectorAll('h2,h3')].map(h=>h.innerText.trim()).filter(Boolean).slice(0,30),text:document.body.innerText.slice(0,6000)}))"
```

For catalog API lookup/listing, keep output small:

```sh
browser-control http-get "https://api.coursera.org/api/courses.v1?fields=name,slug,description,workload,partnerIds,primaryLanguages,domainTypes&limit=20&start=0"
```

Use visible course pages for ratings, modules, quizzes, outcomes, and paid degree
pages because API fields are often incomplete.

## Browser-control friction fixes

Coursera search filters can be slow and the `Free` label is often buried in app
state or hidden until cards scroll into view. Do not spend dozens of turns trying
different undocumented query parameters. Preferred flow:

1. search once with the user query;
2. if a visible Filter/Sort panel is needed, open it once and apply the obvious
   checkbox/ref;
3. open the best official course/specialization page;
4. answer from visible page text such as title, provider, level, rating,
   `Taught in English`, modules, and skills.

For language questions, course pages expose the answer more reliably than search
cards:

```sh
browser-control eval "(()=>{const t=document.body.innerText; return JSON.stringify({title:document.querySelector('h1')?.innerText, language:t.match(/Taught in [A-Za-z]+/)?.[0], snippets:t.match(/.{0,50}(Taught in|language|Free|Enroll).{0,50}/gi)?.slice(0,8)})})()"
```

If the exact "free" constraint is ambiguous, state the visible Coursera evidence
you used rather than exhausting the catalog.

## Failure guardrails

Scope matters: `Course`, `Specialization`, `Guided Project`, `Free`, level,
language, duration, and sort order are distinct. Include the product type and
provider in the final answer. For counts after filters, verify the visible result
count after filters are applied; do not count only visible cards. For ratings,
compute from the course review distribution visible on the official course page.

Do not substitute product types. If the task asks for a Guided Project, the final
answer must be a Guided Project or must state that no matching Guided Project was
visible after applying the Guided Project filter. A Course or Specialization can
be mentioned only as a fallback, not as the primary answer.

For Guided Project tasks, search both the visible Guided Project filter and
Coursera Project Network/catalog text. If the topic term is specialized, try a
closely related official term from the task domain before declaring no match, but
the final answer still needs the requested product type.

Guided Project search can fail when the page's AI/search layer does not use the
same vocabulary as the project title. Before declaring no matching Guided
Project, search the exact topic plus adjacent domain concepts, tools, and
provider terms, and include `/projects/` links from Coursera search results or
site search. Open plausible project pages and verify level, duration, provider,
and subject text from the project page.

Use Coursera's own suggestion chips and AI-overview terms as a domain thesaurus.
For science/technical topics, try foundational theories, methods, instruments,
and software/tool terms surfaced by the page, then filter back to `/projects/`
and the requested level on the project detail page.

When a discipline-level Guided Project query returns no exact cards, broaden
within the discipline before giving up: search core theories, landmark names,
standard tools/software, and data-analysis methods that the discipline uses.
Keep the product type strict (`/projects/` or visible `Guided Project`) and use
the project detail page to verify duration, provider, level, and subjects. A
closely related Guided Project is better than a Course/Specialization fallback
only when its official page actually covers a central subtopic of the requested
discipline.

For "newest" or sorted tasks, verify the sort control after applying the query
and filters. If the first visible result is unrelated to the query, continue to
the first result that satisfies the requested topic and state the visible sort
evidence.

If Coursera's newest sort produces unrelated global results, keep the sort
applied but choose the first visible result that satisfies the query/topic and
product type. Do not make an unrelated first card the primary answer.

For `Free` tasks, distinguish `Free`, `Free Trial`, `Financial aid available`,
and paid subscription labels. If only `Free Trial` is visible, do not report it
as a free course.

For free-tag language questions, scroll enough cards to see an actual `Free`
badge or open the course page and verify free enrollment/audit wording. A `Free
Trial` badge is a paid-subscription trial, not the same constraint.

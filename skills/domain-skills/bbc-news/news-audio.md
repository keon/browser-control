# News Audio

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. The BBC News front page rendered cleanly with `main`, many `data-testid` nodes, article links, timestamps, and no consent blocker from this US run. Latest headlines are volatile; answer from observed BBC section/article text.

Use BBC pages only (`bbc.com/news`, `bbc.com/sport`, `bbc.co.uk/search`). For
latest/current tasks, prefer section pages/RSS and include the observed date or
section context.
Apply `freshness.md` for latest, current, today, yesterday, recent, next, and
within-window questions; convert the requested window to concrete dates before
selecting an article, fixture, or list item.

## Search and sections

```sh
browser-control open "https://www.bbc.co.uk/search?q=QUERY+WORDS"
browser-control inspect --query "QUERY WORD"
```

Useful direct starts: `https://www.bbc.com/news`, `/news/world`, `/news/business`,
`/news/technology`, `/news/health`, `/news/science-environment`,
`https://www.bbc.com/sport`, and `https://www.bbc.com/weather`.
For top/latest headlines, RSS can give compact official titles:

```sh
browser-control http-get "https://feeds.bbci.co.uk/news/rss.xml" | head -80
```

## Article extraction

```sh
browser-control eval "JSON.stringify(({title:document.querySelector('h1')?.innerText,time:document.querySelector('time')?.innerText||document.querySelector('time')?.getAttribute('datetime'),author:[...document.querySelectorAll('[rel=author], [data-testid*=byline], span')].map(x=>x.innerText.trim()).find(t=>/^By /.test(t)),summary:[...document.querySelectorAll('main p, article p')].map(p=>p.innerText.trim()).filter(Boolean).slice(0,12),images:[...document.querySelectorAll('figure img,img')].map(i=>({alt:i.alt,caption:i.closest('figure')?.innerText})).filter(x=>x.alt||x.caption).slice(0,8)}))"
```

For sports tables/leaderboards/fixtures, open the relevant BBC Sport page, use
`inspect --query` for the league/tournament/player, then extract visible table
rows with `eval`. If a consent prompt appears, click the accept/continue button
once and continue.

For search-result pages, verify the article date on the opened article or RSS
entry. Do not use a search result merely because it matches keywords if the
published date is outside the requested live window.

## Failure guardrails

For `latest`, `today`, or `recent`, use BBC section pages/RSS/search sorted by
date and include the observed date; do not answer with an old article unless the
page explicitly presents it as latest. For Market Data/source, Sport leaderboards,
Culture reviews, and Audio lists, open that exact BBC section and quote the
section-specific label/source/list item.

For BBC Sport fixture tasks, completed-match pages may replace kickoff time with
`FT`. If the task asks when a match started, look for the fixture page, match
centre, scoreboard metadata, or archived fixture list before finalizing. If only
`FT` is visible, say the start time is not visible on the current official page.

When a sports question names a team and asks for the most recent match start,
check both the team fixtures/results page and the competition fixture page. A
completed match row with only `FT` is not enough if an older fixture listing or
match centre still exposes the kickoff time.

For navigation/section-count tasks, count named sections from the current BBC
navigation/menu or section index, not article headline occurrences. State what
surface was counted.

BBC may expose topical issue sections as navigation links, topic pages, or live
index modules rather than top-level menu labels. For "how many sections" tasks,
inspect the menu plus the News section index/search for section-like links before
answering zero.

For Audio/Podcast tasks, distinguish a current featured episode, a category
section, and a dated "best of" list. Do not substitute one for another without
stating that the requested section/list is no longer visible.

For BBC Audio requests with a named list label, category, or dated editorial
collection, search BBC Sounds/Audio and BBC site search for the exact label. If
the label exists lower on the page or behind a category tab, report from that
list rather than the generic hero item.

For dated BBC Audio/Podcast editorial lists, do not stop at the current Audio
landing page. Search BBC site search and BBC Sounds for the exact year plus list
wording, then open any official article/list page that matches the dated request.

Year-end podcast lists may live outside the current Audio landing page and may
be indexed as BBC Sounds collections, press/articles, or search-result snippets.
For "best podcasts of YEAR" style prompts, search combinations of `BBC Sounds`,
`BBC podcasts`, `best podcasts`, the year, and any visible candidate titles.
Prefer an official BBC list/collection; if only official search snippets expose
the dated list, report the snippet evidence and clearly separate it from current
Audio recommendations.

For a current "featured as <label>" request, look for the named label in hidden
page text, section headings, and category tabs before falling back to the hero
episode. Report the item under that exact label if it is present anywhere on the
official Audio/Sounds surface.

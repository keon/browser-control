# Sports Research

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. ESPN NBA standings rendered four visible tables plus header scoreboard links (`Gamecast`, `Box Score`) and league/date selects. Direct league URLs are reliable entry points; extract tables/cards with compact eval.

Use ESPN official pages. Sports pages are mostly server-rendered; direct league
URLs beat homepage wandering. For date tasks, convert dates to `YYYYMMDD` in URLs.
Apply `freshness.md` for latest/current/yesterday/last-window tasks before
choosing scoreboards, standings, transactions, or leader pages.

## Direct URLs

```sh
browser-control open "https://www.espn.com/nba/scoreboard/_/date/YYYYMMDD"
browser-control open "https://www.espn.com/nba/standings"
browser-control open "https://www.espn.com/nba/schedule/_/date/YYYYMMDD"
```

Swap league prefixes as needed: `nba`, `nfl`, `nhl`, `mlb`, `mens-college-basketball`,
`womens-college-basketball`, `soccer`. Useful pages: `/teams`, `/stats`,
`/team/roster/_/name/abbr`, `/team/depth/_/name/abbr`, `/team/injuries/_/name/abbr`,
`/search/_/q/QUERY`.

## Extract tables, scores, articles

```sh
browser-control inspect --query "TEAM OR LEAGUE"
browser-control eval "JSON.stringify(({title:document.querySelector('h1')?.innerText,headings:[...document.querySelectorAll('h1,h2,h3')].map(h=>h.innerText.trim()).filter(Boolean).slice(0,30),tables:[...document.querySelectorAll('table')].slice(0,5).map(t=>t.innerText.slice(0,1500)),cards:[...document.querySelectorAll('article,section')].map(x=>x.innerText.trim()).filter(Boolean).slice(0,12)}))"
```

For game/player details, open the box score or recap link from scoreboard and
extract visible leaders, score, date, and article summary. For ticket tasks,
follow ESPN's ticket link only after the game is identified and report the first
visible official ticket price.

For relative-date scoreboard tasks, compute the concrete date first and open
that date's scoreboard. For "most recent" completed game, walk backward from the
evaluation date until an ESPN page shows a final/completed game; do not stop on
an older playoff/finals page if later completed games exist.

## Failure guardrails

Separate historical-season tasks from current/live pages. If the prompt names a
season/date (for example 2023-24 or Dec 25, 2023), use ESPN URLs for that date or
season and do not mix in current standings. Search-count tasks must include all
matching ESPN teams and the requested league subset. ESPN+ Tools are not generic
ESPN+ streaming; look for Bracket Predictor/Analyzer/Dollar Value Generator text.

For rankings, standings, and power indexes, verify the selected season control or
URL. If ESPN redirects to the current season, inspect the page text or URL
parameters before trusting the table. The top and bottom rows must come from the
same selected season/table.

For power-index/ranking tables, extract the full table rows including rank,
team, metric, and season label. Do not infer the last-place team from record,
standings, or a truncated visible viewport; use the row with the worst rank on
the same ranking table.

ESPN ranking pages often split team names and metric columns into adjacent
tables, and the page source may also carry structured `stats` JSON. For BPI,
power index, playoff odds, or similar rankings, join rows by DOM order or parse
the embedded stats objects so the first and last teams come from the ranking
table itself. Do not use standings, win-loss record, or page-search snippets as
a substitute for the ranking order.

For recent/next-game tasks, verify the team schedule page and the current date
window. If no qualifying game exists, say no qualifying game is visible and give
the last/next official schedule evidence; do not invent a score or ticket price.

For transactions and player acquisitions, include the team, player, date, and
transaction type from the official ESPN transaction/news surface. A player name
without the team or date is incomplete for "most recent" tasks.

For team-name search/count tasks, include all ESPN-supported leagues surfaced by
the search/team pages and then count the requested subset separately. Do not rely
only on the first search page if pagination or league tabs are visible.

Team-name searches can hide same-city teams under league tabs, alternate
abbreviations, or current league pages. Check the ESPN search results, `/teams`
pages for major leagues, and obvious city/team aliases before counting.

ESPN often displays abbreviated city names such as `LA`, `NY`, or `KC` on cards
while the official team name expands the city. For city-name counts, open or
extract the team page/title/metadata for abbreviated candidates and count them
when the official full name contains the requested city.

For city team counts, ESPN search is not authoritative by itself. Check canonical
league team indexes such as `/nba/teams`, `/mlb/teams`, `/nfl/teams`,
`/nhl/teams`, `/wnba/teams`, and soccer/team indexes when relevant; collect the
official team names from those pages before counting the requested league subset.

Treat conventional ESPN market abbreviations as city matches when the task asks
for teams with a city in their name. Expand candidates such as `LA`, `NY`, `KC`,
`TB`, `SF`, and `WSH` by opening the team page or reading the team metadata; do
not exclude an abbreviated card merely because the visible label omits the full
city string.

For article lookup tasks, use ESPN search plus date/season terms and open the
best matching article. If the exact article is not found, report that exact
failure instead of answering with a different-year article.

For older ESPN articles, add distinctive title words, author/season terms, and
`site:espn.com` through Google Search if ESPN's own search is recency-biased.
Once an ESPN URL is found, open the official ESPN article and answer from it.

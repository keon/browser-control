# GitHub Research

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. GitHub repository search rendered public repo links, language/topic links, and star anchors without login. REST API remains the fastest official path for repo metadata, contributors, commits, releases, and search.

For public repo/search data, use GitHub official REST API through
`browser-control http-get`; use the browser for Pricing, Skills, product pages,
Trending UI, signup forms, or wiki pages.

## API shortcuts

```sh
browser-control http-get "https://api.github.com/search/repositories?q=QUERY+language:Python+stars:%3E500+pushed:%3E2024-01-01&sort=stars&order=desc&per_page=5"
browser-control http-get "https://api.github.com/repos/OWNER/REPO"
browser-control http-get "https://api.github.com/repos/OWNER/REPO/commits?per_page=3"
browser-control http-get "https://api.github.com/repos/OWNER/REPO/releases?per_page=3"
browser-control http-get "https://api.github.com/repos/OWNER/REPO/contributors?per_page=5"
```

Read fields directly from JSON output: `full_name`, `description`, `language`,
`stargazers_count`, `forks_count`, `created_at`, `pushed_at`, `html_url`.
For files changed in a commit, fetch the commit `url` from the list or:

```sh
browser-control http-get "https://api.github.com/repos/OWNER/REPO/commits/SHA"
```

## Browser pages

```sh
browser-control open "https://github.com/search?q=QUERY&type=repositories"
browser-control inspect --query "repositories"
```

Trending: `https://github.com/trending` or `/trending/python?since=weekly`.
Product/pricing/skills pages are official marketing pages; use `inspect --query`
and compact link extraction. Do not mutate repo state unless a task explicitly
asks; most GitHub browser tasks are read-only.

## Failure guardrails

Do not conflate GitHub search, most-starred repositories, and GitHub Trending.
Trending tasks require `github.com/trending...` page evidence; API search by stars
is not a substitute. For contributors/actions, open the exact repo/course page or
API endpoint and list the requested top contributors/actions exactly as shown.
For product FAQ answers, inspect the current official FAQ text before answering.

For created/updated/newest tasks, apply `freshness.md`: compute the concrete
date cutoff, put it in the GitHub search qualifier (`created:`, `pushed:`, or
`updated:` as requested), and verify the selected repository metadata from the
official API or page. Do not use an older cutoff from a prior run.

For top-contributor tasks, prefer the official contributors API or contributors
page sorted by contribution count. If bots, archived accounts, or hidden members
appear, state the surface used rather than substituting remembered names.

For GitHub Skills courses, distinguish learning objectives from concrete learner
actions. If the task asks what actions learners perform, inspect the course page,
README, or step list and report verbs such as create, resolve, commit, merge,
open, configure, or review exactly as shown.

On Skills pages, generic phrases like "learn the causes" or "understand tools"
are objectives, not learner actions. Prefer numbered steps, README task bullets,
or repository workflow instructions, and include each requested action as a verb
phrase.

If a Skills landing page only exposes objectives, open the linked course
repository/README or step files and extract the concrete learner actions from
headings, checklist items, or workflow steps. Use objective text only as context,
not as the answer to "what actions will learners perform."

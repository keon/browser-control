# Google Search tasks — compact search strategy

Use Google only as a way to find official sources. Prefer official pages and
search result snippets from authoritative sources; do not keep opening many
search engines or archives once official evidence is visible.

```sh
browser-control open "https://www.google.com/search?q=QUERY"
browser-control inspect --query "KEY PHRASE"
```

For product/platform support questions, use the search page to identify the
canonical vendor support page, then use `browser-control http-get` or `inspect`
on that official page. If the official page already exposes the exact answer in
a snippet or page text, answer directly and cite the observed source in your
reasoning.

Useful pattern:

```sh
browser-control http-get "OFFICIAL_URL" | grep -iE "important|keywords|from|task" | head -20
```

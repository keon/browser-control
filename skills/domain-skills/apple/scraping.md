# Apple official site — CLI extraction guide

Use official Apple pages only. Avoid opening many guessed URLs: extract candidate
links first, then navigate one likely official page.

## Newsroom/product release workflow

For questions about latest Apple product releases, start from Apple Newsroom or
Apple site search and extract candidate article links before choosing one:

```sh
browser-control open "https://www.apple.com/newsroom/"
browser-control inspect --query "PRODUCT KEYWORD"
browser-control eval "Array.from(document.querySelectorAll('a')).filter(a=>/PRODUCT KEYWORD/i.test(a.textContent+a.href)).slice(0,20).map(a=>a.textContent.trim()+' | '+a.href).join('\n')"
```

Apple site search is official and can reveal Newsroom/spec links:

```sh
browser-control open "https://www.apple.com/search/PRODUCT+KEYWORD?src=globalnav"
browser-control inspect --query "Newsroom"
```

After selecting an official article/spec page, use compact extraction for the
requested fields:

```sh
browser-control inspect --query "Pricing and Availability"
browser-control inspect --query "starting at"
browser-control inspect --query "storage"
```

Do not wander across support pages unless Newsroom/spec pages lack the requested
release/pricing/spec details.

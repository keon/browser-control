# Apple Pages

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. Apple search for `iPhone` rendered a large official result page with product, shop, compare, support, and current-lineup links. Current/latest product names are time-sensitive; extract from Apple search/product pages rather than hard-coding.

Use official Apple pages only: product pages, Store buy-flow, Support, Newsroom,
and search. Extract candidate links first; avoid guessing many URLs. If a guessed
Apple URL loads, verify the page title/body/navigation confirms the product or
article exists before using it as evidence.

## Find official pages

```sh
browser-control open "https://www.apple.com/search/QUERY+WORDS?src=globalnav"
browser-control inspect --query "QUERY WORD"
browser-control eval "JSON.stringify([...document.querySelectorAll('a')].filter(a=>/QUERY|support|buy|tech specs|newsroom/i.test(a.innerText+' '+a.href)).slice(0,20).map(a=>({text:a.innerText.trim(),href:a.href})))"
```

Direct pages often work: `/iphone/`, `/mac/`, `/ipad/`, `/watch/`, `/airpods/`,
`/apple-vision-pro/`, `/shop/buy-*`, `/support/`, `/newsroom/`.

## Product/spec extraction

```sh
browser-control inspect --query "price"
browser-control inspect --query "Tech Specs"
browser-control eval "JSON.stringify(({title:document.querySelector('h1')?.innerText,headings:[...document.querySelectorAll('h1,h2,h3')].map(h=>h.innerText.trim()).filter(Boolean).slice(0,30),text:document.body.innerText.slice(0,5000)}))"
```

For configurable Store prices, use visible Store controls and verify selected
chip/storage/color/connectivity before reading the displayed price. For Support
questions, prefer Support pages over marketing pages. For latest/release tasks,
use Newsroom, product-family overview pages, compare pages, or current product
page text and include observed dates/prices.

## Failure guardrails

Before final answer, verify Apple values on the exact official page/buy-flow. For
Store config questions, name the selected model/chip/memory/storage and quote the
visible final price; do not infer from base pricing. For "latest" products, use
current Apple product/search pages and never invent model names. For slogans,
trade-in, pickup, or support answers, use the matching Apple page text, not a
nearby marketing card.

For current product-family questions, first establish which current models Apple
lists for that family, then open the relevant Tech Specs or buy page. A product
name that appears only in a guessed URL and not in Apple search, navigation,
family, compare, or Newsroom text is not verified.

Apple marketing copy changes often. If the task asks for a slogan/tagline, prefer
the primary product hero heading/subheading on the exact product page and avoid
section labels such as lineup/navigation headings unless the page clearly uses
them as the product slogan.

If the exact hero copy is hidden behind animations or a campaign page, inspect
the product overview, compare page, and search result title snippets before
answering with a generic navigation label. Taglines usually sit near the product
name in a hero, not in `Explore`, `Lineup`, `Shop`, or support sections.

For tagline/slogan tasks, collect candidate copy from visible hero headings,
subheadings, `meta[name=description]`, Open Graph title/description, and any
hero/overview text near the product name. Reject generic navigation, catalog,
comparison, support, or call-to-action labels unless no marketing sentence is
available and the final answer explicitly says that only a section label was
visible.

For technical specs, use the official Tech Specs page and match the requested
field label exactly. If a spec has separate component weights, distinguish device
weight from battery, band, case, accessory, or shipping/package weight.

When a specification table lists both a bare device value and a configured or
accessory-inclusive value, answer the bare value for `device weight` unless the
task explicitly asks for a configuration including accessories.

Apple sometimes repeats weight values in comparison, buying, support, and tech
spec sections with different inclusions. For "device weight", search the exact
page text for labels such as `Device Weight`, `Weight`, `with band`, `with
accessory`, `battery`, and `varies by configuration`; do not promote an
accessory-inclusive value over a nearby bare device/headset value.

For Store pickup/scheduling, verify both availability and whether the requested
pickup date is selectable. If the date picker only offers a limited window, state
the selectable dates and do not claim the requested date was scheduled.

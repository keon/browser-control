# Retail Workflows

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. Search for `wireless mouse` rendered 16 `[data-component-type="s-search-result"]` cards. Titles and prices came from `h2 span` and `.a-price .a-offscreen`; review counts appeared in card text like `(42K)`, while broad `[aria-label*=rating]` can return the star label instead of count.

Use the real browser session; Amazon may block plain HTTP. Keep tab count at one.
Start with a direct search URL, then use compact card extraction.

## Search and sort

```sh
browser-control open "https://www.amazon.com/s?k=QUERY+WORDS"
browser-control inspect --query "RESULT KEYWORD"
```

Use UI refs for sort/filter when the task asks for a specific Amazon filter. For
price bands, adding words like `under 50`, `4 stars`, `size 7`, `waterproof` to
the query is often faster than opening many filters; verify on cards/details.

## Result card extraction

```sh
browser-control eval "JSON.stringify([...document.querySelectorAll('[data-component-type=\"s-search-result\"]')].slice(0,20).map(el=>({asin:el.getAttribute('data-asin'),title:el.querySelector('h2 span')?.innerText?.trim(),price:el.querySelector('.a-price .a-offscreen')?.innerText,rating:el.querySelector('[aria-label*=\"out of 5 stars\"]')?.getAttribute('aria-label'),reviews:(el.innerText.match(/\(([0-9.,Kk]+)\)/)||[])[1] || el.querySelector('[aria-label*=\"ratings\"]')?.getAttribute('aria-label'),sponsored:!!el.querySelector('.puis-sponsored-label-text'),url:el.querySelector('h2 a')?.href,text:el.innerText.slice(0,350)})))"
```

Filter sponsored/non-product rows unless the task permits them. Open the selected
product for color/size/returns/protection/add-to-cart details.

## Product page extraction

```sh
browser-control eval "JSON.stringify(({title:document.querySelector('#productTitle')?.innerText.trim(),price:document.querySelector('.a-price .a-offscreen')?.innerText,rating:document.querySelector('#acrPopover')?.getAttribute('title'),reviews:document.querySelector('#acrCustomerReviewText')?.innerText,availability:document.querySelector('#availability')?.innerText.trim(),bullets:[...document.querySelectorAll('#feature-bullets li')].map(x=>x.innerText.trim()).filter(Boolean).slice(0,12),returns:document.body.innerText.match(/Returns[\s\S]{0,500}/i)?.[0],delivery:document.body.innerText.match(/FREE delivery|delivery[\s\S]{0,250}/i)?.[0]}))"
```

For reviews, use the reviews link/section and extract the first visible review
with title, stars, date, and body. For add-to-cart tasks, click only after the
product options clearly match, then stop after cart confirmation.

For review-count thresholds, verify the count on the product page, not only the
search card. Normalize `K`/comma abbreviations and distinguish ratings from
written reviews. If the search card and product page disagree, prefer the
product page and state the observed live count.

Amazon often shows multiple count surfaces: search-card `K` abbreviations,
product-page ratings, written-review totals, and color/variation-specific review
counts. For thresholds such as `over 20,000 reviews`, inspect the review link,
product title/color, and any selected variation before declaring no match. If a
candidate is known by the task wording but a color variation lowers the count,
state the exact variation/count and continue checking other in-scope variations.

## Used-offer and cart stop rules

For "used" condition tasks, the all-offers page (`/gp/offer-listing/ASIN?...`)
often exposes rows without good semantic labels. Extract title plus offers, sort
locally, and answer from the first valid condition/price instead of exhaustively
checking every color/ASIN:

```sh
browser-control eval "JSON.stringify(({title:document.querySelector('#productTitle')?.innerText?.trim(),offers:[...document.querySelectorAll('div[id=\"aod-offer\"], #aod-pinned-offer')].map(o=>({condition:o.querySelector('#aod-offer-heading')?.innerText?.replace(/\s+/g,' ').trim(),price:(o.innerText.match(/\$[0-9,.]+/)||[])[0],text:o.innerText.slice(0,220)})).filter(x=>x.condition&&x.price)}))"
```

If the task asks for the cheapest matching used condition, collect enough offers
from the first relevant product/color page to identify a matching minimum; do not
keep opening more candidates after a valid cheaper matching offer is found unless
the task explicitly asks for global comparison across products.

For add-to-cart tasks:

1. verify the product title/options match;
2. click `#add-to-cart-button` or `input[name=submit.add-to-cart]`;
3. wait and verify with cart count or an "Added to cart" confirmation.

If two click/submit attempts do not change the cart or show confirmation, stop and
answer with the selected item plus the observed cart status. Do not loop on the
same button.

For save/list/cart tasks, final answer should say one of:

- completed, with visible cart/list confirmation or count;
- selected item found but blocked by sign-in/site policy, with the sign-in/cart
  evidence;
- no matching item found, with the failed hard constraint.

Do not mark an action complete merely because the target product page was opened.

For save/list tasks, an opened wishlist popover is not saved evidence unless it
shows the item added or a named list updated. For cart tasks, the cart count or
cart page must include the selected ASIN/title; if an endpoint redirects to sign
in or the cart stays empty, report the blocker rather than claiming completion.

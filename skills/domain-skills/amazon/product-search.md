# Product Search

Use Amazon pages through `browser-control`; Amazon often blocks plain external
HTTP and may behave differently for anonymous sessions. Start with a direct
search URL, extract cards compactly, then open the selected product page to
verify title, variation, price, rating, and counts.

## Search

```sh
browser-control open "https://www.amazon.com/s?k=QUERY+WORDS"
browser-control inspect --query "RESULT KEYWORD"
browser-control eval "JSON.stringify([...document.querySelectorAll('[data-component-type=\"s-search-result\"]')].slice(0,30).map(el=>({asin:el.getAttribute('data-asin'),title:el.querySelector('h2 span')?.innerText?.trim(),price:el.querySelector('.a-price .a-offscreen')?.innerText,rating:el.querySelector('[aria-label*=\"out of 5 stars\"]')?.getAttribute('aria-label'),reviews:(el.innerText.match(/\\(([0-9.,Kk]+)\\)/)||[])[1] || el.querySelector('[aria-label*=\"ratings\"]')?.getAttribute('aria-label'),sponsored:!!el.querySelector('.puis-sponsored-label-text'),url:el.querySelector('h2 a')?.href,text:el.innerText.slice(0,400)})).filter(x=>x.asin&&x.title))"
```

Filter sponsored/non-product rows unless the task allows them. For price bands,
ratings, sizes, colors, and feature keywords, Amazon query words are often faster
than UI filters, but the product page must verify the final constraints.

## Product Detail

```sh
browser-control open "https://www.amazon.com/dp/ASIN"
browser-control inspect --query "Add to Cart"
browser-control eval "JSON.stringify(({title:document.querySelector('#productTitle')?.innerText?.trim(),price:document.querySelector('.a-price .a-offscreen')?.innerText,rating:document.querySelector('#acrPopover')?.getAttribute('title'),reviews:document.querySelector('#acrCustomerReviewText')?.innerText,availability:document.querySelector('#availability')?.innerText.trim(),selected:[...document.querySelectorAll('[aria-checked=\"true\"], .selection')].map(x=>x.innerText.trim()).filter(Boolean).slice(0,12),bullets:[...document.querySelectorAll('#feature-bullets li')].map(x=>x.innerText.trim()).filter(Boolean).slice(0,12),cart:!!document.querySelector('#add-to-cart-button, input[name=\"submit.add-to-cart\"]'),list:!!document.querySelector('#add-to-wishlist-button-submit, #wishListMainButton, input[name=\"submit.add-to-registry.wishlist\"]')}))"
```

Open the detail page before finalizing any threshold. Search cards can show
abbreviated or variation-specific prices/counts, while product pages may show
rating count, written reviews, and selected variation separately.

## Constraints

Hard constraints include product type, brand/model, color, size, condition,
price band, rating, review/rating count, delivery/availability, and variant. If
the selected variation fails a threshold, keep searching or state the mismatch.

For review-count thresholds, normalize `K` and comma abbreviations, and
distinguish ratings from written reviews. If search card and product page counts
disagree, prefer the product page and say which count surface was used.

Before concluding that no product meets a high rating/review threshold, broaden
the search at least once without price or variation filters, using the core
product type plus the hard feature/color words. Open high-review candidates and
verify their current selected variation price/color on the product page; Amazon
search cards can hide a qualifying variation behind a different card price.

For threshold tasks, do not let one filtered result page prove absence. Cross
check with a direct keyword query, a product-type synonym query, and the top
high-review candidates visible in the category. Only answer "no match" after the
product detail pages for plausible high-review candidates fail a hard constraint.

When a plausible in-band candidate appears with a lower search-card count, still
open its detail page before excluding it. Search cards can show child-variation
counts while the product page exposes a parent listing count or a different
selected color/offer count.

## Actions

For cart/list/save tasks:

1. verify the exact product and selected options;
2. click the appropriate Add to Cart/Add to List control once or twice;
3. verify with an "Added" confirmation, cart count/cart page, or visible list
   update before saying the action completed.

An opened wishlist popover, account flyout, or clicked button is not completion.
If Amazon redirects to sign-in, shows no list, or the cart stays empty, answer
with the selected item plus the observed login/site-policy blocker. Do not claim
the item was saved or added unless the visible confirmation or count proves it.

## Blockers

If a CAPTCHA, sign-in wall, unavailable variation, or silently rejected cart/list
action appears, stop after documenting the official page evidence. Do not keep
retrying the same action path or bypass authentication.

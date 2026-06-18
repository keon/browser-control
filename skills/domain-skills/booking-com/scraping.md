# Booking.com — CLI extraction guide

Use the real browser; Booking often WAF-blocks plain HTTP. Keep output compact.

## Search URL workflow

For hotel search pages, navigate the current tab directly instead of filling the
homepage if you know destination and dates. Build the URL from task parameters:

```sh
browser-control open "https://www.booking.com/searchresults.html?ss=DESTINATION&checkin=YYYY-MM-DD&checkout=YYYY-MM-DD&group_adults=N&group_children=0&no_rooms=1&selected_currency=USD"
browser-control wait load 10
browser-control inspect --query "per night"
```

Common params: `ss`, `checkin`, `checkout`, `group_adults`, `group_children`,
`no_rooms`, and `selected_currency=USD`. Use Booking filter params only when the
task explicitly asks for filtering and you can infer the filter from the UI or
current URL. Booking may strip dates/destination when only a text destination is
supplied, so verify with `page-info` and visible search fields.

## Date picker fallback

If Booking drops date params, use the visible date picker with DOM dates:

```sh
browser-control eval "document.querySelector('[data-date=\"YYYY-MM-DD\"]')?.click(); 'date clicked'"
```

Advance months with the visible next button:

```sh
browser-control eval "document.querySelector('[aria-label=\"Next month\"]')?.click(); 'next'"
```

## Extract result cards

Prefer one compact extraction over repeated screenshots/scrolling:

```sh
browser-control eval '
JSON.stringify([...document.querySelectorAll("[data-testid=property-card], div:has([data-testid=title])")].slice(0,12).map(card=>({
  name: card.querySelector("[data-testid=title], h3, h2")?.innerText?.trim(),
  price: card.querySelector("[data-testid=price-and-discounted-price]")?.innerText?.trim() || (card.innerText.match(/\$[0-9,.]+[^\n]*(?:night|total)?/i)||[])[0],
  score: (card.innerText.match(/Scored\s+[0-9.]+|[0-9.]+\s+(?:Wonderful|Excellent|Very Good|Good)/i)||[])[0],
  area: card.innerText.split("\n").find(x=>/downtown|center|subway|district|arr\.|area/i.test(x)),
  text: card.innerText.slice(0,500)
})))'
```

If direct URL and form submission both strip the search params, do not keep
looping. Use the current official city/search page as fallback, extract the best
matching options, and be explicit about what Booking exposed.

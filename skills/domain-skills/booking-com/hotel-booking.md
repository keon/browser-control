# Hotel Booking

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. A direct search URL loaded Booking's search shell but dropped the requested Paris/future params to visible defaults (`Tue, May 26 — Wed, May 27`) and no property cards. Always verify destination/date fields before trusting results; if params drop, fill the visible form once.

Use the real browser; Booking often WAF-blocks plain HTTP. Prefer direct search
URLs, then one compact result-card extraction. Verify that destination/dates did
not get dropped.

## Search URL

```sh
browser-control open "https://www.booking.com/searchresults.html?ss=DESTINATION&checkin=YYYY-MM-DD&checkout=YYYY-MM-DD&group_adults=2&group_children=0&no_rooms=1&selected_currency=USD"
browser-control wait load 10
browser-control inspect --query "per night"
```

Common params: `ss`, `checkin`, `checkout`, `group_adults`, `group_children`,
`no_rooms`, `selected_currency=USD`. If Booking strips params, use visible fields
or date cells (`[data-date=YYYY-MM-DD]`) once; do not loop on the same failure.

## Result cards

```sh
browser-control eval "JSON.stringify([...document.querySelectorAll('[data-testid=property-card], div:has([data-testid=title])')].slice(0,15).map(card=>({name:card.querySelector('[data-testid=title],h3,h2')?.innerText?.trim(),price:card.querySelector('[data-testid=price-and-discounted-price]')?.innerText?.trim()||(card.innerText.match(/\$[0-9,.]+[^\n]*/)||[])[0],score:(card.innerText.match(/Scored\s+[0-9.]+|[0-9.]+\s+(Wonderful|Excellent|Very Good|Good)/i)||[])[0],area:card.innerText.split('\n').find(x=>/center|downtown|district|near|km|miles/i.test(x)),text:card.innerText.slice(0,700)})))"
```

Use filters/sort only when the task requires them. For “book/reserve”, stop at
selection/availability summary unless credentials/payment are required.

For booking/reservation tasks, a practical completion boundary is reaching the
official checkout/details page with a selected property, dates, occupancy, room,
and required filters verified. Do not enter personal or payment details. Final
answer should say "selected and advanced to checkout/details" unless the site
shows a confirmed reservation.

If checkout requires account sign-in, guest identity, or payment details, treat
that as an external credential/payment blocker. Capture the selected property,
room, total price, dates, guests, and confirmed filters before flagging the
blocker. Do not keep trying alternate hotels merely to bypass required personal
or payment fields.

Before choosing a property, verify every hard filter on the result card or hotel
page: destination/area, dates, guests/children, free cancellation, breakfast,
airport shuttle, pool, WiFi, fitness/spa, rating threshold, and price/currency.
If the site changes availability and a different property satisfies all filters,
that is acceptable; state the live property selected and the verified filters.

If a state/region search drops exact dates, guests, rooms, or prices, do not use
generic destination cards as the final answer. Refill the visible form once or
open a city/property result and verify the requested dates/occupancy on the
hotel page before answering. If Booking still strips the parameters, report the
state loss as a site limitation rather than presenting unpriced generic hotels
as exact-date matches.

For book/reserve tasks, the strongest non-credential completion is reaching the
secure details/checkout page with the selected property, room, dates, occupancy,
total price, and required amenities/filters visible. Do not enter personal or
payment details. If confirmation beyond that point is required, classify it
as a credential/payment boundary, not a search failure.

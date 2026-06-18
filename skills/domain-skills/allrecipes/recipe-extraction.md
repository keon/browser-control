# Recipe Extraction

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/` plus a recipe detail probe. Search results for `vegetarian lasagna` rendered plain recipe links like `Easy Vegetarian Spinach Lasagna 117 Ratings`; recipe detail exposed one JSON-LD `Recipe` with `aggregateRating`, ISO durations, `recipeIngredient`, instructions, and `nutrition`.

Use official Allrecipes pages. Most tasks are recipe search + recipe-page extraction.
Prefer search URL, one `inspect`, then one JSON `eval` on the chosen recipe.

## Search

```sh
browser-control open "https://www.allrecipes.com/search?q=QUERY+WORDS"
browser-control inspect --query "rating"
```

Search result pages change layout. Extract cards by link + nearby text instead of
clicking filters repeatedly:

```sh
browser-control eval "JSON.stringify([...document.querySelectorAll('a[href*=/recipe/]')].slice(0,30).map(a=>{let c=a.closest('article,li,div')||a;return {title:(a.innerText||a.getAttribute('aria-label')||'').trim(),url:a.href,text:c.innerText.slice(0,500)}}))"
```

## Recipe page extraction

Recipe pages usually expose schema.org Recipe JSON-LD. Use it first; it is more
stable than visual selectors.

```sh
browser-control eval "JSON.stringify([...document.querySelectorAll('script[type=\"application/ld+json\"]')].map(s=>{try{return JSON.parse(s.textContent)}catch(e){return null}}).flatMap(x=>Array.isArray(x)?x:[x]).flatMap(x=>x&&x['@graph']?x['@graph']:[x]).filter(x=>x&&/Recipe/.test(String(x['@type']))).map(r=>({name:r.name,yield:r.recipeYield,prep:r.prepTime,cook:r.cookTime,total:r.totalTime,rating:r.aggregateRating,ingredients:r.recipeIngredient,instructions:(r.recipeInstructions||[]).map(i=>i.text||i.name||i).slice(0,20),nutrition:r.nutrition})))"
```

If JSON-LD is sparse, use visible text:

```sh
browser-control inspect --query "Ingredients"
browser-control inspect --query "Nutrition"
browser-control inspect --query "Reviews"
```

For “latest review”, click or query Reviews once, then extract visible review
blocks and dates. For collection/category tasks, use Allrecipes navigation/search
and extract section links/text from the current official page only.

## Failure guardrails

Hard constraints matter. Verify every requested ingredient, diet label, rating
threshold, review/rating count, prep/cook/total time, and recipe type on the
recipe page before finalizing. Do not answer with the closest recipe if it misses
a threshold unless you explicitly say no fully matching recipe was visible and
name the missing threshold.

Ingredient wording is literal unless the task says "like" or "similar". If the
prompt names an ingredient family such as leaves/greens, olives, nuts, dairy-free,
or gluten-free, verify the ingredient list contains that family or an obvious
member of it. Do not infer it from the recipe title or cuisine style.

Allrecipes may distinguish ratings, written reviews, and comments. If the prompt
says reviews, prefer visible review count text; if only JSON-LD exposes
`ratingCount` or `reviewCount`, report which field you used.

Collection/category pages can mix individual recipe cards with article/listicle
cards. If a task asks for recipes, return individual recipe titles/links, not
roundup article titles, unless the page itself labels the roundup as the
requested item.

When search results are sparse, try one broader official query before declaring
no match: remove diet words that may be encoded as tags, try singular/plural
ingredient names, and open likely recipe detail pages to verify ingredients and
counts from JSON-LD. Do not treat search-result snippets as complete ingredient
or review evidence.

For section recommendation tasks, inspect the current section page for cards that
link directly to `/recipe/` pages. If the top visible items are galleries or
roundups, open them only to extract the individual recipe titles they recommend;
do not list the roundup article titles as recipes.

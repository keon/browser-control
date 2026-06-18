# Dictionary Grammar

## Observed context (2026-05-26)

Collected with browser-control in `.browser-control/site-context/20260526T101310Z/`. Word pages expose visible definition text plus stable-ish classes: `.entry-body__el`, `.hw`, `.pos`, `.uk .ipa`, `.us .ipa`, `.def`, `.eg`, and `.trans`. The page also has pronunciation buttons and translation sections.

Use official Cambridge Dictionary pages. Direct URLs are faster than homepage
search for words, thesaurus, translations, and grammar.

## Direct starts

```sh
browser-control open "https://dictionary.cambridge.org/dictionary/english/WORD"
browser-control inspect --query "Definition"
```

Patterns: `/dictionary/english/WORD`, `/thesaurus/WORD`,
`/dictionary/english-chinese-simplified/WORD`, `/dictionary/english-french/WORD`,
`/dictionary/english-spanish/WORD`, and site search with
`https://dictionary.cambridge.org/search/english/direct/?q=QUERY`.

## Dictionary extraction

```sh
browser-control eval "JSON.stringify([...document.querySelectorAll('.entry-body__el, .pr.entry-body__el')].slice(0,6).map(e=>({headword:e.querySelector('.hw')?.innerText,partOfSpeech:e.querySelector('.pos')?.innerText,uk:e.querySelector('.uk .ipa')?.innerText,us:e.querySelector('.us .ipa')?.innerText,defs:[...e.querySelectorAll('.def')].map(x=>x.innerText.trim()).slice(0,8),examples:[...e.querySelectorAll('.eg')].map(x=>x.innerText.trim()).slice(0,8),translations:[...e.querySelectorAll('.trans')].map(x=>x.innerText.trim()).slice(0,8)})))"
```

For grammar pages, use search then open the Cambridge grammar result and extract
headings/examples. For Plus quizzes/games/shop/language-switch tasks, use normal
browser interaction and answer only from visible Cambridge pages.

## Grammar-page shortcut

Many grammar tasks have direct URLs. Try the direct slug first and use
`inspect --query` before static parsing:

```sh
browser-control open "https://dictionary.cambridge.org/grammar/british-grammar/fewer-or-less"
browser-control inspect --query "fewer" | head -150
```

If browser rendering stalls, use `browser-control http-get` plus bounded shell
filters (`grep`/`sed`/`head`). Do not read Claude tool-result cache files or call
unavailable tools such as `Read`; only Bash/browser-control is enabled in eval.

For comparison grammar entries, the answer usually needs:
- the rule in one sentence;
- one Cambridge example for each side of the contrast;
- the article title/URL if visible.

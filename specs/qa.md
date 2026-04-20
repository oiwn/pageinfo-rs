# QA: Manual Test URLs

Curated set of URLs for hand-checking `pginf` across diverse source types.

Run through the typical workflow for each URL:

```
pginf fetch <url>
pginf links <url>
pginf meta <url>
pginf json <url>
pginf text <url>
```

## News (US/RU/ES)

```
pginf fetch "https://www.interfax.ru/business/1048231"
pginf fetch "https://www.foxnews.com/health/gut-microbes-could-key-fighting-toxic-long-lasting-forever-chemicals-research-says"
pginf fetch "https://www.elmundo.es/como/2026/04/17/69e23b1ae85ece50198b4578.html"
```

## Crypto news

```
pginf fetch "https://decrypt.co/302057/sec-can-do-better-crypto-task-force"
pginf fetch "https://www.coindesk.com/coindesk-indices/2026/04/15/crypto-long-and-short-fighting-fraud-in-the-digital-age-why-state-led-identity-is-the-future"
```

## Blog / DIY

```
pginf fetch "https://caseelegance.com/blogs/humidor-resources/building-a-humidor-diy"
pginf fetch "https://www.azwoodman.com/Cigar-Humidor-Plan.html"
```

## Government (AU)

```
pginf fetch "https://www.accc.gov.au/media-release/accc-investigating-retailers-making-concerning-black-friday-claims"
```

## Personal site / minimal

```
pginf fetch "https://patrickrhone.com"
```

## Scraper-resistant (needs browser emulation)

```
pginf fetch "https://www.reddit.com/r/rust/comments/1sp4mib/a_purerust-htmlcssmarkdown_to_pdf_converter_way/" --browser chrome137
```
pginf analyze -u "https://www.interfax.ru/business/1048231"
pginf analyze -u "https://www.foxnews.com/health/gut-microbes-could-key-fighting-toxic-long-lasting-forever-chemicals-research-says"
pginf analyze -u "https://www.elmundo.es/como/2026/04/17/69e23b1ae85ece50198b4578.html"
```

## Crypto news

```
pginf analyze -u "https://decrypt.co/302057/sec-can-do-better-crypto-task-force"
pginf analyze -u "https://www.coindesk.com/coindesk-indices/2026/04/15/crypto-long-and-short-fighting-fraud-in-the-digital-age-why-state-led-identity-is-the-future"
```

## Blog / DIY

```
pginf analyze -u "https://caseelegance.com/blogs/humidor-resources/building-a-humidor-diy"
pginf analyze -u "https://www.azwoodman.com/Cigar-Humidor-Plan.html"
```

## Government (AU)

```
pginf analyze -u "https://www.accc.gov.au/media-release/accc-investigating-retailers-making-concerning-black-friday-claims"
```

## Personal site / minimal

```
pginf analyze -u "https://patrickrhone.com"
```

## Scraper-resistant (needs browser emulation)

```
pginf analyze -u "https://www.reddit.com/r/rust/comments/1sp4mib/a_purerust_htmlcssmarkdown_to_pdf_converter_way/" --browser chrome137
```

---
name: pginf
description: Use pginf to analyze a web page's structure, URL patterns, and metadata. Invoke when asked to inspect a site, understand its URL structure, or gather evidence for building a crawler config.
argument-hint: <url>
allowed-tools: Bash
installed-by: pginf
---

`pginf` is a CLI tool for HTTP-based web page analysis. It extracts URL
structure, metadata, and embedded data from a page without a browser.

## Commands

```
pginf fetch <url>                           # fetch + cache, print HTTP metadata
pginf fetch <url> --json                    # same, JSON output
pginf fetch <url> --refresh                 # bypass cache, re-fetch
pginf links <url>                           # processed links + URL summaries
pginf links <url> --filter internal         # internal links only
pginf links <url> --filter external         # external links only
pginf links <url> --format json
pginf links <url> --format toon
pginf meta <url>                            # curated metadata (title, lang, meta tags)
pginf meta <url> --format json
pginf meta <url> --format toon
pginf json <url>                            # structured data (JSON-LD, Next.js, inline)
pginf json <url> --json
pginf text <url>                            # extracted text content (plain text)
pginf text <url> --format json
pginf text <url> --format toon
pginf html -u <url>                         # full HTML
pginf html -u <url> -s "div.article"        # elements matching CSS selector
pginf http -u <url>                         # raw request/response debug
pginf install skills local                  # install skill to <project>/.agents/skills/pginf/
pginf install skills global                 # install skill to ~/.agents/skills/pginf/
pginf help tool                             # built-in guide
```

## Global flags

Apply to any command that fetches:

```
pginf --proxy <url>              # proxy with auth: socks5://user:pass@host:port
pginf --browser <name>           # TLS fingerprint: chrome137, firefox, safari, edge, okhttp
pginf --timeout <seconds>        # request timeout (default: 30)
```

Use `--proxy` when direct access is blocked or when you need residential
proxies. Supports SOCKS5, HTTP, and HTTPS proxies with inline auth.

Use `--browser` when the site blocks default HTTP clients. The tool
automatically retries with different browsers on 403/429/503 errors.
Non-standard block codes (e.g. 462, 465) are NOT retried automatically —
always pass `--browser chrome137` explicitly when a WAF-protected site
returns an unusual 4xx status or an "Access Denied" page.

Available browser names: `chrome137`, `chrome136`, ..., `chrome100`, `firefox`,
`safari`, `edge`, `okhttp`.

## Output

Commands default to text output. `links`, `meta`, and `text` use
`--format text|json|toon`. `fetch` and `json` still use `--json`.

## Caching

Pages are cached in `.pginf/`. All commands auto-fetch and cache if the page
is not already cached.

- `--refresh`: refetch and overwrite cache entry
- `--no-cache`: skip cache read/write entirely

## Typical workflow

1. `pginf fetch <url>` — load the page into cache, inspect HTTP metadata
2. `pginf links <url> --format toon` — inspect processed links and URL summaries
3. `pginf meta <url> --format toon` — inspect curated metadata
4. `pginf json <url>` — check for structured data
5. `pginf text <url> --format toon` — extract page content

## Library usage

`PageClient` is the core HTTP client, usable from Rust:

```rust
use pageinfo_rs::{PageClient, Emulation, FetchResult};

let client = PageClient::builder()
    .browser(Emulation::Chrome137)
    .build();

let result: FetchResult = client.fetch("https://example.com").await?;
// result.input_url, result.final_url, result.status, result.headers, result.body, result.duration_ms
```

### Link extraction

```rust
use pageinfo_rs::{extract_links, Link};
use pageinfo_rs::dom_content_extraction::scraper::Html;
use url::Url;

let doc = Html::parse_document(&html_body);
let base = Url::parse("https://example.com")?;

// All links, normalized (lowercase host, no fragment)
let links: Vec<Link> = extract_links(&doc, &base);

// Internal links can be selected from processed links
let internal: Vec<&Link> = links.iter().filter(|link| link.is_internal).collect();

// Per-link normalization
let mut link = links[0].clone();
link.normalize();
link.strip_tracking();
link.is_asset();            // true for .css, .js, .png, etc.
link.is_same_host(&base);   // exact host comparison
```

Also available: `extract_raw_links`, `extract_registered_domain`, `RawLink`,
`LinkOptions`, `UrlFacts`, `DateKind`.

Automatic browser fallback (Chrome136 → Firefox139 → Safari18.5) only
triggers on 403/429/503. For WAFs using custom codes, set `.browser()`
explicitly.

## Key caveats

- Locale-prefixed paths (e.g. `/es/`, `/fr/`) appear in utility URLs and
  should be blacklisted in crawler configs
- `sponsored-content`, `press-release`, `author`, `newsletters`, `videos`,
  `podcasts`, `price` are typically non-editorial sections
- The tool uses HTTP only -- JS-rendered content may be incomplete
- Browser emulation changes TLS fingerprint and headers, but does not execute
  JavaScript
- WAFs using non-standard block codes (465, 462, etc.) are not retried
  automatically; pass `--browser chrome137` or set `.browser()` in library
  usage to avoid silent failures

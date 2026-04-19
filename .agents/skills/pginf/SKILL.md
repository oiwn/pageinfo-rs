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
pginf analyze -u <url>           # full report
pginf analyze -u <url> links     # URL groups and path depth
pginf analyze -u <url> meta      # curated metadata only
pginf analyze -u <url> json      # embedded structured/JSON data
pginf analyze -u <url> --refresh # bypass cache, re-fetch
pginf http -u <url>              # raw request/response debug
pginf help tool                  # built-in guide
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

## Output sections

**URL Groups** -- internal links grouped by first path segment, with link count
and sample URLs. Use this to understand site sections and identify article vs
non-article areas.

**Path Depth** -- distribution of internal URL depths. Depth-5 paths with date
segments (year/month/day) are typically articles.

**Utility URLs** -- detected non-content URLs (privacy, terms, sitemaps, feeds,
locale variants).

**Structured Data** -- detected JSON-LD, Next.js data, and inline JSON payloads.

**Curated Metadata** -- filtered high-signal meta tags (description, robots,
og:type, article:section).

**Detected article pattern** -- heuristic guess at the article URL pattern.
Often incomplete (may show only one section). Derive the full pattern from the
URL groups table instead.

## Caching

Results are cached in `.pageinfo/`. Use `--refresh` if the page may have
changed.

## Library usage

`PageClient` is the core HTTP client, usable from Rust:

```rust
use pageinfo_rs::{PageClient, Emulation};

let client = PageClient::builder()
    .browser(Emulation::Chrome137)
    .build();

let cached_page = client.fetch("https://example.com").await?;
// cached_page.html contains the raw HTML
// cached_page.fetch.status is the HTTP status code
```

`pageinfo_rs` re-exports `Emulation` and the `wreq_util` / `wreq` crates,
so no extra direct dependencies are needed.

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

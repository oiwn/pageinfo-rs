---
name: pginf
description: Use pginf to analyze a web page's structure, URL patterns, and metadata. Invoke when asked to inspect a site, understand its URL structure, or gather evidence for building a crawler config.
argument-hint: <url>
allowed-tools: Bash
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

Use `--browser` when the site blocks default HTTP clients (403/429). The tool
automatically retries with different browsers on these errors.

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

## Key caveats

- Locale-prefixed paths (e.g. `/es/`, `/fr/`) appear in utility URLs and
  should be blacklisted in crawler configs
- `sponsored-content`, `press-release`, `author`, `newsletters`, `videos`,
  `podcasts`, `price` are typically non-editorial sections
- The tool uses HTTP only -- JS-rendered content may be incomplete
- Browser emulation changes TLS fingerprint and headers, but does not execute
  JavaScript

# pageinfo-rs

[![CI](https://github.com/oiwn/pageinfo-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/oiwn/pageinfo-rs/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/oiwn/pageinfo-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/oiwn/pageinfo-rs)
[![crates.io](https://img.shields.io/crates/v/pageinfo-rs.svg)](https://crates.io/crates/pageinfo-rs)
[![License: GPL-3.0](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](https://github.com/oiwn/pageinfo-rs/blob/main/LICENSE)
![Rust 1.85+](https://img.shields.io/badge/rust-1.85%2B-orange.svg)

CLI tool and library for researching web pages. Built to help LLMs inspect sites and build crawlers.

HTTP-only. No browser automation. Uses `wreq` with TLS fingerprinting via `wreq-util` for browser emulation.

## What It Does

Fetches a page and exposes structural evidence:

- page identity and fetch result
- internal URL structure (groups, depth, sections)
- curated metadata
- feed-like URLs
- structured-data / embedded JSON signals (JSON-LD, Next.js data, inline JSON)
- extracted page text

## Install

```bash
cargo install pageinfo-rs
```

Binary name: `pginf`. Library crate: `pageinfo_rs`.

## Update

```bash
cargo install pageinfo-rs --force
```

After updating, re-run `pginf install skills local` (or `global`) to refresh the LLM skill file.

## Library Usage

`PageClient` is the core HTTP client. Usable from any async Rust code:

```rust
use pageinfo_rs::{PageClient, Emulation};

let client = PageClient::builder()
    .proxy("socks5://user:pass@host:port")?
    .browser(Emulation::Chrome137)
    .timeout(std::time::Duration::from_secs(30))
    .build();

let result = client.fetch("https://example.com").await?;
```

### Link Extraction

```rust
use pageinfo_rs::{extract_links, Link};
use pageinfo_rs::dom_content_extraction::scraper::Html;
use url::Url;

let doc = Html::parse_document(&html_body);
let base = Url::parse("https://example.com")?;

let links: Vec<Link> = extract_links(&doc, &base);
```

Also exported: `extract_registered_domain`, `UrlFacts`, `DateKind`.

Features:

- **Proxy support** with inline auth (`socks5://user:pass@host:port`). Falls back to `HTTPS_PROXY`/`HTTP_PROXY` env vars.
- **Browser emulation** via `wreq_util::Emulation` — sets TLS fingerprint and headers. Available: Chrome 100–137, Firefox, Safari, Edge, OkHttp.
- **Automatic fallback** — on 403/429/503 or connection errors, retries with the next browser in the fallback chain. Default chain: Chrome 136, Firefox 139, Safari 18.5.
- **Timeout** — configurable, default 30 seconds.
- **Redirect following** — `fetch()` follows HTTP redirects by default (up to 10 hops). Configurable via `.redirect(wreq::redirect::Policy)`: `Policy::limited(n)`, `Policy::none()` (surface the raw 3xx as an error), or `Policy::custom(...)`. `get_raw()` never follows.

## CLI Commands

### `fetch`

Fetch a page, cache it, and print HTTP metadata.

```bash
pginf fetch https://example.com
pginf fetch https://example.com --json
```

Cache flags:

```bash
pginf fetch https://example.com --refresh
pginf fetch https://example.com --no-cache
```

### `links`

Show URL groups, path depth, and internal/external link structure.

```bash
pginf links https://example.com
pginf links https://example.com --filter internal
pginf links https://example.com --filter external --format toon
pginf links https://example.com --format json
```

### `meta`

Show curated page metadata.

```bash
pginf meta https://example.com
pginf meta https://example.com --format json
pginf meta https://example.com --format toon
```

### `json`

Show structured data signals such as JSON-LD and Next.js data.

```bash
pginf json https://example.com
pginf json https://example.com --json
```

### `text`

Extract page text content.

```bash
pginf text https://example.com
pginf text https://example.com --format json
pginf text https://example.com --format toon
```

### `html`

Show HTML content, optionally filtered by CSS selector. Uses the same page cache
as the analysis commands.

```bash
pginf html -u https://example.com                    # full HTML
pginf html -u https://example.com -s "div.article"   # elements matching selector
pginf html -u https://example.com -s "h1, h2"        # multiple selectors
pginf html -u https://example.com --no-cache         # fresh fetch
```

### `http`

Low-level HTTP debug command. Shows request/response headers, body, and timing.

```bash
pginf http -u https://example.com
```

### `install`

Install pginf skill files for AI coding agents.

```bash
pginf install skills local     # <project>/.agents/skills/pginf/SKILL.md
pginf install skills global    # ~/.agents/skills/pginf/SKILL.md
```

### `help`

Built-in documentation.

```bash
pginf help
pginf help fetch
pginf help links
pginf help meta
pginf help json
pginf help text
pginf help http
pginf help tool
```

## Global Flags

Apply to all commands that fetch pages:

```bash
pginf --proxy socks5://user:pass@host:port fetch https://example.com
pginf --browser chrome131 fetch https://example.com
pginf --timeout 60 fetch https://example.com
```

| Flag | Description |
|---|---|
| `--proxy <URL>` | Proxy URL with optional inline auth |
| `--browser <NAME>` | Browser emulation: `chrome137`, `firefox`, `safari`, `edge`, `okhttp` |
| `--timeout <SECS>` | Request timeout in seconds |

## For LLMs

An LLM tool skill is available at [`skills/pginf.md`](skills/pginf.md). Install it with:

```bash
pginf install skills local     # project-local
pginf install skills global    # user-level
```

After upgrading pginf, re-run the install command to update the skill file. It detects the `installed-by: pginf` marker and overwrites safely.

## Cache

`fetch`, `links`, `meta`, `json`, `text`, and `html` cache fetched pages
locally in `.pginf/`. Stored data: fetch metadata, response headers, raw HTML.

Cache behavior:

- default: read cache on hit, fetch on miss, store result
- `--refresh`: refetch and overwrite cache entry
- `--no-cache`: skip cache read and write

## License

GPL-3.0

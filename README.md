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

## Library Usage

`PageClient` is the core HTTP client. Usable from any async Rust code:

```rust
use pageinfo_rs::{PageClient, Emulation};

let client = PageClient::builder()
    .proxy("socks5://user:pass@host:port")?
    .browser(Emulation::Chrome137)
    .timeout(std::time::Duration::from_secs(30))
    .build();

let cached_page = client.fetch("https://example.com").await?;
```

`pageinfo_rs` re-exports `Emulation`, `wreq`, and `wreq_util` — no extra direct dependencies needed.

Features:

- **Proxy support** with inline auth (`socks5://user:pass@host:port`). Falls back to `HTTPS_PROXY`/`HTTP_PROXY` env vars.
- **Browser emulation** via `wreq_util::Emulation` — sets TLS fingerprint and headers. Available: Chrome 100–137, Firefox, Safari, Edge, OkHttp.
- **Automatic fallback** — on 403/429/503 or connection errors, retries with the next browser in the fallback chain. Default chain: Chrome 136, Firefox 139, Safari 18.5.
- **Timeout** — configurable, default 30 seconds.

## CLI Commands

### `analyze`

Main research command. Uses local page cache by default.

```bash
pginf analyze -u https://example.com
```

Focused views:

```bash
pginf analyze -u https://example.com links
pginf analyze -u https://example.com meta
pginf analyze -u https://example.com json
```

Cache flags:

```bash
pginf analyze -u https://example.com --refresh
pginf analyze -u https://example.com --no-cache
```

### `http`

Low-level HTTP debug command. Shows request/response headers, body, and timing.

```bash
pginf http -u https://example.com
```

### `html`

Show HTML content, optionally filtered by CSS selector. Uses same cache as `analyze`.

```bash
pginf html -u https://example.com                    # full HTML
pginf html -u https://example.com -s "div.article"   # elements matching selector
pginf html -u https://example.com -s "h1, h2"        # multiple selectors
pginf html -u https://example.com --no-cache         # fresh fetch
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
pginf help analyze
pginf help http
pginf help tool
```

## Global Flags

Apply to all commands that fetch pages:

```bash
pginf --proxy socks5://user:pass@host:port analyze -u https://example.com
pginf --browser chrome131 analyze -u https://example.com
pginf --timeout 60 analyze -u https://example.com
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

## Cache

`analyze` and `html` cache fetched pages locally in `.pginf/`. Stored data: fetch metadata, response headers, raw HTML.

Cache behavior:

- default: read cache on hit, fetch on miss, store result
- `--refresh`: refetch and overwrite cache entry
- `--no-cache`: skip cache read and write

## Architecture

```
src/
  client.rs          PageClient — HTTP fetching, proxy, browser emulation, fallback
  http_display.rs    HTTP transaction types and formatting (for `http` command)
  skills.rs          Embedded skill file + install logic (for `install` command)
  analyzer/          Page analysis: link extraction, URL grouping, metadata, structured data
  cache/             File-based page cache (.pginf/)
  html.rs            Legacy page info extraction (used by `http` command)
  help.rs            Built-in help text
  main.rs            CLI entry point
```

All HTTP fetching flows through `PageClient`. No raw `wreq::Client` construction outside of `client.rs`.

## License

GPL-3.0

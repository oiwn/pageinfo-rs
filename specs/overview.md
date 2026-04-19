## Project Direction

`pageinfo-rs` is a CLI tool and Rust library for researching web pages, designed so an LLM can inspect a site and build or adapt a crawler.

## Product Principles

- Prefer evidence over strong built-in heuristics
- Keep output compact and LLM-readable
- Make commands granular so an LLM can ask follow-up questions
- Separate raw data collection from presentation
- Preserve enough raw content for the LLM to reason directly

## Core Components

### `PageClient` (`src/client.rs`)

Single entry point for all HTTP fetching. Returns `FetchResult` (no cache coupling). Supports:

- proxy with inline auth and env var fallback
- browser emulation via `wreq_util::Emulation` (TLS fingerprint + headers)
- automatic fallback on 403/429/503/connection errors
- configurable timeout

Re-exported as `pageinfo_rs::PageClient` and `pageinfo_rs::FetchResult` for library use.

### `resolve_page()` (`src/resolve.rs`)

Shared helper used by all commands that need page data. Handles:
- check cache → return if hit
- fetch via `PageClient` if miss
- store result in cache
- return `ResolveOutput { fetch_result, from_cache }`

### Commands

Top-level commands (no nesting):

- `fetch <url>` — fetch + cache, print HTTP metadata (status, headers, timing)
- `links <url>` — URL groups, path depth, internal/external links
- `meta <url>` — curated metadata (title, lang, description, og:type, etc.)
- `json <url>` — structured data (JSON-LD, Next.js, inline JSON)
- `text <url>` — extracted text content via dom-content-extraction
- `html <url>` — raw HTML, optional CSS selector filter
- `http <url>` — low-level HTTP debug (full request/response)

All commands support `--json` for machine-readable output. Default is markdown.

### `http` Command

Low-level debug command showing full HTTP transaction (request/response headers, body, timing). Uses `http_display.rs` for formatting.

### Cache (`src/cache/`)

File-based page cache in `.pginf/`. Stores fetch metadata, response headers, raw HTML.

## Implementation Status

### Done

- Flat command structure: `fetch`, `links`, `meta`, `json`, `text`, `html`, `http`
- `--json` flag on all analysis commands
- `PageClient` decoupled from cache (returns `FetchResult`)
- `resolve_page()` shared helper for cache-or-fetch logic
- `text` command with dom-content-extraction
- Global CLI flags: `--proxy`, `--browser`, `--timeout`
- File-based page cache with refresh/no-cache support
- URL deduplication and normalization
- Metadata filtering to high-signal fields
- Feed detection
- Structured data detection (JSON-LD, Next.js, inline JSON)
- URL grouping by first path segment
- Path depth distribution

### Not Done Yet

- More granular URL bucketing / similarity clustering
- Sampling across multiple pages
- Better query parameter analysis
- Anchor text sampling in URL groups
- Markdown text extraction via DCE density tree

## Open Questions

- Should the tool generate regex candidates itself? Current direction: no, expose evidence.
- Should `text --format markdown` use DCE density tree extraction? Needs upstream support.

## Specs

- `specs/cache.md` — cache design
- `specs/ctx.md` — current task context
- `specs/ideas.md` — future ideas

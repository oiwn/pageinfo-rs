# Changelog

## v0.2.0

### Breaking changes

- Removed `analyze` command. Replaced by individual top-level commands: `fetch`, `links`, `meta`, `json`, `text`.
- `PageClient::fetch()` now returns `FetchResult` instead of `CachedPage`. No longer writes to cache internally.

### New commands

- `pginf fetch <url>` — fetch and cache a page, print HTTP metadata (status, headers, timing, body size).
- `pginf links <url>` — URL groups, path depth, internal/external link analysis. Supports `--inbound`, `--outbound` flags.
- `pginf meta <url>` — curated metadata (title, lang, description, og:type, etc.).
- `pginf json <url>` — structured data detection (JSON-LD, Next.js, inline JSON).
- `pginf text <url>` — text content extraction via dom-content-extraction. Supports `--format text|markdown`.

### New features

- `--json` flag on all analysis commands for machine-readable output.
- `resolve_page()` shared helper centralizes cache-check → fetch → cache-store logic.
- `FetchResult` struct with `to_cached_page()` conversion method.
- Re-exports `FetchResult` and `dom_content_extraction` from library.

### Changes

- `PageClient` decoupled from cache. Fetch logic returns raw data; caching is handled by the command layer.
- Cache dir renamed from `.pageinfo/` to `.pginf/`.
- All help topics updated to reflect new command surface.
- Behavioral guidelines added to `AGENTS.md`.
- Dependencies: wreq 6, wreq-util 3, dirs 6.

## v0.1.2

- `analyze` command with `links`, `meta`, `json` subcommands
- `PageClient` with proxy, browser emulation, fallback, timeout
- File-based page cache with refresh/no-cache support
- URL grouping, metadata filtering, feed detection, structured data detection

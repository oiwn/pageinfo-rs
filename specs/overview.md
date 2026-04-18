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

Single entry point for all HTTP fetching. Supports:

- proxy with inline auth and env var fallback
- browser emulation via `wreq_util::Emulation` (TLS fingerprint + headers)
- automatic fallback on 403/429/503/connection errors
- configurable timeout

Used by both `analyze` and `http` commands. Re-exported as `pageinfo_rs::PageClient` for library use.

### `analyze` Command

Primary research command. Exposes:

- grouped internal URLs with depth and section info
- curated metadata (description, robots, og:type, article:section, etc.)
- feed detection (RSS, Atom, feed paths)
- structured data (JSON-LD, Next.js data, inline JSON)
- extracted text content

Subcommands: `links`, `meta`, `json` for focused views.

Integrates with file cache. Supports `--refresh` and `--no-cache`.

### `http` Command

Low-level debug command showing full HTTP transaction (request/response headers, body, timing). Uses `http_display.rs` for formatting.

### Cache (`src/cache/`)

File-based page cache in `.pageinfo/`. Stores fetch metadata, response headers, raw HTML. Used by `analyze`.

## Implementation Status

### Done

- `analyze` with focused subcommands (`links`, `meta`, `json`)
- `PageClient` with proxy, browser emulation, fallback, timeout
- Global CLI flags: `--proxy`, `--browser`, `--timeout`
- File-based page cache with refresh/no-cache support
- URL deduplication and normalization
- Metadata filtering to high-signal fields
- Feed detection
- Structured data detection (JSON-LD, Next.js, inline JSON)
- URL grouping by first path segment
- Path depth distribution
- Test coverage: ~82% line coverage

### Not Done Yet

- Machine-readable JSON output for `analyze` (currently markdown only)
- More granular URL bucketing / similarity clustering
- Sampling across multiple pages
- Better query parameter analysis
- Anchor text sampling in URL groups

## Open Questions

- Should `analyze` also expose JSON output? Current direction: markdown is enough for now.
- Should the tool generate regex candidates itself? Current direction: no, expose evidence.
- Should granularity be subcommands or flags? Currently subcommands.

## Specs

- `specs/cache.md` — cache design
- `specs/ctx.md` — current task context
- `specs/ideas.md` — future ideas

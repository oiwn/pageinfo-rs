# Changelog

## 2026-08-10 — Removed Obscura rendering

- Removed the feature-gated `pginf render` command and public renderer API.
  Obscura proved unstable in practice and panicked frequently.
- Removed the git-only Obscura dependency, restored Tokio's default runtime,
  and re-enabled crates.io publishing.
- Recorded Chromium/CDP via `chromiumoxide` as the future rendering direction;
  Servo remains a far-future idea.

## Unreleased

### Breaking changes

- `pginf meta`, `pginf links`, and `pginf text` now use `--format text|json|toon`
  for output selection.
- Removed legacy `pginf meta --json`, `pginf links --json`, and
  `pginf text --json`.
- Replaced `pginf links --inbound/--outbound` with
  `pginf links --filter all|internal|external`.
- Removed `pginf text --format markdown`; markdown text rendering is deferred.
- Removed the `extract_internal_links` compatibility wrapper from the public
  library API. Filter `extract_links()` results by `Link::is_internal` instead.

### New features

- `PageClient::fetch()` now follows HTTP redirects (default `Policy::limited(10)`);
  `FetchResult.final_url` reflects the post-redirect URL.
- `PageClientBuilder::redirect(wreq::redirect::Policy)` — full control from the
  caller (`limited(n)`, `none()`, `custom(...)`). `Policy::none()` surfaces the raw
  3xx as `ClientError::Fetch`. Ignored by `get_raw()` (`http` command keeps showing
  the first hop).
- Shared typed output/rendering system with `OutputFormat` and `RenderOutput`.
- `pginf meta`, `pginf links`, and `pginf text` support TOON output via
  `--format toon`.
- `pginf links` now renders processed link rows with preserved `raw_url`,
  resolved absolute `url`, text, rel, and internal/external classification.
- `pginf text --format json|toon` returns `url`, `content`, and
  `content_length`.
- Public link extraction API: `extract_links`, `extract_raw_links`,
  `extract_registered_domain`, `Link`, `RawLink`, `LinkOptions`, `UrlFacts`,
  `DateKind` available at crate root.
- `Link::normalize()` — lowercases host, drops fragment.
- `Link::strip_tracking()` — removes `utm_*`, `fbclid`, `gclid` query params.
- `Link::is_same_host(&Url) -> bool` — exact host comparison.
- `Link::is_asset() -> bool` — detects css, js, png, jpg, svg, woff2, pdf, etc.
- `LinkOptions { normalize, strip_tracking_params, max }` with `Default`.
- `extract_links()` now normalizes all links by default (lowercase host, no fragment).
- `FetchResult` now records `emulation_used`, `proxy_used` (masked), and `attempts` for fetch transparency.
- `analyzer::link`, `analyzer::url_facts`, `analyzer::date_kind` modules are now `pub mod`.
- Downgraded wreq to 5.3, wreq-util to 2.2.6 (stable releases).

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
- Dependencies: wreq 5, wreq-util 2, dirs 6.

## v0.1.2

- `analyze` command with `links`, `meta`, `json` subcommands
- `PageClient` with proxy, browser emulation, fallback, timeout
- File-based page cache with refresh/no-cache support
- URL grouping, metadata filtering, feed detection, structured data detection

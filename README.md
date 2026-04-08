# pageinfo-rs

`pageinfo-rs` is a CLI tool for researching web pages so an LLM can help build or adapt crawlers.

It is HTTP-only. It does not use browser automation.

The tool is designed to return evidence, not final decisions:

- page identity and fetch result
- internal URL structure
- curated metadata
- feed-like URLs
- structured-data / embedded JSON signals
- extracted page text

## Install

```bash
cargo install pageinfo-rs
```

This installs the `pginf` binary.

## What It Is For

Use `pginf` when you want to inspect a site page and understand:

- what kinds of internal URLs it links to
- what page-level metadata exists
- whether the page exposes feeds
- whether the page contains JSON-LD or framework bootstrap data
- what the extracted text content looks like

This is useful when an LLM needs grounded page evidence before writing crawler rules or site-specific config.

## Commands

### `analyze`

Main research command.

Default full report:

```bash
pginf analyze -u https://example.com
```

Focused link view:

```bash
pginf analyze -u https://example.com links
```

Focused metadata view:

```bash
pginf analyze -u https://example.com meta
```

Focused structured-data / embedded-JSON view:

```bash
pginf analyze -u https://example.com json
```

Cache flags:

```bash
pginf analyze -u https://example.com --refresh
pginf analyze -u https://example.com --no-cache
```

What `analyze` returns:

- full report: header, summary, curated metadata, URL groups, structured-data summary, extracted content
- `links`: URL grouping and path-depth evidence
- `meta`: curated metadata only
- `json`: structured-data summary only

Cache behavior:

- default: read cache on hit, fetch on miss, store fetched raw page
- `--refresh`: skip cache read, fetch again, overwrite cache entry
- `--no-cache`: do not read or write cache

Current caveat:

- focused analyze views currently use the syntax `pginf analyze -u <URL> links`
  rather than `pginf analyze links -u <URL>`

### `http`

Low-level HTTP debug command.

```bash
pginf http -u https://example.com
```

Use it when you need transport-level detail:

- request URL and headers
- response status and headers
- raw response body
- timing

This is for fetch/debugging, not the normal research workflow.

### `help`

Built-in documentation for humans and LLM tools.

```bash
pginf help
pginf help analyze
pginf help http
pginf help tool
```

`pginf help tool` is the compact built-in guide intended for agent/tool usage.

## Typical Workflow

1. Start with:

```bash
pginf analyze -u https://example.com
```

2. Narrow the view if needed:

```bash
pginf analyze -u https://example.com links
pginf analyze -u https://example.com meta
pginf analyze -u https://example.com json
```

3. If fetch behavior itself looks suspicious:

```bash
pginf http -u https://example.com
```

4. If you need the built-in guide:

```bash
pginf help tool
```

## Cache

`analyze` uses a local raw-page cache by default.

Current cache root:

```text
.pageinfo/
```

Stored data is raw-only:

- fetch metadata
- response headers
- raw HTML

See [`specs/cache.md`](specs/cache.md) for the current cache specification.

## Current Status

The command set is intentionally small right now:

- `analyze`
- `http`
- `help`

The tool is still evolving, especially the shape and quality of `analyze` output. Treat the output as evidence for reasoning, not as ground truth.

## License

GPL-3.0

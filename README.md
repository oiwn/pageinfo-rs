# pageinfo-rs

CLI tool that analyzes web pages and produces structured, LLM-friendly output so an LLM agent can generate crawler configs without manually inspecting the site.

HTTP only. No browser automation.

## Install

```bash
cargo install pageinfo-rs
```

This installs the `pginf` binary.

## Usage

### Analyze a single page

```bash
pginf analyze -u https://example.com
```

Extracts links, metadata, URL patterns, and main content text.

### Sample multiple pages

```bash
pginf sample -u https://example.com -m 10 -c 3 -o ./output
```

Fetches the seed page plus sampled internal pages, builds aggregate statistics, and persists artifacts (raw HTML, JSON, report).

### Raw HTTP dump

```bash
pginf http -u https://example.com
```

Dumps the full HTTP request/response transaction.

## Features

- **URL pattern detection** — infers article URL patterns (e.g. `/{section}/{year}/{month}/{day}/{slug}`)
- **Date detection** — contextual year/month/day identification in URL paths
- **Section grouping** — links grouped by first path segment
- **Content extraction** — main body text via CETD algorithm (dom-content-extraction)
- **Aggregate stats** — cross-page URL statistics when sampling multiple pages
- **Table output** — structured comfy-table output ready for LLM consumption

## License

MIT

# Project State

## v0.2.0 — current

**CLI commands:** `fetch`, `links`, `meta`, `json`, `text`, `html`, `http`, `install skills {local|global}`, `help`

Removed: `analyze` (replaced by individual top-level commands).

### Command surface

```
pginf fetch <url>                         # fetch + cache, print HTTP metadata only
pginf fetch <url> --json                  # same, JSON output

pginf links <url>                         # URL groups, path depth, samples
pginf links <url> --inbound               # internal links only
pginf links <url> --outbound              # external links only
pginf links <url> --json

pginf meta <url>                          # curated metadata (title, lang, meta tags)
pginf meta <url> --json

pginf json <url>                          # structured data: JSON-LD, Next.js, inline
pginf json <url> --json

pginf text <url>                          # dom-content-extraction plaintext
pginf text <url> --format markdown        # DCE as markdown (currently same as text)
pginf text <url> --json

pginf html <url> [-s <selector>]          # raw HTML, optional CSS filter
pginf http <url>                          # low-level HTTP debug
```

Global flags: `--proxy`, `--browser`, `--timeout`, `--refresh`, `--no-cache`
All commands default to markdown output. `--json` for machine-readable.

### Architecture

- `client.rs` — PageClient, returns `FetchResult` (no cache coupling)
- `resolve.rs` — shared `resolve_page()` (cache-check → fetch → cache-store)
- `analyzer/` — link extraction, URL grouping, metadata, structured data
- `cache/` — file-based page cache
- `http_display.rs` — HTTP transaction formatting
- `html.rs` — legacy page info (used by `http` command)
- `help.rs` — built-in help text
- `skills.rs` — skill file install

### JSON output shapes

**fetch --json:** `{input_url, final_url, status, headers{}, duration_ms, cached, body_size}`
**links --json:** `{url, total_internal, total_external, groups[{section, count, samples[]}], depth_distribution[], utility_urls[]}`
**meta --json:** `{url, title, lang, tags[{name, content}]}`
**json --json:** `{url, json_ld_count, kinds[]}`
**text --json:** `{url, format, content, content_length}`

### Key deps

wreq 6, wreq-util 3, dirs 6, scraper (via dom-content-extraction)

### Cache

`.pginf/` in CWD (const `CACHE_DIR` in `cache/types.rs`)

### Re-exports (lib.rs)

`PageClient`, `FetchResult`, `Emulation`, `dom_content_extraction`, `wreq`, `wreq_util`

**Tests:** 106 passing
**Manual QA set:** `specs/qa.md`

### Not Done Yet

- Markdown text extraction via DCE density tree (needs upstream support)
- More granular URL bucketing / similarity clustering
- Sampling across multiple pages
- Better query parameter analysis
- Anchor text sampling in URL groups

---

## v0.1.2 — previous

**CLI commands:** `analyze`, `http`, `html`, `install skills {local|global}`, `help`
**Tests:** 91 passing

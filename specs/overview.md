# pageinfo-rs — Project Overview

**Purpose:** CLI tool that analyzes web pages and produces structured output
(LLM-friendly) so an LLM agent can generate crawler configs without manually
inspecting the site.

HTTP only. No browser automation.

---

## CLI Commands

| Command | Description |
|---|---|
| `analyze -u <URL>` | Fetch single URL, extract links/metadata/URL patterns |
| `sample -u <URL> [-m N] [-c N] [-o DIR]` | Fetch seed + sample pages, build aggregate stats, persist artifacts |
| `http -u <URL>` | Raw HTTP transaction dump (request/response/timing) |

---

## Source Layout

```
src/
  lib.rs                          — pub mod html, http, analyzer
  main.rs                         — CLI entry (clap)

  http.rs                         — HttpTransaction, retrieve_page() via wreq
  html.rs                         — Legacy PageInfo (title/meta only, used by Http command)

  analyzer/
    mod.rs                        — Re-exports: PageInfo, SampleCollector, SampleOptions
    error.rs                      — AnalyzerError enum (Fetch, Parse, InvalidUrl, Io)
    meta_tag.rs                   — MetaTag { name, content }
    link.rs                       — Link struct + extract_links() from HTML
    date_kind.rs                  — DateKind enum + classify_segment()
    url_facts.rs                  — UrlFacts::from_links(), URL pattern detection, AggregateBuilder
    page_info.rs                  — analyzer::PageInfo::fetch() + format_for_llm() with comfy-table
    aggregate.rs                  — AggregateUrlFacts + cross-page merge
    sample_options.rs             — SampleOptions { max_pages, concurrency }
    sample_collector.rs           — SampleCollector::collect(), storage, format_for_llm()
```

---

## Data Flow

### `analyze` command

```
URL → wreq GET → HTML parse (dom-content-extraction/scraper)
  → extract title, lang, meta tags
  → extract main content text via dom-content-extraction CETD algorithm
  → extract all <a href> links → resolve to absolute URLs
  → UrlFacts::from_links() → section grouping, depth distribution, date detection
  → format_for_llm() → comfy-table output
```

### `sample` command

```
Seed URL → analyzer::PageInfo::fetch()
  → collect internal links → group by first path segment
  → pick one URL per group (prefer date-like paths) up to max_pages
  → fetch concurrently (semaphore for concurrency limit)
  → AggregateUrlFacts::from_page_facts() — merge across all pages
  → persist: raw HTML, page JSON, aggregate JSON, report.md via tokio::fs
```

---

## Key Data Structures

### analyzer::PageInfo

Fetched page with full analysis. Fields: `url`, `final_url`, `domain`, `status`,
`title`, `lang`, `meta: Vec<MetaTag>`, `links: Vec<Link>`, `url_facts: UrlFacts`,
`raw_html`, `text_content: Option<String>`.

### UrlFacts

Computed from internal links on a page:

- `total_internal` / `total_external` — link counts
- `depth_distribution` — path depth histogram
- `top_first_segments` — most common first path segments (top 20)
- `url_samples_by_section` — actual URL path samples per section (up to 8 unique)
- `date_positions` — detected Year/Month/Day positions in URL paths
- `likely_utility_urls` — URLs matching utility keywords (about, privacy, terms, etc.)
- `detected_url_pattern()` — infers article URL pattern like `/{section}/{year}/{month}/{day}/{slug}`

### Date Detection

Contextual: finds Year positions first (4-digit, 1900-2100), then checks
adjacent positions for Month (1-12) and Day (1-31) ranges. Avoids ambiguity
between month/day by using position relative to detected year.

### Link

`{ url, text, rel, is_internal }`. Internal means same registered domain
(e.g. "coindesk.com" from "www.coindesk.com").

### AggregateUrlFacts

Merged UrlFacts across multiple pages: summed counts, unioned URL samples
(deduped), unioned utility URLs.

---

## Output Format

All output uses `comfy-table` (UTF8_FULL_CONDENSED preset) for tables.

Sections in `analyze` output:
1. **Header table** — URL, final URL, status, title, lang
2. **Links** — internal/external count
3. **Meta Tags** — property/content table
4. **URL Patterns** — detected article pattern + section table (section name, link count, sample URLs)
5. **Path Depth** — depth/count table
6. **Utility URLs** — list
7. **Extracted Content** — main body text (CETD algorithm)

`sample` output: aggregate header + same structure, then per-page sections.

---

## Dependencies

| Crate | Role |
|---|---|
| `wreq` | HTTP client (not reqwest) |
| `comfy-table` | Terminal table rendering |
| `dom-content-extraction` | HTML parsing (re-exports scraper) + content extraction via CETD |
| `serde` / `serde_json` | Serialization |
| `sha2` | URL hashing for storage keys |
| `tokio` | Async runtime |
| `url` | URL parsing |
| `clap` | CLI argument parsing |
| `thiserror` | Error types |

---

## Specs

- `specs/idea.md` — Original design specification
- `specs/ctx.md` — Implementation plan and context (partially stale, check against actual code)

# Site Analyzer

Specification for the site analyzer library — a tool that collects structured
information about a web page or set of pages to help an LLM agent generate
crawler configs.

HTTP only. No browser automation.

---

## Purpose

Given a URL, gather enough structured information about a site's link patterns,
URL taxonomy, and page metadata so that an LLM agent can write a crawler config
without manually inspecting the site.

---

## Two Modes

### 1. `PageInfo` — Single URL Analysis

Fetches one URL and extracts structured data from it.

```rust
pub struct PageInfo {
    pub url: String,            // original requested URL
    pub final_url: String,      // after redirects
    pub domain: String,         // e.g. "coindesk.com"
    pub status: u16,
    pub title: Option<String>,
    pub meta: Vec<MetaTag>,
    pub links: Vec<Link>,
    pub url_facts: UrlFacts,    // computed from links
}

pub struct MetaTag {
    pub name: Option<String>,   // name= or property= attribute
    pub content: Option<String>,
}

pub struct Link {
    pub url: String,            // absolute URL, resolved against page base
    pub text: Option<String>,   // anchor text, trimmed
    pub rel: Option<String>,    // rel attribute if present
    pub is_internal: bool,      // same registered domain as the page
}
```

#### `UrlFacts`

Raw observations computed from all internal links on the page. No
interpretation — just facts. The LLM draws conclusions.

```rust
pub struct UrlFacts {
    pub total_internal: usize,
    pub total_external: usize,

    /// Distribution of path depth among internal URLs.
    /// Key: depth (number of segments), Value: count.
    /// e.g. {1: 12, 2: 3, 4: 47} means 47 URLs have 4-segment paths.
    pub depth_distribution: BTreeMap<usize, usize>,

    /// Most common first path segments and their occurrence count.
    /// Sorted by count descending. e.g. [("markets", 34), ("tech", 21)]
    pub top_first_segments: Vec<(String, usize)>,

    /// For each path depth, sample of distinct values seen at each segment
    /// position. Up to 5 examples per position.
    /// e.g. depth 4 → position 0: ["markets", "tech"], position 1: ["2026"], ...
    pub segment_samples: BTreeMap<usize, Vec<Vec<String>>>,

    /// Positions that appear to contain dates (4-digit year, 2-digit month/day).
    /// e.g. [(1, DateKind::Year), (2, DateKind::Month), (3, DateKind::Day)]
    pub date_positions: Vec<(usize, DateKind)>,

    /// URLs containing known utility keywords — not content.
    /// Keywords: about, contact, privacy, terms, login, careers,
    /// advertise, newsletter, sitemap, rss, feed, help, faq
    pub likely_utility_urls: Vec<String>,
}

pub enum DateKind {
    Year,   // matches \d{4}
    Month,  // matches 0[1-9]|1[0-2]
    Day,    // matches 0[1-9]|[12]\d|3[01]
}
```

**Usage:**

```rust
let info = PageInfo::fetch("https://www.coindesk.com/latest-crypto-news", &client).await?;
let report = info.format_for_llm(); // Markdown string
```

---

### 2. `SampleCollector` — Multi-URL Collection

Fetches a seed URL, discovers internal links, samples a subset of them, and
builds aggregate statistics across all fetched pages.

```rust
pub struct SampleCollector {
    pub seed_url: String,
    pub domain: String,
    pub pages: Vec<PageInfo>,       // seed + all sampled pages
    pub aggregate: AggregateUrlFacts,
}

pub struct AggregateUrlFacts {
    pub total_urls_seen: usize,     // across all pages, deduplicated
    pub depth_distribution: BTreeMap<usize, usize>,
    pub top_first_segments: Vec<(String, usize)>,
    pub segment_samples: BTreeMap<usize, Vec<Vec<String>>>,
    pub date_positions: Vec<(usize, DateKind)>,
    pub likely_utility_urls: Vec<String>,
}
```

#### Sampling Strategy

1. Fetch seed URL → collect all internal links
2. Deduplicate by URL
3. Select up to `max_pages` URLs to fetch (default: 5):
   - Group links by first path segment
   - Pick one URL per group (prefer URLs with date-like segments in path)
   - If groups < `max_pages`, fill remaining slots with any unvisited URLs
4. Fetch selected URLs concurrently (respect `concurrency` limit, default: 3)
5. Merge `UrlFacts` across all fetched pages into `AggregateUrlFacts`

```rust
pub struct SampleOptions {
    pub max_pages: usize,       // default: 5
    pub concurrency: usize,     // default: 3
}

impl Default for SampleOptions {
    fn default() -> Self {
        Self { max_pages: 5, concurrency: 3 }
    }
}
```

#### Storage

`SampleCollector` writes artifacts to a caller-provided `object_store::DynObjectStore`.

Per fetched page:
- Raw HTML: `{domain}/pages/{url_hash}.html`
- Page facts JSON: `{domain}/pages/{url_hash}.json`

Aggregate output:
- `{domain}/aggregate.json` — serialized `AggregateUrlFacts`
- `{domain}/report.md` — same content as `format_for_llm()` output

`url_hash` is the first 16 chars of the SHA-256 hex of the normalized URL.

The caller constructs and passes the store — the library does not own storage
configuration. Example:

```rust
let store = Arc::new(LocalFileSystem::new_with_prefix("/tmp/site-analyzer")?);

let collector = SampleCollector::collect(
    "https://www.coindesk.com/latest-crypto-news",
    SampleOptions::default(),
    &client,
    store,
).await?;

let report = collector.format_for_llm();
```

---

## Markdown Output Format

Both `PageInfo` and `SampleCollector` implement `format_for_llm() -> String`
returning a Markdown document.

### `PageInfo::format_for_llm()` example:

```markdown
# Page Analysis: coindesk.com

**URL:** https://www.coindesk.com/latest-crypto-news
**Final URL:** https://www.coindesk.com/latest-crypto-news
**Status:** 200
**Title:** Latest Crypto News | CoinDesk
**Lang:** en

## Meta Tags
- description: "The latest crypto news and information..."
- og:type: article

## Link Summary
- Internal links: 87
- External links: 12

## URL Facts

### Path Depth Distribution
| Depth | Count |
|-------|-------|
| 1     | 14    |
| 2     | 6     |
| 4     | 67    |

### Top First Segments
| Segment  | Count |
|----------|-------|
| markets  | 34    |
| tech     | 21    |
| policy   | 8     |
| price    | 6     |
| about    | 3     |

### Segment Samples by Depth

**Depth 4:**
- Position 0 (5 examples): markets, tech, policy, business, sponsored-content
- Position 1 (3 examples): 2026, 2025, 2024
- Position 2 (3 examples): 04, 03, 12
- Position 3 (3 examples): 06, 05, 28
- Position 4 (3 examples): bitcoin-hits-new-high, ethereum-upgrade-details, ...

**Depth 2:**
- Position 0 (3 examples): price, research, video
- Position 1 (5 examples): bitcoin, ethereum, xrp, solana, cardano

**Depth 1:**
- Position 0 (5 examples): markets, tech, about, privacy, contact-us

### Detected Date Positions
- Position 1: Year (e.g. 2026)
- Position 2: Month (e.g. 04)
- Position 3: Day (e.g. 06)

### Likely Utility URLs
- https://www.coindesk.com/about
- https://www.coindesk.com/contact-us
- https://www.coindesk.com/privacy
- https://www.coindesk.com/terms
- https://www.coindesk.com/advertise
- https://www.coindesk.com/newsletters
- https://www.coindesk.com/sitemap
```

### `SampleCollector::format_for_llm()`

Same structure as above but with an additional **Aggregate** section at the
top summarizing stats across all sampled pages, followed by per-page sections.

---

## HTTP Client

The library accepts a shared HTTP client — it does not create one internally.

Expected client interface: `reqwest::Client` or compatible.

Caller is responsible for:
- User-Agent configuration
- Timeout settings (recommended: 10s connect, 30s read)
- Retry logic

The library surfaces errors — it does not retry.

---

## Error Handling

```rust
pub enum AnalyzerError {
    Fetch { url: String, status: u16 },
    Timeout { url: String },
    Parse { url: String, reason: String },
    InvalidUrl(String),
    Storage(object_store::Error),
}
```

Partial failures in `SampleCollector` (one sampled page fails) are logged and
skipped — the collector continues with the remaining pages.

---

## Dependencies

- `reqwest` — HTTP client
- `scraper` — HTML parsing and link extraction
- `url` — URL parsing and normalization
- `object_store` — storage abstraction
- `serde` / `serde_json` — serialization
- `sha2` — URL hashing for storage keys
- `tokio` — async runtime

---

## Out of Scope

- Browser / JS rendering
- Authentication
- robots.txt handling
- Generating the crawler config (LLM agent's responsibility)
- Rate limiting (caller's responsibility)

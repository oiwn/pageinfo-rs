# Page Cache Specification

## Purpose

The current cache is a V1 raw-page cache used by `analyze`.

Its job is simple:

- avoid refetching pages
- keep fetched HTML locally inspectable
- separate fetch storage from page analysis

The cache is infrastructure, not analysis storage.

## Scope

Implemented scope:

- local cache rooted at `.pageinfo/`
- one entry per normalized final URL
- store raw fetch metadata, response headers, and raw HTML
- load cached raw pages for `analyze`
- support `--no-cache` and `--refresh` on `analyze`

Not in scope in V1:

- derived analysis storage
- links storage
- reports
- indexes
- domain summaries
- session logs
- cache eviction

## Location

Default cache root:

```text
.pageinfo/
```

This lives in the current working directory.

## Directory Layout

Current layout:

```text
.pageinfo/
  VERSION
  pages/
    <cache-key>/
      fetch.json
      headers.json
      page.html
```

There are no index files in V1.

## Cache Key

Each entry is keyed by normalized final URL.

Flow:

1. fetch URL
2. capture final response URL
3. normalize final URL
4. hash normalized final URL with SHA-256
5. store under `.pageinfo/pages/<hash>/`

The original input URL is still stored in metadata.

## URL Normalization

Normalization uses the `url` crate.

Current rules:

- lowercase scheme
- lowercase host
- remove fragment
- remove default port for `http` and `https`

Current non-rules:

- do not reorder query params
- do not strip query params
- do not alter path case

This is intentionally conservative.

## Stored Files

### `VERSION`

Plain text cache schema version.

Current version:

```text
1
```

If the on-disk version does not match, cache initialization fails with a version mismatch error.

### `fetch.json`

Raw fetch metadata.

Current fields:

- `input_url`
- `final_url`
- `normalized_final_url`
- `status`
- `fetched_at`

Current `fetched_at` format:

- Unix timestamp in seconds, stored as a string

Example:

```json
{
  "input_url": "https://example.com",
  "final_url": "https://example.com/news",
  "normalized_final_url": "https://example.com/news",
  "status": 200,
  "fetched_at": "1775600000"
}
```

### `headers.json`

Response headers stored as:

```json
{
  "content-type": "text/html; charset=utf-8"
}
```

Current behavior:

- headers are flattened into `HashMap<String, String>`
- duplicate headers are not preserved separately

### `page.html`

Raw response body as text.

This is the source of truth for later parsing.

## Rust Structures

Current main types:

### `CacheConfig`

- `root_dir: PathBuf`
- `enabled: bool`
- `refresh: bool`

### `CacheKey`

- `normalized_final_url: String`
- `hash: String`

### `CachedFetch`

- `input_url: String`
- `final_url: String`
- `normalized_final_url: String`
- `status: u16`
- `fetched_at: String`

### `CachedPage`

- `fetch: CachedFetch`
- `headers: HashMap<String, String>`
- `html: String`

## Interface

The cache interface is intentionally small.

Current trait:

```rust
pub trait Cache {
    fn init(&self) -> Result<(), CacheError>;
    fn key_for_final_url(&self, final_url: &str) -> Result<CacheKey, CacheError>;
    fn load(&self, key: &CacheKey) -> Result<Option<CachedPage>, CacheError>;
    fn store(&self, page: CachedPage) -> Result<CacheKey, CacheError>;
    fn delete(&self, key: &CacheKey) -> Result<(), CacheError>;
}
```

Current file-backed implementation:

- `FileCache`

Additional helper methods currently used:

- `is_enabled()`
- `should_refresh()`
- `cache_page(...)`

## Analyze Behavior

Current `analyze` behavior:

- default:
  - try cache load first
  - fetch on cache miss
  - store fetched page in cache

- `--no-cache`:
  - do not read cache
  - do not write cache

- `--refresh`:
  - skip cache read
  - fetch again
  - overwrite cache entry

- `--no-cache` with `--refresh`:
  - rejected by CLI argument parsing

## Write Semantics

Current write flow:

1. create `.pageinfo/pages/<hash>/` if missing
2. write `fetch.json`
3. write `headers.json`
4. write `page.html`

There is no temp-file strategy in V1.

## Failure Semantics

Current behavior:

- cache initialization failure is an error
- cache write failure is an error
- missing cache files for an entry are treated as cache miss

## Important Current Limitation

Lookup currently uses `key_for_final_url(url)` before fetch in `analyze`.

That means:

- entries are stored by normalized final URL correctly
- but a later lookup may miss if the requested URL redirects to a different final URL

This is a known limitation of the current V1 implementation.

Fixing it will require one of:

- an input URL to final URL index
- alias files
- lookup by both requested and final URL forms

## Module Layout

Current code layout:

```text
src/cache.rs
src/cache/
  error.rs
  key.rs
  store.rs
  types.rs
```

## Future Extensions

Possible later work:

- configurable cache root
- derived analysis cache
- URL indexes
- better timestamp format
- atomic writes
- cache inspection commands

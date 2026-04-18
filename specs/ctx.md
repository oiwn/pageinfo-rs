# Current Task: HTTP Client Library Layer

## Goal

Expose pageinfo's HTTP fetching as a reusable library with proxy support and automatic browser emulation fallback. Both the CLI and external crates can use it.

## New Module: `src/client.rs`

### `PageClient` Builder

```rust
let client = PageClient::builder()
    .proxy("socks5://user:pass@host:port")?
    .browser(Emulation::Chrome131)
    .fallback_browsers([
        Emulation::Chrome120,
        Emulation::Firefox,
        Emulation::Safari,
    ])
    .build()?;

let page: CachedPage = client.fetch("https://example.com").await?;
```

### Builder fields

| Field | Type | Default | Notes |
|---|---|---|---|
| `proxy` | `Option<String>` | `None` | Parsed via `wreq::Proxy::all()`. Auth inline in URL. Falls back to `HTTPS_PROXY` / `HTTP_PROXY` env var if not set. |
| `browser` | `Option<Emulation>` | `None` | Primary browser emulation via `wreq_util::Emulation`. Sets TLS fingerprint + headers on `wreq::ClientBuilder::emulation()`. |
| `fallback_browsers` | `Vec<Emulation>` | `[Chrome120, Firefox, Safari]` | Tried in order on fetch failure. |
| `max_retries` | `usize` | `3` | Cap on total attempts (primary + fallbacks). |

### `fetch(url: &str) -> Result<CachedPage>`

1. Build `wreq::Client` from current browser config + proxy.
2. `GET` the URL, read body, wrap into `CachedPage` (existing type).
3. On failure, retry with next browser from `fallback_browsers`, up to `max_retries`.

**Fallback triggers:** connection errors, HTTP 403, 429, 503.

**No fallback on:** HTTP 200, 301-308 (redirects handled by wreq), 404, 400, other 4xx.

### `Emulation` parsing from string

Map CLI strings to `wreq_util::Emulation` variants:

- `chrome131`, `chrome130`, `chrome129`, ..., `chrome100` -> corresponding Chrome variant
- `firefox` -> `Firefox`
- `safari` -> `Safari`
- `edge` -> `Edge`
- `okhttp` -> `OkHttp`

Expose as `fn parse_browser(name: &str) -> Result<Emulation>`.

## Changes to Existing Code

### `src/analyzer/page_info.rs`

- `fetch_raw(url, &wreq::Client)` -> `fetch_raw(url, &PageClient)`
- Inside: call `client.fetch(url)` instead of `client.get(parsed).send().await`

### `src/http.rs`

- `retrieve_page(url)` -> `retrieve_page(url, &PageClient)`
- Replace internal `wreq::Client` construction with the passed `PageClient`.

### `src/main.rs`

Add global CLI flags on `Cli` struct (not per-subcommand):

```
pginf --proxy <URL> --browser <NAME> analyze -u <URL>
pginf --proxy <URL> --browser <NAME> http -u <URL>
```

| Flag | Type | Description |
|---|---|---|
| `--proxy` | `Option<String>` | Proxy URL with optional inline auth. |
| `--browser` | `Option<String>` | Browser emulation name (e.g. `chrome131`, `firefox`). |

Both flags are optional. When omitted: no proxy, no browser emulation (current default behavior).

In `main()`: build one `PageClient` from flags, pass it to `analyze` and `http` handlers.

### `src/lib.rs`

Re-export: `pub mod client;` and `pub use client::PageClient;`

## No Changes To

- Cache layer (`src/cache/`) — operates on `CachedPage`, unchanged.
- Analyzer output/formatting — works on parsed HTML, unchanged.
- Dependencies — `wreq` 5 and `wreq-util` 2 already in `Cargo.toml`.

## Task Order

1. Create `src/client.rs` — `PageClient` builder, proxy parsing, emulation wiring, `fetch()` with fallback retry logic.
2. Add `parse_browser()` string -> `Emulation` mapper.
3. Update `src/analyzer/page_info.rs` — accept `&PageClient`.
4. Update `src/http.rs` — accept `&PageClient`.
5. Update `src/main.rs` — global `--proxy` / `--browser` flags, build `PageClient`, pass to handlers.
6. Update `src/lib.rs` — re-export `client` module and `PageClient`.
7. Tests — builder construction, proxy parsing, browser name parsing, fallback behavior.

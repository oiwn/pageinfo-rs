pub fn render(topic: Option<&str>) -> String {
    match topic.map(|t| t.trim().to_ascii_lowercase()) {
        None => general_help(),
        Some(topic) if topic.is_empty() => general_help(),
        Some(topic) if topic == "analyze" => analyze_help(),
        Some(topic) if topic == "http" => http_help(),
        Some(topic) if topic == "tool" => tool_help(),
        Some(topic) => unknown_help(&topic),
    }
}

fn general_help() -> String {
    [
        "# pginf",
        "",
        "Purpose: research web pages so an LLM can inspect site structure and help build or adapt crawlers.",
        "",
        "## Commands",
        "",
        "- `pginf analyze -u <URL>`: main research command; returns the full page report",
        "- `pginf analyze -u <URL> links`: focused link and URL-group view",
        "- `pginf analyze -u <URL> meta`: focused curated-metadata view",
        "- `pginf analyze -u <URL> json`: focused structured-data / embedded-JSON view",
        "- `pginf http -u <URL>`: low-level HTTP debug command; returns request/response details",
        "- `pginf help [topic]`: built-in guide for humans and LLMs",
        "",
        "## Typical Workflow",
        "",
        "1. Start with `pginf analyze -u <URL>`.",
        "2. Use `pginf analyze --refresh` if you need a fresh fetch.",
        "3. Use `pginf analyze --no-cache` if you do not want cache read/write.",
        "4. Use `pginf http -u <URL>` when fetch behavior itself needs debugging.",
        "",
        "## Topics",
        "",
        "- `pginf help analyze`",
        "- `pginf help http`",
        "- `pginf help tool`",
    ]
    .join("\n")
}

fn analyze_help() -> String {
    [
        "# `pginf analyze`",
        "",
        "Purpose: fetch or load one page and return a structured markdown report for crawler research.",
        "",
        "## What It Returns",
        "",
        "- default view: full page report",
        "- `links`: link evidence, section grouping, sample URLs, path-depth summary",
        "- `meta`: curated metadata only",
        "- `json`: structured-data / embedded-JSON summary only",
        "",
        "## Cache Behavior",
        "",
        "- default: read cache on hit, fetch on miss, store fetched raw page",
        "- `--refresh`: skip cache read, fetch again, overwrite cache entry",
        "- `--no-cache`: do not read or write cache",
        "",
        "## Examples",
        "",
        "- `pginf analyze -u https://example.com`",
        "- `pginf analyze -u https://example.com links`",
        "- `pginf analyze -u https://example.com meta`",
        "- `pginf analyze -u https://example.com json`",
        "- `pginf analyze -u https://example.com --refresh`",
        "- `pginf analyze -u https://example.com --no-cache`",
        "",
        "## Notes",
        "",
        "- this command provides evidence, not final crawler configs",
        "- extracted content is intentionally kept so an LLM can inspect the page itself",
    ]
    .join("\n")
}

fn http_help() -> String {
    [
        "# `pginf http`",
        "",
        "Purpose: inspect low-level HTTP behavior for one URL.",
        "",
        "## What It Returns",
        "",
        "- request method and URL",
        "- request headers",
        "- response status",
        "- response headers",
        "- raw response body",
        "- request timing",
        "",
        "## When To Use It",
        "",
        "- page fetches fail unexpectedly",
        "- redirects, headers, or transport behavior need inspection",
        "- you need to compare fetch behavior with `analyze` output",
        "",
        "## Example",
        "",
        "- `pginf http -u https://example.com`",
    ]
    .join("\n")
}

fn tool_help() -> String {
    [
        "# Tool Guide",
        "",
        "This tool helps inspect a web page so an LLM can reason about crawler construction or adaptation.",
        "",
        "## Recommended First Step",
        "",
        "Run `pginf analyze -u <URL>` first.",
        "",
        "## Command Choice",
        "",
        "- use `analyze` for the full page report",
        "- use `analyze ... links` for focused URL/group evidence",
        "- use `analyze ... meta` for curated metadata only",
        "- use `analyze ... json` for structured-data evidence",
        "- use `http` for request/response debugging only",
        "",
        "## Output Expectations",
        "",
        "- `analyze` returns structured markdown intended for reading or passing to an LLM",
        "- `http` returns transport-level debugging output",
        "",
        "## Cache",
        "",
        "- cache is enabled by default for `analyze`",
        "- `--refresh` forces a refetch",
        "- `--no-cache` disables cache read/write for that invocation",
        "",
        "## Caveats",
        "",
        "- current cache lookup can miss if the requested URL redirects to a different final URL",
        "- current `analyze` output is still being improved and should be treated as evidence, not ground truth",
    ]
    .join("\n")
}

fn unknown_help(topic: &str) -> String {
    [
        format!("# Unknown Help Topic: `{topic}`"),
        "".to_string(),
        "Available topics: `analyze`, `http`, `tool`".to_string(),
    ]
    .join("\n")
}

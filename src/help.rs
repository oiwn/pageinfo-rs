pub fn render(topic: Option<&str>) -> String {
    match topic.map(|t| t.trim().to_ascii_lowercase()) {
        None => general_help(),
        Some(topic) if topic.is_empty() => general_help(),
        Some(topic) if topic == "fetch" => fetch_help(),
        Some(topic) if topic == "links" => links_help(),
        Some(topic) if topic == "meta" => meta_help(),
        Some(topic) if topic == "json" => json_help(),
        Some(topic) if topic == "text" => text_help(),
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
        "- `pginf fetch <URL>`: fetch page, cache it, print HTTP metadata",
        "- `pginf links <URL>`: URL groups, path depth, internal/external links",
        "- `pginf meta <URL>`: curated metadata (title, lang, description, og:type, etc.)",
        "- `pginf json <URL>`: structured data (JSON-LD, Next.js, inline JSON)",
        "- `pginf text <URL>`: extracted text content",
        "- `pginf html <URL>`: raw HTML, optionally filtered by CSS selector",
        "- `pginf http <URL>`: low-level HTTP debug (request/response details)",
        "- `pginf help [topic]`: built-in guide for humans and LLMs",
        "",
        "All commands that take a URL support `--json` for machine-readable output.",
        "",
        "## Typical Workflow",
        "",
        "1. Start with `pginf fetch <URL>` to load the page into cache.",
        "2. Use `pginf links <URL>` to inspect URL structure.",
        "3. Use `pginf meta <URL>` or `pginf json <URL>` for deeper analysis.",
        "4. Use `pginf text <URL>` for content extraction.",
        "5. Use `pginf http <URL>` when fetch behavior itself needs debugging.",
        "",
        "## Cache",
        "",
        "- Pages are cached automatically in `.pginf/`.",
        "- `--refresh`: refetch and overwrite cache.",
        "- `--no-cache`: skip cache read/write.",
        "",
        "## Topics",
        "",
        "- `pginf help fetch`",
        "- `pginf help links`",
        "- `pginf help meta`",
        "- `pginf help json`",
        "- `pginf help text`",
        "- `pginf help http`",
        "- `pginf help tool`",
    ]
    .join("\n")
}

fn fetch_help() -> String {
    [
        "# `pginf fetch`",
        "",
        "Fetch a page, cache it, and print HTTP metadata.",
        "",
        "## What It Returns",
        "",
        "- input URL / final URL (after redirects)",
        "- HTTP status code",
        "- response headers",
        "- duration in ms",
        "- whether result came from cache",
        "",
        "## Examples",
        "",
        "- `pginf fetch https://example.com`",
        "- `pginf fetch https://example.com --json`",
        "- `pginf fetch https://example.com --refresh`",
        "- `pginf fetch https://example.com --no-cache`",
    ]
    .join("\n")
}

fn links_help() -> String {
    [
        "# `pginf links`",
        "",
        "Show link grouping and URL structure from a page.",
        "",
        "## What It Returns",
        "",
        "- internal links grouped by first path segment",
        "- path depth distribution",
        "- sample URLs per section",
        "- utility URLs (privacy, terms, feeds, etc.)",
        "",
        "## Flags",
        "",
        "- `--inbound`: show only internal links",
        "- `--outbound`: show only external links",
        "- `--json`: machine-readable output",
        "",
        "## Examples",
        "",
        "- `pginf links https://example.com`",
        "- `pginf links https://example.com --inbound`",
        "- `pginf links https://example.com --json`",
    ]
    .join("\n")
}

fn meta_help() -> String {
    [
        "# `pginf meta`",
        "",
        "Show curated metadata from a page.",
        "",
        "## What It Returns",
        "",
        "- title, lang",
        "- high-signal meta tags (description, robots, og:type, article:section, etc.)",
        "",
        "## Examples",
        "",
        "- `pginf meta https://example.com`",
        "- `pginf meta https://example.com --json`",
    ]
    .join("\n")
}

fn json_help() -> String {
    [
        "# `pginf json`",
        "",
        "Show structured data detected in a page.",
        "",
        "## What It Returns",
        "",
        "- JSON-LD block count and detected types",
        "- Next.js data detection",
        "- inline JSON payload detection",
        "",
        "## Examples",
        "",
        "- `pginf json https://example.com`",
        "- `pginf json https://example.com --json`",
    ]
    .join("\n")
}

fn text_help() -> String {
    [
        "# `pginf text`",
        "",
        "Extract text content from a page using dom-content-extraction.",
        "",
        "## Flags",
        "",
        "- `--format text` (default): plain text extraction",
        "- `--format markdown`: markdown-formatted extraction",
        "- `--json`: machine-readable output",
        "",
        "## Examples",
        "",
        "- `pginf text https://example.com`",
        "- `pginf text https://example.com --format markdown`",
        "- `pginf text https://example.com --json`",
    ]
    .join("\n")
}

fn http_help() -> String {
    [
        "# `pginf http`",
        "",
        "Inspect low-level HTTP behavior for one URL.",
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
        "Run `pginf fetch <URL>` first.",
        "",
        "## Command Choice",
        "",
        "- use `fetch` to load a page into cache and see HTTP metadata",
        "- use `links` for URL structure and link grouping",
        "- use `meta` for curated metadata",
        "- use `json` for structured data (JSON-LD, Next.js)",
        "- use `text` for content extraction",
        "- use `http` for request/response debugging",
        "",
        "## Output",
        "",
        "- All commands default to markdown output",
        "- Pass `--json` for machine-readable JSON",
        "",
        "## Cache",
        "",
        "- cache is enabled by default",
        "- `--refresh` forces a refetch",
        "- `--no-cache` disables cache read/write for that invocation",
        "",
        "## Caveats",
        "",
        "- cache lookup can miss if the requested URL redirects to a different final URL",
        "- the tool uses HTTP only -- JS-rendered content may be incomplete",
    ]
    .join("\n")
}

fn unknown_help(topic: &str) -> String {
    [
        format!("# Unknown Help Topic: `{topic}`"),
        "".to_string(),
        "Available topics: `fetch`, `links`, `meta`, `json`, `text`, `http`, `tool`".to_string(),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn general_help_lists_commands() {
        let help = render(None);
        assert!(help.contains("pginf fetch"));
        assert!(help.contains("pginf links"));
    }

    #[test]
    fn tool_help_mentions_fetch() {
        let help = render(Some("tool"));
        assert!(help.contains("pginf fetch"));
    }

    #[test]
    fn fetch_help_returns_content() {
        let help = render(Some("fetch"));
        assert!(help.contains("HTTP metadata"));
    }

    #[test]
    fn unknown_topic_returns_suggestions() {
        let help = render(Some("nonexistent"));
        assert!(help.contains("Unknown"));
    }
}

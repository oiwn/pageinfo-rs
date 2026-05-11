# Workflow Status: Completed Shared Outputs

This workflow spec tracks only the command surfaces that are done in the
current shared-output migration: `meta`, `links`, and `text`.

`fetch`, `json`, `html`, and any combined `inspect`/`analyze` workflow are
intentionally deferred to later releases.

## Metadata

Command:

```bash
cargo run -- meta <url> --format json
```

Current behavior:

- Uses the shared `OutputFormat` / `RenderOutput` path.
- Supports `--format text|json|toon`.
- Rejects old `--json`.
- Renders curated metadata: URL, title, lang, verbosity, and selected tags.

Observed JSON shape:

```json
{
  "url": "https://example.com/page",
  "title": "Example Page",
  "lang": "en",
  "verbosity": "main",
  "tags": [
    {
      "source": "name",
      "name": "description",
      "content": "A concise page description."
    }
  ]
}
```

## Links

Command:

```bash
cargo run -- links 'https://exodata.space/exoplanets/TOI-7009%20b' --format json
cargo run -- links 'https://exodata.space/exoplanets/TOI-7009%20b' --filter external --format toon
```

Current behavior:

- Uses the shared `OutputFormat` / `RenderOutput` path.
- Supports `--format text|json|toon`.
- Supports `--filter all|internal|external`.
- Rejects old `--json`, `--inbound`, and `--outbound`.
- Renders processed links as the primary rows.
- Preserves `raw_url` from the document and renders resolved absolute `url`.
- Keeps URL groups, depth distribution, and utility URLs as summary evidence.
- JSON and TOON render from the same JSON-shaped value.

Observed JSON shape:

```json
{
  "url": "https://exodata.space/exoplanets/TOI-7009%20b",
  "filter": "all",
  "total_internal": 7,
  "total_external": 4,
  "links": [
    {
      "raw_url": "/docs",
      "url": "https://exodata.space/docs",
      "text": "Docs",
      "rel": null,
      "is_internal": true
    }
  ],
  "groups": [
    {
      "section": "exoplanets",
      "count": 2,
      "samples": ["/exoplanets"]
    }
  ],
  "depth_distribution": [[1, 6]],
  "utility_urls": []
}
```

Observed TOON shape:

```toon
url: "https://exodata.space/exoplanets/TOI-7009%20b"
filter: external
total_external: 4
links[4]{raw_url,url,text,rel,is_internal}:
  "https://github.com/oiwn/exoplanets-catalog","https://github.com/oiwn/exoplanets-catalog",null,noopener noreferrer,false
groups[5]:
  - section: exoplanets
    count: 2
    samples[1]: /exoplanets
depth_distribution[1]:
  - [2]: 1,6
utility_urls[0]:
```

## Text

Command:

```bash
cargo run -- text https://exodata.space/docs --format json
cargo run -- text https://exodata.space/docs --format toon
```

Current behavior:

- Uses the shared `OutputFormat` / `RenderOutput` path.
- Supports `--format text|json|toon`.
- Rejects old `--json` and `--format markdown`.
- Returns structured output with `url`, `content`, and `content_length`.
- Uses `dom_content_extraction::get_content(&document)` via `PageInfo`.

Observed JSON shape:

```json
{
  "url": "https://exodata.space/docs",
  "content": "Exoplanets Catalog ...",
  "content_length": 2391
}
```

# Current task context

## Current state

The shared output migration is complete for:

- `pginf meta <url> --format text|json|toon`
- `pginf links <url> --filter all|internal|external --format text|json|toon`
- `pginf text <url> --format text|json|toon`

Old flags are intentionally rejected for migrated commands:

- `meta --json`
- `links --json`
- `links --inbound`
- `links --outbound`
- `text --json`
- `text --format markdown`

`links` now preserves raw DOM evidence (`RawLink.href` / `Link.raw_url`) and
renders processed absolute `Link.url` rows. `UrlFacts` groups/depth/utility
URLs remain summary evidence.

`text` now renders from typed `TextOutput` with:

```json
{
  "url": "https://example.com",
  "content": "...",
  "content_length": 123
}
```

Reference verification URL for text:

```bash
cargo run --quiet -- text https://exodata.space/docs --format json
cargo run --quiet -- text https://exodata.space/docs --format toon
```

## Next release work

1. Migrate `pginf fetch` to the shared output pattern.
   - Prefer `--format text|json|toon`.
   - Preserve existing fetch metadata shape: input URL, final URL, status,
     duration, cache status, headers, body size, emulation/proxy/attempts.

2. Migrate `pginf json` to typed output.
   - Replace the current summary-only JSON path with a typed command output.
   - Decide whether this release should expose parsed JSON-LD / Next.js payloads
     or only preserve the current counts/signals.

3. Decide what to do with `pginf html`.
   - Option A: keep it as a raw/debug command with selector support.
   - Option B: add shared `--format text|json|toon` rendering for selected
     elements.

4. Revisit docs and installed skill after the remaining command migrations.
   - `README.md`
   - `CHANGELOG.md`
   - `skills/pginf.md`
   - built-in help in `src/help.rs`

## Verification baseline

Run after each command migration:

```bash
cargo fmt
cargo test
```

Manual smoke checks:

```bash
cargo run --quiet -- meta https://exodata.space/docs --format json
cargo run --quiet -- links 'https://exodata.space/exoplanets/TOI-7009%20b' --format toon
cargo run --quiet -- text https://exodata.space/docs --format json
```

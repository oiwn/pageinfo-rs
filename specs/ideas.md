# Future Ideas

## `pginf install-skill` subcommand

Add a CLI subcommand that installs the LLM skill file to popular agent configs:

```
pginf install-skill --agent claude    # ~/.claude/skills/pginf/SKILL.md
pginf install-skill --agent opencode  # .opencode/skills/pginf.md
pginf install-skill --agent cursor    # .cursor/skills/pginf.md
```

Copies `skills/pginf.md` to the right location for the chosen agent.

## Need to have capability to extract text only

Remove CLI capabilities from dom-content-extraction, move it into the "pageinfo-rs",  

## Work to move into `pageinfo-rs` / `pginf`

Implement CLI extraction there using this crate as a dependency.

Suggested dependency shape:

```toml
dom-content-extraction = { version = "...", default-features = false, features = ["markdown"] }
```

### Extraction command

Add a command such as:

```bash
pginf extract -u https://example.com/article
pginf extract -u https://example.com/article --format text
pginf extract -u https://example.com/article --format markdown
pginf extract --file input.html
pginf extract --file input.html --output content.txt
```

Input rules:

- support `--url <URL>`
- support `--file <PATH>`
- require exactly one

Output rules:

- stdout by default
- `--output <PATH>` writes extracted content to a file

Format mapping:

- `text` -> `dom_content_extraction::get_content(&document)`
- `markdown` -> `DensityTree::from_document`, `calculate_density_sum`,
  `extract_content_as_markdown`

### Fetching and decoding

Use `pageinfo-rs` existing HTTP stack, not the old `dce` implementation.

`pginf` should own:

- browser/TLS emulation
- retry behavior for blocked pages
- proxy support
- timeout configuration
- redirects/final URL reporting
- response status diagnostics
- content type / charset inspection
- byte length and decoded length reporting
- non-UTF-8 decoding, including Windows-1251 style pages

### Diagnostics

Because `pginf` is an inspection tool, extraction should support diagnostics:

```bash
pginf extract -u URL --debug
pginf extract -u URL --debug-density
```

Useful diagnostics:

- fetched byte length
- decoded text length
- declared/detected encoding
- selected output format
- extracted content length
- top density nodes
- optional pollution checks for script-like markers

Some pages like interfax could be in old encoding in dom-context-extraction we used "https://docs.rs/encoding_rs/latest/encoding_rs/" for it. And chardetng
 crate

## Ability to show full html

## Render markdown

## Raname temp dir into the "pginf"

## Need to update "wreq" to 6

## Update dom-content-extraction

## Add text extraction command

## Add chardetng (detection)

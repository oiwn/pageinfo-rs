# Current task context

# Description

Add a `Headings` struct and `headings` field to `PageInfo` with 3-level verbosity filtering, following the `MetaVerbosity` pattern in `src/analyzer/meta_tag.rs`.

## Design

### Data

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Headings {
    pub h1: Vec<String>,
    pub h2: Vec<String>,
    pub h3: Vec<String>,
    pub h4: Vec<String>,
    pub h5: Vec<String>,
    pub h6: Vec<String>,
}
```

- `Headings` stored on `PageInfo` — all 6 levels extracted during `from_raw_html`
- New file: `src/analyzer/headings.rs` with `Headings`, `HeadingsOutput`, `HeadingsVerbosity`, extraction, selection, and `RenderOutput`

### Verbosity

Follows the same pattern as `MetaVerbosity` (own enum, same `parse`/`as_str` methods):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadingsVerbosity {
    Main,
    Extended,
    All,
}
```

| Level      | Levels shown |
|------------|-------------|
| `main`     | h1          |
| `extended` | h1, h2      |
| `all`      | h1–h6       |

Selection function `select_headings(headings, verbosity)` filters by level, returns a new `Headings` with empty vecs for excluded levels.

### Output

```rust
pub struct HeadingsOutput {
    pub url: String,
    pub verbosity: HeadingsVerbosity,
    pub headings: Headings,
}
```

`RenderOutput` impl renders text as `## Headings\n\n### h1\n- ...` etc (skip levels with no entries). JSON/toon follow the same pattern as `MetaOutput`.

### Integration

- `PageInfo::from_raw_html` — extract headings from parsed `Html` document
- `PageInfo::headings_output(verbosity)` — returns `HeadingsOutput`
- `PageInfo::format_for_llm` — include headings in the LLM summary
- Update `FAKE_HTML` test fixture with heading elements
- Add tests: extraction, verbosity filtering, output rendering

## Files to touch

1. **New**: `src/analyzer/headings.rs`
2. **Edit**: `src/analyzer/page_info.rs` — add `headings` field, extraction call, output method
3. **Edit**: `src/analyzer/mod.rs` or parent module — `pub mod headings`

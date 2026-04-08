## Current Direction

The project should optimize for one job:

- help an LLM research a site well enough to build or adapt a crawler

Per-site config generation is one example of that job, not the whole product.

This means the tool should not try to decide too much on its own. It should:

- fetch page data reliably
- expose structural signals clearly
- keep output compact and readable
- preserve enough raw evidence for the LLM to reason by itself

The `analyze` command should become the primary LLM-facing command first.

## Scope of This Document

This file focuses on `analyze`.

Caching is a separate component and is specified in:

- `specs/cache.md`

Detailed ideas about future output shapes can move into:

- `specs/idea.md`

## Product Principles

- Prefer evidence over strong built-in heuristics.
- Keep output compact and LLM-readable.
- Make commands granular so an LLM can ask follow-up questions with tools.
- Separate raw data collection from presentation.
- Keep enough raw content and page structure for the LLM to inspect directly.

## Analyze Command Goal

`analyze` should stop behaving like a mixed debug dump and become a focused page research tool.

The main job of `analyze` is to expose:

- what kinds of internal URLs exist on the page
- what kinds of content the page appears to link to
- what structured data exists in the page source
- what page-level metadata may help crawler construction

## Desired Output Characteristics

- concise markdown
- stable ordering
- deduplicated URLs where possible
- enough raw evidence for the LLM to apply its own heuristics
- easy to inspect manually when needed

## Keep

- extracted content
- URL samples
- raw structural evidence useful for reasoning

The extracted content should stay available because the LLM may need it to distinguish article pages from service pages or utility pages.

## Remove or Reduce

- noisy meta tags
- duplicated URLs
- presentation that mixes debugging detail with crawler-relevant information

## Add or Improve

- better URL deduplication and normalization
- clearer grouping of internal URLs
- visibility into embedded JSON and structured data blobs
- feed detection
- better coverage of URL shapes on the page

## Working Direction for `analyze`

The tool should lean toward collecting and presenting evidence, not overfitting heuristics.

Examples of useful evidence:

- grouped internal URLs
- path depth distribution
- repeated path shapes
- query parameter examples
- anchor text samples
- curated metadata
- JSON-LD or other embedded structured data
- detected feed URLs
- extracted content

Examples of things the LLM can infer from that evidence:

- article URL patterns
- likely editorial sections
- likely blacklist targets
- regexes or templates for crawler rules

## Staged Plan

### Stage 1: Clean Up `analyze`

Scope: improve the current `analyze` command without redesigning the whole CLI.

Tasks:

1. Deduplicate URLs consistently.
   Use URL-aware normalization where possible.

2. Keep extracted content, but ensure it is presented intentionally rather than as a noisy dump.

3. Filter meta tags down to high-signal fields.

4. Detect and surface feed-like URLs explicitly.

5. Improve URL grouping so the report gives better coverage of page structure.

6. Expose embedded JSON and structured data in the page source.

The focus of this stage is better evidence, not more opinionated inference.

### Stage 2: Split Analysis Into More Granular Commands or Views

The current CLI is still too coarse for tool-driven LLM usage.

We likely need narrower ways to inspect the same page without rerunning one large report.

Candidate directions:

- `analyze page -u <URL>`
  General page summary

- `analyze links -u <URL>`
  Internal URL inventory grouped by shape, section, or depth

- `analyze metadata -u <URL>`
  Curated metadata and page-level signals

- `analyze json -u <URL>`
  JSON-LD, hydration blobs, embedded application state, and other structured payloads

Exact CLI shape is still open. This may become subcommands, flags, or alternate views.

### Stage 3: Use Cache Through `analyze`

Cache design lives in `specs/cache.md`.

For `analyze`, the intended behavior is:

- use local cache by default
- allow refresh when needed
- keep cached pages inspectable by both the user and the LLM

At this stage the cache only needs to support `analyze`.

### Stage 4: Revisit Sampling Later

Sampling is not the priority right now.

The current direction is to make `analyze` good enough that an LLM can do follow-up reasoning itself. We can revisit `sample` later if real usage shows that one-page evidence is insufficient.

## Notes on Heuristics

Heuristics should support the LLM, not replace it.

Good heuristics:

- deduplication
- stable grouping
- feed detection
- extraction of structured data types
- URL bucketing for better coverage

More aggressive heuristics should be optional or deferred when the LLM can infer the answer from presented evidence.

If bucketing or similarity clustering becomes important, using a string-similarity crate may help group related URL shapes.

## Immediate Implementation Order

Short-term order for code changes:

1. refactor `analyze` output shape
2. deduplicate and normalize URLs better
3. filter metadata
4. expose feeds explicitly
5. expose embedded JSON / structured data
6. improve URL grouping / bucketing
7. wire `analyze` into cache

## Open Questions

- Should `analyze` stay markdown-only for now, or later also expose machine-readable JSON?
  Current direction: markdown is enough for now.

- Should the tool generate regex candidates itself?
  Current direction: no, prefer exposing evidence so the LLM can generate them.

- Should cache be used only by `analyze` initially?
  Current direction: yes.

- Should granularity be implemented as subcommands or alternate views?
  Still open.

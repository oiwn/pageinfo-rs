# Current Task Context

## Task

The next task is to make `analyze` a much better LLM-facing research tool.

This task is not about broad architecture anymore. Cache V1 is already in place. The focus now is the quality and structure of `analyze` output.

## Goal

`analyze` should help an LLM inspect a page and answer questions such as:

- what kinds of internal URLs are present
- what kinds of content the page links to
- what page-level metadata is useful
- whether the page contains structured data or embedded JSON
- what feed-like URLs exist
- what is text content on the page

The command should provide evidence in a form that helps the LLM reason, instead of trying to solve crawler design fully by itself.

## Boundaries

In scope:

- improve `analyze` output
- improve URL deduplication and grouping
- reduce noisy metadata
- expose feed-like URLs
- expose embedded JSON / structured data
- keep extracted content available

Out of scope for this task:

- changing cache design
- redesigning the whole CLI
- reviving or extending `sample`
- generating crawler configs directly
- adding strong site-specific heuristics

## Current Problems

The current `analyze` output still has several problems:

- too much mixed presentation, not enough structure
- duplicated URLs
- meta tag output is noisy
- feeds are not called out explicitly
- embedded JSON is not surfaced
- URL grouping is useful but still shallow
- extracted content is present, but not intentionally framed

Current direction:

- treat the current `analyze` implementation as something to rebuild, not polish lightly
- keep only the parts that are actually useful
- do not preserve existing output shape just for continuity

## Target Output Shape

The next version of `analyze` should remain markdown-based, but the sections should be more intentional.

Proposed output shape:

1. Header
   - URL
   - final URL
   - status
   - domain
   - title
   - lang
   - include `lang` only if it exists as an actual page signal

2. Summary
   - internal link count
   - external link count
   - number of distinct first-level sections
   - whether feeds were detected
   - whether embedded structured data was detected

3. Curated Metadata
   - only high-signal meta tags

4. URL Groups
   - grouped internal URLs by first path segment
   - deduplicated sample URLs per group
   - path depth summary

5. Feeds
   - explicit list of feed-like URLs if present

6. Structured Data
   - JSON-LD presence
   - other embedded JSON/blob signals

7. Extracted Content
   - keep available
   - present as supporting evidence, not as the main report

This is still a report, not a config generator.

## Command Set Direction

The tool should move toward a small set of agent-friendly commands instead of one overloaded report.

Current direction:

### 1. `analyze`

Primary page research command.

Purpose:

- fetch or load one page
- present a structured markdown report
- act as the default command an LLM uses first

Short-term scope:

- header
- summary
- curated metadata
- URL groups
- feeds
- structured-data summary
- full extracted content

### 2. `http`

Low-level HTTP debug command.

Purpose:

- inspect raw request/response behavior
- debug fetch issues

This command is for transport/debugging, not normal crawler research.

### 3. `help`

Integrated help command designed for both humans and LLMs.

Purpose:

- explain what commands exist
- explain what each command is for
- explain what kind of evidence each command returns
- show example invocations
- help an LLM choose the right next command

This should be more detailed than default Clap help.

Recommended forms:

- `pginf help`
- `pginf help analyze`
- `pginf help http`
- `pginf help tool`

### 4. Future command candidates

These are not part of the immediate implementation, but they are likely useful:

- `analyze links`
- `analyze meta`
- `analyze json`
- `cache show`
- `cache delete`

Current direction:

- do not implement all of them now
- design `help` so it can describe them once they exist

## Integrated Help for LLM Use

The tool should have a help mode that reads like tool documentation, not only CLI syntax.

The goal is to let an LLM call one command and understand:

- what the tool does
- what commands exist
- what output each command returns
- when to use each command
- what the main workflow is

### Help Content Requirements

The integrated help should include:

1. Tool purpose
   - explain that the tool helps research web pages for crawler building/adaptation

2. Command catalog
   - list commands
   - short purpose for each

3. Typical workflow
   - start with `analyze`
   - use `http` for fetch/debug problems
   - use cache flags when needed

4. Output expectations
   - `analyze` returns structured markdown evidence
   - `http` returns transport-level details

5. Flag explanations
   - especially `--no-cache` and `--refresh`

6. Example invocations
   - short, copyable examples

### `help tool`

One dedicated help view should be designed specifically for agents.

Suggested purpose:

- act as a compact built-in manual for LLM/tool use

Suggested content:

- tool summary
- recommended first command
- command descriptions
- expected outputs
- cache behavior
- caveats

This should be plain markdown and easy to paste into an LLM context window.

## Detailed Requirements

### 1. URL Deduplication

`analyze` should deduplicate URLs more reliably before building facts and samples.

Requirements:

- remove exact duplicates
- normalize URLs conservatively
- avoid duplicate display in utility/sample sections
- keep URL semantics intact

Use the `url` crate for normalization where possible.

### 2. Meta Tag Filtering

Meta output should become curated instead of dumping everything.

Keep only tags likely to matter for crawler reasoning, such as:

- description
- robots
- og:type
- content language indicators
- page category hints
- canonical-related signals if exposed via metadata

Drop low-signal tags such as:

- numeric IDs
- image dimensions
- app-specific rendering flags
- generic social noise that does not help crawling

The keep-list should be explicit and easy to extend.

### 3. Feed Detection

`analyze` should surface feed-like URLs explicitly instead of leaving them mixed inside generic utility URLs.

Detection should remain simple in this task.

Examples:

- URLs containing `rss`
- URLs containing `feed`
- Atom-like feed endpoints if obvious

Output should show a dedicated `Feeds` section when matches exist.

### 4. Embedded JSON / Structured Data

`analyze` should detect whether the page contains embedded structured data.

Minimum coverage:

- `script[type="application/ld+json"]`
- obvious JSON hydration blobs such as framework bootstrap data
- large inline JSON-like script blocks worth signaling

For this task, it is enough to report presence and short summaries.

Examples of output:

- JSON-LD found
- Next.js data blob found
- embedded JSON script found

Do not try to fully model every schema yet.

### 5. URL Grouping

The section grouping already exists, but this task should make it more useful.

Requirements:

- deduplicated samples per group
- clearer ordering
- stable output
- better coverage of repeated URL shapes

This can stay based on first path segment for now, but implementation should avoid obvious noise.

### 6. Extracted Content

Extracted content stays.

The task is not to remove it, but to make the whole report read as structured evidence instead of ending in a giant dump with no framing.

Current direction:

- extracted content should stay full
- the LLM on the other side can analyze it
- do not truncate it in this task

## Implementation Approach

The work should be done inside existing `analyze` structures first.

Likely touch points:

- `src/analyzer/page_info.rs`
- `src/analyzer/url_facts.rs`
- `src/analyzer/link.rs`

Possible small supporting additions:

- helper functions for URL normalization/dedup
- helper functions for feed detection
- helper functions for meta filtering
- helper functions for JSON blob detection

Do not redesign the full module layout unless needed.

## Deliverables

This task should produce:

1. cleaner `analyze` markdown output
2. deduplicated URL presentation
3. filtered metadata output
4. explicit feed detection section
5. structured-data detection section
6. integrated help design and implementation plan
7. tests for the new logic where practical

## Testing Requirements

Since Rust code will change, the implementation must finish with:

- `cargo fmt`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`

Add tests where they give real value, especially for:

- URL deduplication behavior
- feed detection logic
- meta filtering logic
- structured-data detection helpers

## Non-Goals

This task should not:

- generate regexes for the user
- decide editorial vs non-editorial sections aggressively
- create per-site configs automatically
- add a new cache layer for derived data

The point is to improve evidence quality, not to over-automate conclusions.

## Suggested Order

Recommended implementation order:

1. define final command/help shape
2. clean up output structure in `page_info`
3. add URL deduplication improvements
4. add metadata filtering
5. add feed detection
6. add structured-data detection
7. add or update tests

## Open Questions

- Should structured-data output be just presence/summary, or include small samples?
  Current direction: presence/summary first.

- Should extracted content be truncated in the report?
  Current direction: no, keep it full.

- Should feed detection stay heuristic-only for now?
  Current direction: yes.

- Should integrated help be plain Clap help only?
  Current direction: no, add a richer built-in help view for LLM use.

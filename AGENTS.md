# AGENTS.md

Project architecture is in `specs/overview.md`. Read it first.

## Working on this project

- Run `cargo fmt` after finishing any task that changes Rust code.
- `cargo clippy --all-targets -- -D warnings` must pass when Rust code changed — fix before stopping.
- `cargo test` after every change to Rust code.
- Skip cargo commands if no `.rs` or `Cargo.toml` files were modified.
- Keep responses short — fit a single screen. No walls of text.
- View file before editing. Match whitespace exactly.
- No comments in code unless asked.

## Behavioral Guidelines

### Think Before Coding

- State assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop and ask before implementing.

### Simplicity First

- No features beyond what was asked. No speculative abstractions.
- No "flexibility" or "configurability" that wasn't requested.
- If you write 200 lines and it could be 50, rewrite it.

### Surgical Changes

- Touch only what you must. Don't refactor adjacent code unprompted.
- Match existing style, even if you'd do it differently.
- Remove only the dead code your own changes created.
- Every changed line should trace to the user's request.

### Goal-Driven Execution

- Transform tasks into verifiable goals. State a brief plan for multi-step work.
- Define success criteria upfront: "Write a test that reproduces it, then make it pass."
- Loop independently until criteria are met.

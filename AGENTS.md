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

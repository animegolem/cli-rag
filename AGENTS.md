# Repository Guidelines

## Project Structure & Module Organization
- `src/bin/adr-rag.rs`: Thin binary wiring CLI to library.
- `src/cli.rs` and `src/commands/*`: Clap definitions and subcommand handlers.
- `src/{config,model,discovery,index,validate,graph,watch,util}.rs`: Core modules (config I/O, front matter parsing, scanning/indexing, validation, graph ops, watcher, helpers).
- Config file: `.adr-rag.toml` at repo root (or discovered upward).
- Build artifacts in `target/`; temporary notes in `tmp/`.

## Build, Test, and Development Commands
- With Cargo:
  - `cargo build` | `cargo build --release`
  - `cargo run --bin adr-rag -- <subcommand>`
  - `cargo test`
- Shell completions: `adr-rag completions bash|zsh|fish` (e.g., `adr-rag completions zsh > ~/.zsh/completions/_adr-rag`).

## Coding Style & Naming Conventions
- Rust edition 2021; 4‑space indentation; wrap at ~100 cols.
- Names: modules/functions `snake_case`, types/traits `CamelCase`, constants `SCREAMING_SNAKE_CASE`.
- Prefer small, focused modules under `src/commands/` for each subcommand.
- Always run `cargo fmt` and `cargo clippy` before pushing.

## Testing Guidelines
- Unit tests live inline using `mod tests { ... }` within modules (see `model.rs`, `validate.rs`, `commands/graph.rs`).
- Name tests descriptively; assert on exact messages where practical.
- Add tests next to changed code and cover common error paths.
- Run the full suite: `cargo test`.

## Commit & Pull Request Guidelines
- Commits: imperative mood, concise subject; include a focused body when changing behavior (why + brief what).
- Reference issues with `Fixes #123` when applicable.
- PRs must include: summary, rationale, notable tradeoffs, and a short demo (e.g., command and output snippet) for CLI changes.
- CI hygiene: ensure `cargo fmt`, `cargo clippy`, and `cargo test` pass locally.

## Security & Configuration Tips
- Do not commit machine‑specific paths in `.adr-rag.toml`; prefer relative `bases`, `index_relative`, and `groups_relative`.
- Validate before relying on indexes: `adr-rag validate --format json`.
- Use `--config` or `ADR_RAG_CONFIG` to test alternate configs without editing the default.

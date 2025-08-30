# Repository Guidelines (cli-rag)

## Project Structure & Module Organization
- `src/bin/cli-rag.rs`: Thin binary wiring CLI to library.
- `src/cli.rs` and `src/commands/*`: Clap definitions and subcommand handlers.
- `src/{config,model,discovery,index,validate,graph,watch,util}.rs`: Core modules (config I/O, front matter parsing, scanning/indexing, validation, graph ops, watcher, helpers).
- Config file: `.cli-rag.toml` at repo root (or discovered upward). Legacy `.adr-rag.toml` is no longer used.
- Build artifacts in `target/`; temporary notes in `tmp/`.

## Build, Test, and Development Commands
- With Cargo:
  - `cargo build` | `cargo build --release`
  - `cargo run --bin cli-rag -- <subcommand>`
  - `cargo test`
- Shell completions: `cli-rag completions bash|zsh|fish` (e.g., `cli-rag completions zsh > ~/.zsh/completions/_cli-rag`).
- Justfile helpers:
  - `just run <args>` → runs `cargo run --bin cli-rag -- <args>`
  - `just fmt`, `just fmt-check`, `just lint`, `just test`
  - `just precommit` → local pre-CI checks (fmt, clippy, line-length guard)

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
- Do not commit machine‑specific paths in `.cli-rag.toml`; prefer relative `filepaths` (alias `bases`), `index_relative`, and `groups_relative`.
- Validate before relying on indexes: `cli-rag validate --format json`.
- Use `--config` or `CLI_RAG_CONFIG` to test alternate configs without editing the default.

## Pre‑CI Expectations (Local)
- Before committing, ensure the following pass locally:
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - Optional: `cargo test` (full suite) if you changed logic
  - Line‑length guard: no Rust source file in `src/` (excluding tests) exceeds 350 lines

### Optional: Git pre‑commit hook
- We provide a local pre‑commit hook at `.githooks/pre-commit`.
- Enable once per repo: `git config core.hooksPath .githooks`
- The hook runs `cargo fmt --check`, `cargo clippy -D warnings`, and the line‑length guard script.
- To skip locally: commit with `--no-verify` (not recommended).

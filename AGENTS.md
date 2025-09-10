# Repository Guidelines (cli-rag)

## Contracts‑First Development
- Authoritative interfaces live in `contracts/`.
- JSON outputs use camelCase and MUST validate against `contracts/cli/*.schema.json`.
- TOML and Lua use snake_case; ResolvedConfig emitted to JSON must match `contracts/v1/resolved_config.json`.
- Prefer updating contracts over ADR docs; ADRs may drift and are not the source of truth.

## Refactor Boldly (Alpha)
- This project is alpha/non‑prod with no users. Do not be conservative during refactors.
- Remove dead code rather than leaving TODO hooks for later; keep surfaces aligned to `contracts/`.
- Prioritize matching schemas, exit codes, ordering, and NDJSON conventions defined in `contracts/global-conventions.md`.

## Deprecation Markers
- Use the exact token `_DEPRECATED` to tag anything slated for removal.
- Placement:
  - Put a preceding line comment on the declaration to be removed (fn/struct/impl/module/const/flag): `// _DEPRECATED: <reason> | <removal-phase> | <ticket-id>`.
  - For files, add a top‑of‑file comment with the same token.
  - For CLI flags/help, add `(DEPRECATED)` in the help text and keep a code comment with `_DEPRECATED`.
- Examples:
  - `// _DEPRECATED: legacy groups writer | Phase 2 | AI-IMP-001`
  - `// _DEPRECATED: old JSON shape | Phase 1 cleanup | AI-IMP-001`
- Policy:
  - Prefer `_DEPRECATED` over ad‑hoc TODOs for removals, so `rg -n "_DEPRECATED"` yields a complete removal list.
  - PRs that remove deprecated code should reference the marker(s) and the Bridge Plan phase.

## Project Structure & Module Organization
- `src/bin/cli-rag.rs`: Thin binary wiring CLI to library.
- `src/cli.rs` and `src/commands/*`: Clap definitions and subcommand handlers.
- `src/{config,model,discovery,index,validate,graph,watch,util}.rs`: Core modules (config I/O, front matter parsing, scanning/indexing, validation, graph ops, watcher, helpers).
- Config file: `.cli-rag.toml` at repo root (or discovered upward). Legacy `.adr-rag.toml` is no longer used.
- Build artifacts in `target/`; temporary notes in `tmp/`.

## Tooling for shell interactions
- Is it about finding FILES? use 'fd'
- Is it about finding TEXT/strings? use 'rg'
- Is it about finding CODE STRUCTURE? use 'ast-grep'
- Is it about SELECTING from multiple results? pipe to 'fzf'
- Is it about interacting with JSON? use 'jq'
- Is it about interacting with YAML or XML? use 'yq'

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

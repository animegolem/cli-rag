# Repository Guidelines (cli-rag)

## Scope & Precedence
- This AGENTS.md applies to the whole repo. Nested AGENTS.md files apply to their subtrees and take precedence for files they cover.

## Contracts‑First Development
- Authoritative interfaces live in `contracts/`.
- JSON outputs use camelCase and MUST validate against `contracts/v1/**/*.schema.json` (e.g., `contracts/v1/index/index.schema.json`, `contracts/v1/config/resolved_config.json`, and CLI responses under `contracts/v1/cli/`).
- TOML and Lua use snake_case; ResolvedConfig emitted to JSON must match `contracts/v1/resolved_config.json`.
- Prefer updating contracts over ADR docs; ADRs may drift and are not the source of truth.

## Refactor Boldly (Alpha)
- This project is alpha/non‑prod with no users. Do not be conservative during refactors.
- Remove dead code rather than leaving TODO hooks for later; keep surfaces aligned to `contracts/`.
- Prioritize matching schemas, exit codes, ordering, and NDJSON conventions defined in `contracts/global-conventions.md`.

## Stop Points & Change Control
- Any change to contracts (schemas under `contracts/` or `contracts/global-conventions.md`) requires prior discussion and a logged entry in `contracts/changelog.md` with rationale and impact.
- Coordinate before changing exit codes, NDJSON handshake/event order, or protocol version signaling fields.

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
- Assistants must never amend commits or rewrite published history; always add new commits instead.
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

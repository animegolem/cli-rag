# adr-rag Refactor Checklist

This checklist tracks the stepwise decomposition of `src/main.rs` into cohesive modules and command handlers. Mark items as you complete them.

## Prep
- [x] Write refactor plan and checklist next to DEVLOG (`refactor-checklist.md`).
- [x] Create `src/lib.rs` scaffold (exports, `run` entry placeholder).

## Phase 1 — Core Types & Graph
- [x] Extract `model.rs`: `FrontMatter`, `AdrDoc`, `parse_front_matter_and_title`, `file_mtime`, `file_size`.
- [x] Extract `graph.rs`: `bfs_path`, `compute_cluster` (+ small helpers), pure functions only.
- [x] Extract `config.rs`: `Config` + defaults, `find_config_upwards`, `load_config`, `write_template`.
- [x] Update imports and ensure `cargo build` passes.

## Phase 2 — Discovery & Index IO
- [x] Extract `discovery.rs`: `scan_docs`, `scan_docs_in_base`, `load_docs_from_index`, `load_docs`, `incremental_collect_docs`.
- [x] Extract `index.rs`: `write_indexes`, `write_groups_config`.
- [x] Centralize schema globset compilation (shared utility/state if needed).
- [x] Build and quick-run affected commands (`doctor`, `topics`).

## Phase 3 — Validation
- [x] Extract `validate.rs`: `SchemaCfg`, `SchemaRule`, and `validate_docs`.
- [x] Replace tuple return with `ValidationReport { ok, errors, warnings, doc_count, id_count }` (internal only; keep CLI output the same).
- [x] Move/expand unit tests for validation into module tests.

## Phase 4 — Utilities & Watcher
- [x] Extract `util.rs`: `try_open_editor`, small IO helpers, constants (consider moving `TEMPLATE` here).
- [x] Extract `watch.rs`: notify watcher setup, debounce, incremental validate/index write.
- [x] Keep `watch` logic orchestrational; reuse discovery/index/validate.

## Phase 5 — CLI Split & Output
- [x] Extract `cli.rs`: `Cli`, `Commands`, `ValidateArgs`, completions plumbing.
- [x] Create `src/commands/` and move handlers:
  - [x] `init.rs`
  - [x] `doctor.rs`
  - [x] `search.rs`
  - [x] `topics.rs`
  - [x] `group.rs`
  - [x] `get.rs`
  - [x] `cluster.rs`
  - [x] `path.rs`
  - [x] `validate_cmd.rs`
  - [x] `watch_cmd.rs`
  - [x] `completions.rs`
- [x] Wire `main.rs` to call command handlers and use `cli.rs` types.
- [x] Add small formatting helpers for plain/json output to avoid drift.

## Phase 6 — Main Thin Entry
- [x] Reduce `main.rs` to parse args and delegate to modular command handlers.
- [x] Move entry to `src/bin/adr-rag.rs`; keep logic in `src/lib.rs`.

## Phase 7 — Tests, Build, Docs
- [x] Ensure all existing tests pass; relocate module tests next to code.
- [x] `cargo build` and spot-check core commands.
- [x] Update `tools/adr-rag/README.md` with new structure.
- [ ] Add a short note in `DEVLOG.md` summarizing the refactor.

## Optional Improvements (post-refactor)
- [ ] Introduce `DocIndex` cache (id map, dependents map, compiled globsets).
- [ ] Add `note_type` to `AdrDoc` (computed once).
- [ ] Add `tracing` for watch/discovery debug logging (off by default).

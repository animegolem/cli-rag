# cli-rag Bridge: ADRs → Codebase Checklist

This file captures a technical requirements bridge between the ADRs in `docs/ADR/` and the current Rust implementation under `src/`. It identifies features, current status, concrete gaps, and a proposed implementation order. Use it to drive scoped tickets and verify progress via `cargo test` and CLI demos.

## Scope & Method
- Sources: ADRs (including AI ADRs) and `docs/ADR/repomix-output.xml` vs. the code in `src/`.
- Focus: v1/v1.1 CLI surface, config behavior, validation, graph/retrieval, indexing/watch, and foundations for GTD/TUI/Lua/AI.

## High-Level Feature Map

### CLI Surface (ADR-003b, ADR-003c)
- Present: `init`, `doctor`, `search`, `topics`, `group`, `get`, `cluster`, `graph`, `path`, `validate`, `watch`, `completions`.
- Gaps:
  - [ ] `new` command (templating, id generation, `--edit`, `--dry-run`, `--print-body`).
  - [ ] `get --format md|ai|json` (partial: `ai` format added; no `md` variant yet).
  - [ ] `get --depth N --include-bidirectional` (current: only immediate deps; `include_dependents` boolean).
  - [ ] `graph --graph-format ascii` and `--root none` (current: mermaid/dot/json only; requires id).
  - [ ] `search` filters (`--schema`, `--recent`, `--field`, `--kanban`, `--TODO`).
  - [ ] `status` command (aka improved `doctor`) with table output, color, `--json`, `--verbose`.
  - [ ] Cross-cutting: global `--json`/`--ndjson` coherence; align flags/names per ADR-003b/003c (partial: `watch --json` added; `doctor --json` includes capabilities; formats not fully unified).

### Config & Loader (ADR-001, ADR-004, ADR-006)
- Present: `.cli-rag.toml` load/discover; `bases` (alias: `filepaths`), `index_relative`, `groups_relative`, `file_patterns`, `ignore_globs`, `allowed_statuses`, defaults; `[[schema]]` with `required`, `unknown_policy`, `allowed_keys`, keyed `rules`.
- Gaps:
  - [ ] `config_version` and versioned parsing/upgrade path.
  - [x] Single top-level config enforcement with clear codes (E100) and unique schema names (E120).
  - [x] `import = [...]` for external schema files and enforcement that imports contain schemas only (E110).
  - [ ] Consistent env/flag naming and docs (`ADR_RAG_*` vs ADR naming); clarify `.cli-rag.toml` in docs (code uses `.cli-rag.toml` and `CLI_RAG_*`).
  - [ ] Template scaffolds in `init --schema` (`--separate` option per ADR-003c).
  - [ ] Authoring knobs: `editor` default and `background_watch = true` wiring.
  - [ ] Index model: single index per repo (collapse groups + file index) and disallow multiple indexes; define final on-disk path (partial: unified index writer/reader present; per-base still written).

### Validation Engine (ADR-006, ADR-007, ADR-AI-003)
- Present: id required; global status allowlist (overridden by schema rule); duplicate/conflict detection; `depends_on`/`supersedes`/`superseded_by` existence; schema required/unknown/allowed-keys; rule types (array/date), `min_items`, `regex`; `refers_to_types` basic check.
- Gaps:
  - [ ] Uniform machine-readable error codes and shapes (E2xx/E24x etc.) wired through `validate --json|--ndjson` (partial: initial codes now emitted for common cases; stabilize and document numbering).
  - [x] File matches multiple schemas → error (priority/first-match policy) (E200).
  - [ ] Cycle detection options per schema (warn/error) and DAG policy (partial: basic cycle detection emits warnings; no per-schema policy yet).
  - [ ] Extensible graph edges: classify `graph_edges = [...]` that auto-validate as id refs (ADR-AI-003); not just hardcoded keys.
  - [ ] Filename uniqueness across graph when id generator used (ADR-001 note).
  - [ ] Better isolated/orphan analysis surfaced in `doctor/status` (currently only warning text from validation; formalize).
  - [ ] Frontmatter policy: `allow_unknown` default + per-field rules (regex/range), e.g., `tags` regex, `related_files` extensions.
  - [ ] Body policy: `heading_check` (exact|missing_only|ignore) and `line_count.scan_policy` (on_creation|on_validate) driven by `{{LOC|N}}`.
  - [ ] Edges policy: `required_edges`, `detect_cycles`, `wikilinks.{min_outgoing,min_incoming,severity}`, `cross_schema.allowed_targets`.

### Graph & Retrieval (ADR-003b/003c)
- Present: BFS `path` with bidirectional traversal; `cluster` computation; `graph` render mermaid/dot/json; `get` prints content and immediate deps; `groups/topics` summary tools.
- Gaps:
  - [ ] `path` output with edge kinds and line refs when mention-derived (partial: line refs/locations included; edge kinds not yet).
  - [ ] ASCII graph rendering (graphviz ASCII-like) and optional navigation affordances later; `graph --graph-format ascii`.
  - [ ] Graph traversal over all configured `graph_edges`, not just `depends_on`.
  - [ ] `get --depth` neighborhood export with `ai`/compact JSON for LLMs (partial: `ai` format implemented; no depth yet).

### Indexing & Watch & Cache (ADR-AI-001)
- Present: JSON index writer per base; groups writer; `watch` with debounce, incremental re-parse by `mtime/size`.
- Gaps:
  - [ ] Three-layer cache (in-mem session, on-disk rebuildable, source-of-truth in notes) with simple file lock.
  - [ ] `validate --rebuild-cache` hook to force recomputation of expensive metadata.
  - [x] `watch --json` NDJSON stream of events (for NVIM/TUI) with event kinds (`validated`, `index_written`, `groups_written`); option to write groups during watch.
  - [ ] Adopt “single index per repo”: unify current per-base writes; derive groups view from index (partial: unified index written at config root; readers prefer unified; per-base remains for compatibility).

### Templates, Parsing, and `new` (ADR-001, ADR-011, ADR-010)
- Present: None in code yet; config template includes examples.
- Gaps:
  - [ ] Minimal templating engine: support tokens `{{id}}`, `{{title}}`, `((frontmatter))`, `{{LOC|N}}`, `{{date}}`, `{{time}}`.
  - [ ] Id generator + filename template + variable precedence policy (e.g., `["system","frontmatter","computed"]`).
  - [ ] Limited frontmatter rendering and merging rules.
  - [ ] Lua escape hatch (validate/generate hooks) with sandboxed, deterministic API surfaces (spans; read-only note/frontmatter/headings/links; engine-provided `now`/`rand`/KV store); defer IO to engine.
  - [ ] Consider migration toward a Markdown AST (markdown-rs/oxide) for robust spans and inline token detection.
  - [ ] Title case normalization for generated titles (e.g., via `heck`/`convert_case`).

### GTD / TODO / Kanban (ADR-009, ADR-AI-002)
- Present: None.
- Gaps:
  - [ ] Parse inline TODO syntax like `TODO@HIGH: -[] task` and/or `managed_frontmatter` for GTD fields.
  - [ ] `search --TODO` and `search --kanban` filters; agenda list output (with completed retention window, default 3 days).
  - [ ] Kanban frontmatter keys: `kanban_status`, `due_date`, optional `kanban_statusline`; `due_date_warning` days.
  - [ ] LSP-aligned checkbox updates; color mapping by priority in NVIM.
  - [ ] Use cache to track completion timestamps and volatile agenda state (avoid unnecessary file churn).

### NVIM/TUI Integration (ADR-002, ADR-011)
- Present: None.
- Gaps:
  - [ ] Provide stable `--json`/`--ndjson` outputs, spans where applicable, to drive NVIM integration (partial: JSON/NDJSON surfaces exist for search/topics/group/validate/doctor; `watch --json` added; basic locations for path/validate).
  - [ ] Plan for `nvim-oxi` plugin scaffolding consuming `watch --json`, `get --format json|ai`, `validate --json`; scenes: Agenda, Kanban, Vault (templates/notes), Graph nav.
  - [ ] Consider minibuffer-like quick edits or open-in-editor workflow; fuzzy finder over tracked notes.

### MCP Wrapper (ADR-005)
- Present: None.
- Gaps:
  - [ ] After CLI stabilizes, thin MCP server wrapper exposing NDJSON outputs as tools.

### Documentation (ADR-012, ADR-013)
- Present: ADRs authored; no man/tldr or notebook docs wired.
- Gaps:
  - [ ] Man pages via pandoc; `tldr` entries; nicer notebook docs near v1.0.

## Proposed Implementation Order (Phased)

1) Config & Validation Foundations
   - [x] Loader invariants: single config, unique schemas, imports-only schemas (E100/E110/E120).
   - [ ] `config_version` + gentle warnings; `init --schema` scaffolds.
   - [ ] Validation codes + multiple schema match error; cycle policy; validator knobs for frontmatter/body/edges (rules/unknown policy exist; codes/cycle/multi-match pending).
   - [ ] Decide and implement single-index model and path (partial: unified index implemented; per-base still written).

2) CLI Surface Lock-in (v1)
   - [ ] Normalize global `--format` across commands; keep `graph`’s own `--graph-format`.
   - [ ] `status` (doctor++) with table output and `--json`.
   - [ ] `search --schema|--recent` and `group --list` polish.

3) Graph & Retrieval Improvements
   - [ ] `get --format ai|json` + `--depth` + `--include-bidirectional` (partial: `ai` done).
   - [ ] `path` edge-kinds; `graph` ASCII option (partial: `path` line refs done).
   - [ ] Generalize traversal over `graph_edges`.

4) Index/Watch/Cache
   - [x] NDJSON watch stream (`{"event":"validated"|"index_written"|...}`) and optional groups write.
   - [ ] Disk cache layer and simple file lock; `--rebuild-cache` plumbing; background watch integration from config.

5) Templates and `new`
   - [ ] Minimal templating + id generation + frontmatter injection.
   - [ ] `new --dry-run|--print-body|--edit` flow.

6) GTD/TODO (baseline)
   - [ ] Inline TODO extraction; `search --TODO`; basic agenda view.

7) Lua escape hatch (limited)
   - [ ] Read-only query API, validate hooks; generation hooks opt-in.

8) NVIM/MCP & Docs (post-v1 polish)
   - [ ] NVIM plugin scaffolding fed by JSON surfaces; MCP wrapper after CLI stabilizes.
   - [ ] Man/tldr + notebooks.

## Per-ADR Worklists (Actionable)

### ADR-001 cli-rag.toml
- [ ] Add `config_version` support and surfaced warnings for deprecated versions.
- [ ] Implement `[[schema]]`-driven `new` with id/filename templates.
- [ ] Enforce filename uniqueness when id generator is active.
- [x] Support `import = [".adr-rag/templates/*.toml"]` for schemas.
 - [ ] Add validator knobs: frontmatter allow_unknown + per-field rules; body heading/LOC policies; edges required/min_in|min_out/cross-schema/cycle.
 - [ ] Authoring: `editor` default and `background_watch` wiring.
 - [ ] Adopt single index per repo and collapse groups view into index (partial: unified index present; per-base still written).

### ADR-002 Visual Mode (NVIM/TUI)
- [ ] Provide NDJSON event stream and JSON retrieval to power Magit/org-like UI.
 - [ ] Optional ASCII graph for navigation and simple graph navigation affordances.
 - [ ] Fuzzy finder, collapsible schema sections, agenda + kanban panes.

### ADR-003a/003b/003c CLI
- [ ] Lock verbs/flags; add `new`, `status`, `get` formats, `search` filters.
- [ ] Resolve `--format` collision by using `--graph-format` for graph output type.
- [ ] Add `ai draft` subcommands (v1.1) behind a feature flag after templates exist.

### ADR-004 Config Versioning
- [ ] Versioned parsing; `cli-rag config upgrade` helper (scaffold).

### ADR-005 MCP
- [ ] Thin MCP server mirroring CLI tools (post CLI lock-in).

### ADR-006 Config Loader Errors
- [ ] E100/E110/E120 codes; clear messages; JSON emission via `validate --ndjson` and `status --json`.

### ADR-007 General Error Codes
- [ ] Unify error numbering and shape; include `{severity, code, msg, span?, field?}` (partial: initial codes now emitted in `validate --json|--ndjson`; finalize table and propagation).

### ADR-008 AI RAG
- [x] Defer advanced query DSL; start with `get --format ai` (cache deferred).

### ADR-009 GTD/Kanban
- [ ] Parse TODOs + simple frontmatter fields; `search --TODO|--kanban`.

### ADR-010 Lua Escape Hatch
- [ ] Implement limited hook API with spans, read-only engine views.

### ADR-011 Text Parsing Stack
- [ ] Migrate to markdown AST where needed; keep fast summaries; surface spans.

### ADR-012/013 Docs
- [ ] Man/tldr + notebook docs near v1.

### ADR-AI-001 Three-layer Cache
- [ ] Memory/disk cache + file locking + `--rebuild-cache`.
 - [ ] Debounced writes; crash-safe rebuild; clear rebuild policy (`validate --rebuild-cache`).

### ADR-AI-002 GTD/Kanban Integration
- [ ] Agenda outputs + Kanban states and completions integration plan.
 - [ ] Inline TODO parsing; retention window; due-date warnings; caching completion timestamps; NVIM color mapping.

### ADR-AI-003 Extensible Graph Edges
- [ ] Add `graph_edges` in schema; auto-validate refs; traversal uses all edges.
 - [ ] Per-edge overrides (severity, cross-schema); cycle detection policy per schema.

## Current Implementation Snapshot (for orientation)
- Config: see `src/config/` (`loader.rs`, `schema.rs`, `defaults.rs`, `template.rs`).
- Parsing: see `src/model.rs` (front matter + title); no AST yet.
- Validation: see `src/validate.rs` (ids, status, refs, schema rules, unknown policy).
- Graph: see `src/graph.rs` (BFS path, cluster); `src/commands/graph.rs` for renderers.
- Discovery: see `src/discovery/` (`scan.rs`, `per_base.rs`, `unified.rs`).
- Index/Watch: see `src/index.rs`, `src/watch.rs` (incremental, debounce; unified index writing at config root).
- CLI Surface: see `src/cli.rs` and `src/commands/*`.

## Ticketization Hints
- Prefer vertical slices aligned to phases above; add unit tests near code.
- Keep outputs stable; extend JSON/NDJSON with additive fields only.
- Gate larger features behind flags to ship incremental value.

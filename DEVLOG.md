# adr-rag – Dev Handoff (quick log)

Date: 2025-08-22

Scope: Recent enhancements to the repo‑local ADR CLI (schemas, indexing, doctor, watch, incremental validate) aligned with IMP-004.

What changed
- Init UX: `adr-rag init` opens `.adr-rag.toml` in an editor by default; `--silent` to skip. Upward config discovery kept.
- Commands: `pathcmd` → `path`. Added `completions` (bash|zsh|fish). Global `--format` applies to all commands.
- Validate behavior: Scans markdown and writes per‑base JSON indexes by default on success; `--dry-run` avoids writes; `--full-rescan` forces full parse. Warn (don’t fail) on isolated nodes (no edges).
- Incremental validate: Index now stores `mtime` and `size`; we re‑parse only new/changed files, drop removed, then write updated indexes on success.
- Watch mode: `adr-rag watch` uses FS notifications with debounce to re‑validate incrementally and update indexes continuously.
- Index format: Now `{ generated_at: <unix_secs>, items: [...] }`; entries include `file`, `id`, `title`, `tags`, `status`, `depends_on`, `supersedes`, `superseded_by`, `groups`, `type`, `mtime`, `size`. Loader accepts legacy array‑only indexes as well.
- Schemas (note types): Optional `[[schema]]` blocks in TOML control per‑type rules. Enabled defaults for ADR and IMP. Unknown keys policy per type (ignore|warn|error; default ignore). Basic rules supported: enums (`allowed`), arrays (`type="array"`, `min_items`), regex, dates (`type="date"`, `format`), and `refers_to_types` for cross‑type deps. Index records include `type`.
- Doctor: Adds per‑type counts and unknown‑key stats, alongside config summary and conflict detection.
- README + template: Generalized defaults (no project‑specific paths). Documented schemas, unknown policy, completions, incremental validate, and watch.

Notable file touches
- `tools/adr-rag/src/main.rs`: command surface, schema parsing + validation, incremental collector, watch subcommand, doctor/reporting, index writer/loader.
- `tools/adr-rag/Cargo.toml`: added `notify`, `rayon` (for future parallel parse), `chrono` dropped in favor of std time for timestamps.
- `tools/adr-rag/README.md`: updated usage, flags, schema docs, and behavior notes.

Usage quickies
- Validate (incremental): `adr-rag validate` (writes indexes on success)
- Full rescan: `adr-rag validate --full-rescan`
- Dry run: `adr-rag validate --dry-run`
- Watch: `adr-rag watch` (debounce 400ms; writes on success)
- Doctor JSON: `adr-rag doctor --format json`
- Completions: `adr-rag completions bash|zsh|fish`

Open ideas / next steps
- Graph export: Add `graph` subcommand to emit Graphviz DOT for clusters or filtered views.
- Parallel parse: Wire up Rayon in incremental collector (changed files only) if needed.
- Git‑assisted mode (optional/later): Use git status to prune candidate set in very large repos.
- `config edit` subcommand to open resolved config directly.
- Doctor: optional per‑type unknown key listing when requested.

Notes
- Schema validation currently applies to files we parsed on this run (changed/new); cross‑ref checks still validate across all loaded docs.
- Multi‑base behavior: indexes and groups remain per‑base; read‑path picks up existing indexes when present, otherwise scans.

---

Refactor (2025-08-23)
- Split monolithic `main.rs` (~1.4k LOC) into cohesive modules under `src/`:
  - `config`, `model`, `discovery`, `index`, `validate`, `graph`, `util`, `watch`, plus `cli` and `commands/*`.
- Introduced `src/lib.rs` (library crate) and moved the binary entry to `src/bin/adr-rag.rs` to keep the bin thin.
- Normalized JSON printing via `commands::output::print_json`.
- Kept behavior the same; cargo build/test pass.

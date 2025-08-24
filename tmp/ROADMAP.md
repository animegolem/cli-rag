# CLI-RAG Roadmap (Brief)

A concise plan to tighten the core library + CLI for agent/automation use, defer heavy graph features, and prepare extraction to a dedicated repo.

## Scope and Direction
- Focus: correctness, stable NDJSON/JSON protocol, predictable CLI ergonomics.
- Pause: new graph features (DOT subgraphs, styling). Keep current `graph` minimal and mark experimental.
- Goal: produce a clean library consumed by CLI and, later, MCP.

## Status Snapshot
- Core implemented: discovery (index-first), validation rules by schema, index versioning, NDJSON beginnings, Windows-safe globs, de-dupe by id.
- Next immediate: retarget Phase F to protocol + ergonomics (below).

## Rollout Order (High-Level)
- Step 0 (now): Repo hygiene + T0 tests — keep/expand inline unit tests; add .gitignore and ensure fmt/clippy clean.
- Step 1 (parallel): T1 integration harness — add `tests/` and minimal stable tests (help/doctor/init) without touching protocol.
- Step 2: Phase F protocol scaffolding — introduce `protocol` module and centralized output, no behavior change; keep T1 green.
- Step 3: Phase F command rewiring — migrate commands to central output; expand T2 tests for search/topics/group/validate.
- Step 4: Protocol lock — finalize structs/schemas; add T3 snapshot/golden tests.
- Step 5: Optional polishing — T4 watch/graph tests as time permits.

## Phase F — Protocol + Ergonomics (Retargeted)

- Output formats and flags
  - [x] Switch global `--format` to clap ValueEnum: `plain|json|ndjson` (default `plain`)
  - [x] Keep `graph --format` as ValueEnum: `mermaid|dot|json` (mark experimental)
  - [x] Include `groups` in search JSON/NDJSON; pending cluster/graph
- Canonical protocol types
  - [x] Introduce `protocol` module with Serialize (JSON Schema later via `schemars`)
  - [ ] Define stable NDJSON records:
    - search_result: `{ id, title, file, tags, status, groups }`
    - topic_count: `{ topic, count }`
    - group_member: `{ id, title, status, groups, file? }`
    - validate_header: `{ ok, doc_count }`
    - validate_issue: `{ type: "error"|"warning", file, message, code? }`
    - cluster_member: `{ id, title, status, groups }`
    - graph_minimal (if kept): `{ root, nodes:[{id,title,status,groups}], edges:[{from,to}] }`
  - [ ] Generate and commit JSON Schemas under `tools/adr-rag/protocol/`
- Emission helpers
  - [x] Centralize plain/json/ndjson printing scaffold in `commands::output` (enum `Format`)
  - [x] Rewire `search`, `topics`, `group`, `cluster` to protocol types (JSON/NDJSON)
  - [x] Lock `validate` JSON shape to typed issues and include `doc_count`; keep NDJSON typed
  - [ ] Ensure `topics`, `group`, `cluster`, `search`, `validate` all support `ndjson`
- Validate/watch behavior
  - [ ] Library-only returns `ValidationReport`; CLI controls exit codes
  - [ ] Index writes are atomic (tmp + rename); document behavior
  - [ ] Optional alias: `validate --watch-once` (same as `watch`) to keep indices warm
- Graph scope (defer heavy features)
  - [ ] Mark `graph` experimental in README/help
  - [ ] Unify `graph --format json` to `graph_minimal` schema
  - [ ] Defer DOT subgraphing

Acceptance (Phase F)
- [x] Clap enums validate formats; typos fail at parse time
- [ ] NDJSON records conform to committed JSON Schemas
- [ ] `groups` present where applicable
- [ ] Central exit-code control; side effects documented
- [ ] README updated for NDJSON-first workflow and experimental graph status

## Testing Roadmap (Parallel to Phase F)

Rationale: minimize churn by starting with stable seams now and growing coverage alongside changes.

T0 — Inventory and Keep Inline
- [x] (Start now) Catalog current inline unit tests (`model`, `validate`, `commands::graph`).
- [x] (Start now) Expand edge cases in-place (no FM, TOML FM, CRLF, missing title; schema rules, isolated ADRs).
- [x] (Start now) Avoid moving tests that need private access until module boundaries stabilize.

T1 — Stable Integration Harness
- [x] (Start in parallel with Phase F scaffolding) Add dev-deps: `assert_cmd`, `predicates`, `assert_fs`/`tempfile`.
- [x] Create `tests/` with minimal fixtures and helpers.
- [x] Add resilient tests for stable surfaces: `doctor --format json` (fixtures minimal).
- [x] Add `--help` test (prints usage and key commands).
- [x] Add `init --silent --force` test (writes config in temp dir).

 T2 — Command Coverage Without Protocol Lock
  - [x] Fixtures for: minimal valid set (for search/topics/group/cluster).
  - [x] Tests for `search`, `topics`, `group`, and `cluster`.
  - [x] Basic `validate --format ndjson --dry-run` header test (empty base).
  - [x] Tests for `validate --format json --write-groups` (non-empty groups; verify sections/ids).
  - [x] Config precedence tests (`--config`, env vars, --base).
  - [x] Group NDJSON test (header + member lines).

T3 — Protocol Lock + Snapshots
- [ ] Introduce central `protocol` structs (NDJSON/JSON) and JSON Schemas.
- [ ] Add golden/snapshot tests (`insta` or `trycmd`) for protocol outputs.
- [ ] Roundtrip serde tests for protocol types.

T4 — Nice-to-have
- [ ] Graph rendering snapshots (Mermaid/DOT minimal).
- [ ] Watch debounce behavior (feature-flagged, may mock filesystem events).

Exit Criteria
- [ ] CI runs `fmt`, `clippy -D warnings`, `test` (unit + integration).
- [ ] Integration tests cover at least one happy path per command.
- [ ] Protocol outputs validated against committed schemas (once locked in T3).

## Near-Term Backlog (after Phase F)
- [ ] `config edit` (open `.adr-rag.toml`)
- [ ] Friendlier errors (plain) + structured codes (json/ndjson)
- [ ] Extend NDJSON to any remaining commands
- [ ] Tables/colors for doctor/validate (Phase B polish) — optional

## Authoring Enhancements (Opt-in, staged)
- [ ] `new --schema <S> --title <T> [--id ...] [--edit]` scaffold with schema `template`
- [ ] Config extensions (per schema):
  - `identity_source = "frontmatter" | "filename"` (default `frontmatter`)
  - `filename_id_regex = "^(ADR|IMP)-(\\d+)"`
  - `template = """..."""` with `{{id}}`/`{{title}}`
  - `max_length_lines = <int>`, `min_backlinks = <int>` (rule severities apply)
- [ ] Optional `editor = "nvim" | "code" | ...` override

## Discovery and Performance
- Index-first reads (already in place): commands prefer index when present; otherwise scan.
- Recommend running `validate` once (or `watch`) before heavy usage.
- Defer daemon/server mode unless latency becomes a pain point.

## Extraction Plan — New CLI-RAG Repo
- [ ] Create new repo (e.g., `cli-rag`) and migrate from `tools/adr-rag/`
- [ ] Preserve history (git subtree split or `git filter-repo`)
- [x] Add `Justfile` (initial):
  - `just setup` (toolchain, hooks)
  - `just dev-build` / `just build` (release)
  - `just test` (unit + integration)
  - `just fmt`, `just fmt-check`, `just lint`, `just run -- …`
  - `just schema` (re-generate JSON Schemas)
  - `just docs` (README/protocol refresh)
- [ ] CI: fmt, clippy, build, tests, schema drift detection
- [ ] Publish protocol JSON Schemas as artifacts for MCP consumers

## Future UI — Thin TUI + MCP (Vision)
- Thin TUI
  - Simple “master control” view (templates, tracked notes)
  - Launch configured editor; on exit, run `validate`/watch to update index
  - Universal fuzzy jump across tracked notes
- MCP integration
  - Consume NDJSON protocol + JSON Schemas
  - Agent creates notes via `new --schema ...` and relies on validation and index update

## Next Steps
- Implement Phase F (retargeted) items above
- Update README to reflect NDJSON-first workflow and experimental graph
- Proceed with repo extraction once Phase F is complete

# CLI-RAG Roadmap (Brief)

A concise plan to tighten the core library + CLI for agent/automation use, defer heavy graph features, and prepare extraction to a dedicated repo.

## Scope and Direction
- Focus: correctness, stable NDJSON/JSON protocol, predictable CLI ergonomics.
- Pause: new graph features (DOT subgraphs, styling). Keep current `graph` minimal and mark experimental.
- Goal: produce a clean library consumed by CLI and, later, MCP.

## Status Snapshot
- Core implemented: discovery (index-first), validation rules by schema, index versioning, NDJSON beginnings, Windows-safe globs, de-dupe by id.
- Next immediate: retarget Phase F to protocol + ergonomics (below).

## Phase F — Protocol + Ergonomics (Retargeted)

- Output formats and flags
  - [ ] Switch global `--format` to clap ValueEnum: `plain|json|ndjson` (default `plain`)
  - [ ] Keep `graph --format` as ValueEnum: `mermaid|dot|json` (mark experimental)
  - [ ] Include `groups` in all JSON/NDJSON outputs (search/cluster/graph)
- Canonical protocol types
  - [ ] Introduce `protocol` module with Serialize + JSON Schema (`schemars`)
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
  - [ ] Centralize plain/json/ndjson printing in `commands::output` for protocol types
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
- [ ] Clap enums validate formats; typos fail at parse time
- [ ] NDJSON records conform to committed JSON Schemas
- [ ] `groups` present where applicable
- [ ] Central exit-code control; side effects documented
- [ ] README updated for NDJSON-first workflow and experimental graph status

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
- [ ] Add `Justfile`:
  - `just setup` (toolchain, hooks)
  - `just build` / `just release`
  - `just test` (unit + schema generation check)
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


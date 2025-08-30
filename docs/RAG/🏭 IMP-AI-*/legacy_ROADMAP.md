 # CLI-RAG Roadmap (Brief)

Legend
- [DO-NEXT]: Safe to do now; does not change public CLI/TOML/protocol surfaces
- [WAIT-CLI]: Wait until CLI verbs/flags are frozen (ADR-003a/b)
- [WAIT-CONFIG]: Wait until TOML format is frozen (ADR-001/ADR-004)
- [WAIT-PROTOCOL]: Wait until protocol record shapes are finalized (protocol.rs)
- [BLOCKED-ADR-003]: Conflicts with ADR-003 as written
- [BLOCKED-ADR-001]: Conflicts with ADR-001/ADR-004 as written

## Current Efforts 

- Output formats and flags
  - [x] Switch global `--format` to clap ValueEnum: `plain|json|ndjson` (default `plain`)
  - ~~[x] Keep `graph --format` as ValueEnum: `mermaid|dot|json` (mark experimental)~~ [BLOCKED-ADR-003] Renamed per ADR-003 to `--graph-output` (update code/docs together) [WAIT-CLI]
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
  - [ ] Generate and commit JSON Schemas under `cli-rag/docs/protocol/` [DO-NEXT]
- Emission helpers
  - [x] Centralize plain/json/ndjson printing scaffold in `commands::output` (enum `Format`)
  - [x] Rewire `search`, `topics`, `group`, `cluster` to protocol types (JSON/NDJSON)
  - [x] Lock `validate` JSON shape to typed issues and include `doc_count`; keep NDJSON typed
  - [x] Ensure `topics`, `group`, `cluster`, `search`, `validate` all support `ndjson`
- Validate/watch behavior
  - [ ] Library-only returns `ValidationReport`; CLI controls exit codes [DO-NEXT]
  - [ ] Index writes are atomic (tmp + rename); document behavior [DO-NEXT]
  
Acceptance (Phase F)
- [x] Clap enums validate formats; typos fail at parse time
- [ ] NDJSON records conform to committed JSON Schemas [WAIT-PROTOCOL]
- [ ] `groups` present where applicable [WAIT-PROTOCOL]
- [ ] Central exit-code control; side effects documented [DO-NEXT]
- [ ] README updated for NDJSON-first workflow and experimental graph status [DO-NEXT]

## Testing Roadmap 

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
- [X] Introduce central `protocol` structs (NDJSON/JSON)
- [ ] Introduce JSON Schemas. [WAIT-PROTOCOL]
- [ ] Add golden/snapshot tests (`insta` or `trycmd`) for protocol outputs. [WAIT-PROTOCOL]
- [ ] Roundtrip serde tests for protocol types. [WAIT-PROTOCOL]

T4 — Nice-to-have
- [ ] Graph rendering snapshots (Mermaid/DOT minimal).
- [ ] Watch debounce behavior (feature-flagged, may mock filesystem events).

Exit Criteria
- [ ] CI runs `fmt`, `clippy -D warnings`, `test` (unit + integration).
- [ ] Integration tests cover at least one happy path per command.
- [ ] Protocol outputs validated against committed schemas (once locked in T3).

## Near-Term Backlog (after Phase F)
- [ ] `config edit` (open `.adr-rag.toml`) [WAIT-CONFIG]
- [ ] Friendlier errors (plain) + structured codes (json/ndjson)
- [ ] Extend NDJSON to any remaining commands
- [ ] Tables/colors for doctor/validate (Phase B polish) — optional

## Authoring Enhancements (Opt-in, staged)
- [ ] `new --schema <S> --title <T> [--id ...] [--edit]` scaffold with schema `template` [WAIT-CONFIG]
- [ ] Config extensions (per schema):
  - `identity_source = "frontmatter" | "filename"` (default `frontmatter`) [WAIT-CONFIG]
  - `filename_id_regex = "^(ADR|IMP)-(\\d+)" [WAIT-CONFIG]`
  - `template = """..."""` with `{{id}}`/`{{title}}` [WAIT-CONFIG]
  - `max_length_lines = <int>`, `min_backlinks = <int>` (rule severities apply) [WAIT-CONFIG]
- [ ] Optional `editor = "nvim" | "code" | ...` override [WAIT-CONFIG]

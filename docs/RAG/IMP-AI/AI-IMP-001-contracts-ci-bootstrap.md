---
node_id: AI-IMP-001
tags:
  - IMP-LIST
  - Implementation
confidence_score: 0.8
created_date: 2025-09-10
---

# AI-IMP-001-contracts-ci-bootstrap

## Phase 1: ResolvedConfig + Info/Validate Alignment
<!-- Define the Current issue, it's scope, and intended remediation -->
<!-- Define a single, measurable outcome. What specific state means we are done? -->
<!-- Link to project docs as relevant or present (eg, adr, imp, log) -->
The CLI must emit JSON that conforms to contracts for `info` and `validate`, and write a resolved config snapshot. Target contracts:
- ResolvedConfig: `contracts/v1/resolved_config.json`
- Info: `contracts/cli/info.schema.json`
- Validate: `contracts/cli/validate_result.schema.json`

Outcome: running `info --format json` and `validate --format json --dry-run` on a minimal repo validates against the above schemas; `validate` (non-dry) writes `.cli-rag/resolved.json` which validates against ResolvedConfig.

### Files to Touch
- `src/commands/doctor.rs` (rename to `info` surface and JSON shape)
- `src/cli.rs` (rename Doctor→Info, flags)
- `src/commands/validate_cmd.rs` (envelope `{ ok, docCount, diagnostics[] }`)
- `src/protocol.rs` (ensure `PROTOCOL_VERSION`; helper structs if needed)
- `src/index.rs` (resolved config snapshot write, if placed here)
- `docs/RAG/BRIDGE_PLAN_V1.md` (mark Phase 1 complete when done)

### Implementation Checklist
- [ ] CLI: Rename `Doctor` command to `Info`; update help and flags.
- [ ] Info JSON: include `protocolVersion`, `{config:{path,version,deprecated}}`, `{index:{path,exists}}`, `{cache:{aiIndexPath,exists}}`, `{capabilities:{watchNdjson,aiGet:{retrievalVersion},pathLocations,luaApiVersion}}`.
- [ ] Validate JSON: change output to `{ ok, docCount, diagnostics:[{severity,code,msg,path?,span?,field?,nodeId?}] }` (map existing errors/warnings into unified array with normalized fields and codes).
- [ ] Resolved snapshot: after non‑dry validate, write `.cli-rag/resolved.json` (camelCase) matching `contracts/v1/resolved_config.json`.
- [ ] Deterministic ordering and casing: ensure keys are camelCase in JSON, snake_case remains internal/TOML.
- [ ] Exit codes: preserve `0/1/2/4/5` semantics as per `contracts/global-conventions.md` for these commands.

### Acceptance Criteria
Given a minimal repo using `contracts/v1/user_config/cli-rag.toml`
When I run `cli-rag info --format json`
Then the JSON validates against `contracts/cli/info.schema.json` and includes `protocolVersion`.

Given the same repo
When I run `cli-rag validate --format json --dry-run`
Then the JSON validates against `contracts/cli/validate_result.schema.json` and `ok=true` for empty repos.

Given the same repo
When I run `cli-rag validate --format json`
Then `.cli-rag/resolved.json` exists and validates against `contracts/v1/resolved_config.json`.

## Phase 1 CI: Contracts Gates
Add a `contracts` job to CI (docs only here; workflow will be edited in a follow‑up PR):
- Validate schemas load (parse) for `contracts/cli`, `contracts/index`, `contracts/v1`.
- Build release binary.
- Spin a temp fixture:
  - Write `.cli-rag.toml` from `contracts/v1/user_config/cli-rag.toml` (adjust base path).
  - Create an empty notes base.
- Run and validate:
  - `info --format json` → `contracts/cli/info.schema.json`.
  - `validate --format json --dry-run` → `contracts/cli/validate_result.schema.json`.
  - `validate --format json` then validate `.cli-rag/resolved.json` → `contracts/v1/resolved_config.json`.

## Phase 2: Unified Index (Edge Kind + Locations)
Unify index writing to single path `config.scan.index_path`; ensure edges carry `kind` and optional `locations`.

### Files to Touch
- `src/index.rs`, `src/discovery/unified.rs`, `src/discovery/*`
- `src/model.rs` (if spans needed)
- `docs/RAG/BRIDGE_PLAN_V1.md`

### Implementation Checklist
- [ ] Write only one authoritative index at `scan.index_path`.
- [ ] Edges include `{from,to,kind}`; mention‑derived edges include `locations[{path,line}]`.
- [ ] Populate minimal `computed` fields per schema.

### Acceptance Criteria
When I run non‑dry validate
Then `.cli-rag/index.json` validates against `contracts/index/index.schema.json`.

## Issues Encountered
N/A at ticket creation.


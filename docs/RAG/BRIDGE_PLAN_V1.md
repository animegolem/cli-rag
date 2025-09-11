---
title: Bridge & Build Plan v1
status: draft
---

# Bridge & Build Plan v1

Purpose: contracts‑first master checklist (supersedes IMP-* tickets). Tracks current status, gaps, priorities, and conventions for the v1 surface. Contracts live under `contracts/` and are authoritative.

## Conventions
- Casing: TOML/Lua snake_case; JSON camelCase
- Exit codes: see `contracts/global-conventions.md`
- Deterministic ordering: search, neighbors, graph/path per conventions
- NDJSON watch handshake: first event `{"event":"watch_start","protocolVersion":1}`

## CLI Surfaces

- info [[AI-IMP-001-contracts-ci-bootstrap]]
  - Contract: `contracts/v1/cli/info.schema.json`
  - Status: Complete (protocolVersion, config/index/cache, capabilities)
  - Gaps: None for Phase 1
  - Priority: High (done)

- validate
  - Contract: `contracts/v1/cli/validate_result.schema.json`
  - Status: Complete (`{ ok, docCount, diagnostics[] }`, exit 2 on failure)
  - Gaps: Optional span/field/nodeId as future enrichment
  - Priority: High (done)

- search
  - Contract: `contracts/cli/search_result.schema.json`
  - Status: returns bare array; must return `{results:[...]}` with typed `note|todo|kanban`
  - Gaps: add filters and deterministic sort
  - Priority: Medium

- graph
  - Contract: `contracts/v1/cli/graph.schema.json`
  - Status: Pending (current uses members; lacks `kind`)
  - Gaps: output `root`, `nodes`, `edges[{from,to,kind}]`
  - Priority: Medium

- path
  - Contract: `contracts/v1/cli/path.schema.json`
  - Status: Pending (missing `ok`, nodes, edge `kind`/`locations`)
  - Gaps: align to contract; include `locations`
  - Priority: Medium

- ai get
  - Contract: `contracts/v1/cli/ai_get.schema.json`
  - Status: ensure `protocolVersion`, `retrievalVersion`, neighbors ordering/limits
  - Gaps: implement `neighborStyle` variants; enforce depth/fanout policies
  - Priority: Medium

- ai index (plan/apply)
  - Contracts: `contracts/v1/cli/ai_index_plan.schema.json`, `contracts/v1/cli/ai_index_apply_report.schema.json`
  - Status: plan/apply wiring TBD; cache shape in ADR aligns
  - Gaps: hashing of source index; tag write policy
  - Priority: Low

## ResolvedConfig (camelCase)
- Contract: `contracts/v1/config/resolved_config.json`
- Status: Updated to enumerate `validate.frontmatter|body|edges|gtd`; `defaultMaxNodes` removed
- Gaps: None major; implementation to follow (Lua overlay/versioning)
- Priority: High (loader + emitter)

## Unified Index
- Contract: `contracts/v1/index/index.schema.json`
- Status: Complete (single index at `config.index_relative`, edges.kind, mentions.locations)
- Gaps: Additional computed fields optional (tokenEstimate, topics)
- Priority: High (done)

## Watch (NDJSON)
- Contract reference: `contracts/global-conventions.md`
- Status: Stream events; add initial `watch_start`
- Gaps: event envelope standardization
- Priority: Medium

## Lua API
- Contract reference: `contracts/global-conventions.md` (hooks + ctx)
- Status: overlay semantics and versioning to implement
- Gaps: sandbox, `--no-lua`, `luaApiVersion` exposure in info
- Priority: Medium

## Implementation Order (suggested)
1) Loader/ResolvedConfig emitter (config + Lua overlay; write snapshot)
2) Info/Validate JSON surfaces to contract shapes
3) Unified index writing (edges.kind/locations, computed fields)
4) Graph/Path outputs to contract shapes
5) Search envelope + filters (note/todo/kanban kinds)
6) AI Get neighbors/policies; AI Index plan/apply basics
7) Watch NDJSON handshake and event envelope

## Rollout With CI (Contracts‑First)

- Phase 0 (pre‑refactor)
  - CI: ensure schemas parse (load all in `contracts/cli`, `contracts/index`, `contracts/v1`).
  - CI: run rustfmt, clippy, unit/integration tests, line guard.
  - Docs: AGENTS.md contracts‑first + refactor‑boldly present (done).

- Phase 1 (ResolvedConfig + Info/Validate)
  - Impl: emit `.cli-rag/resolved.json` snapshot; align `info` and `validate` to schemas.
  - CI gates:
    - Validate `info --format json` → `contracts/v1/cli/info.schema.json`.
    - Validate `validate --format json --dry-run` → `contracts/v1/cli/validate_result.schema.json`.
    - Validate `.cli-rag/resolved.json` → `contracts/v1/config/resolved_config.json`.

- Phase 2 (Graph/Path)
  - Impl: align `graph --format json` and `path --format json` to contracts (`v1/cli/graph.schema.json`, `v1/cli/path.schema.json`).
  - CI gates:
    - Validate `graph` output on a small fixture.
    - Validate `path` output on a small fixture.

- Phase 3 (Search)
  - Impl: envelope `{results:[...]}` with typed items and deterministic ordering.
  - CI gate: validate `search --format json` → `contracts/v1/cli/search_result.schema.json`.

- Phase 4 (AI surfaces)
  - Impl: `ai get` neighbors/style policies; `ai index plan/apply` basics.
  - CI gates: validate outputs against `contracts/v1/cli/ai_get.schema.json`, `contracts/v1/cli/ai_index_plan.schema.json`, `contracts/v1/cli/ai_index_apply_report.schema.json` on small fixtures.

- Phase 5 (Watch)
  - Impl: NDJSON handshake first event and event envelopes.
  - CI gate: spawn short watch session; assert first line handshake, then terminate.

### Change Controls
- Schema changes require PR label `schema-change` and an update to this bridge plan.
- Breaking output changes bump `PROTOCOL_VERSION`.

## Notes
- Contracts are authoritative. Prefer updating `contracts/` over ADR text.
- Alpha/non‑prod: remove dead code, avoid compatibility crutches.

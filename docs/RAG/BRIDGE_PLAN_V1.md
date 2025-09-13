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
- CLI flags: long flags use kebab-case (e.g., `--graph-format`, `--full-rescan`).

## CLI Surfaces

- info [[AI-IMP-001-contracts-ci-bootstrap {COMPLETE}]]
  - Contract: `contracts/v1/cli/info.schema.json`
  - Status: Complete (protocolVersion, config/index/cache, capabilities)
  - Gaps: None for Phase 1
  - Priority: High (done)

- validate
  - Contract: `contracts/v1/cli/validate_result.schema.json`
  - Status: Complete (`{ ok, docCount, diagnostics[] }`, exit 2 on failure)
  - Gaps: Optional span/field/nodeId as future enrichment
  - Priority: High (done)

- search [[AI-IMP-003-gtd-schema-polish {COMPLETE}]] [[AI-IMP-004-gtd-emitters-and-ci {COMPLETE}]]
  - Contract: `contracts/v1/cli/search_result.schema.json`
  - Status: Complete (protocolVersion; envelope `{results:[...]}`; filters; GTD emitters for todos/kanban with dueDate, priorityScore, span, source)
  - Gaps: Optional scoring refinements
  - Priority: Done

- graph [[AI-IMP-002-graph-path-contracts-alignment {COMPLETE}]]
  - Contract: `contracts/v1/cli/graph.schema.json`
  - Status: Complete (root, nodes, edges with kind; deterministic ordering; protocolVersion)
  - Gaps: None
  - Priority: Done

- path [[AI-IMP-002-graph-path-contracts-alignment {COMPLETE}]]
  - Contract: `contracts/v1/cli/path.schema.json`
  - Status: Complete (ok flag; nodes; edges with kind and locations; protocolVersion)
  - Gaps: None
  - Priority: Done

- ai get [[AI-IMP-004-gtd-emitters-and-ci {COMPLETE}]] [[AI-IMP-005-ai-get-neighbor-policies-and-style {COMPLETE}]]
  - Contract: `contracts/v1/cli/ai_get.schema.json`
  - Status: Complete (protocolVersion/retrievalVersion; neighborStyle metadata|outline|full; depth/fanout enforcement; GTD hints on root and neighbors)
  - Gaps: Optional outline tuning and scoring
  - Priority: Done

- ai index (plan/apply) 
  - Contracts: `contracts/v1/cli/ai_index_plan.schema.json`, `contracts/v1/cli/ai_index_apply_report.schema.json`
  - Status: plan/apply wiring TBD; cache shape in ADR aligns
  - Gaps: hashing of source index; tag write policy
  - Priority: Low

## ResolvedConfig (camelCase)
- Contract: `contracts/v1/config/resolved_config.json`
- Status: Snapshot emitter implemented (`.cli-rag/resolved.json`); fields aligned per contract
- Gaps: Lua overlay/versioning not yet implemented
- Priority: High (loader/overlay)

## Unified Index
- Contract: `contracts/v1/index/index.schema.json`
- Status: Complete (single index at `config.index_relative`, edges.kind, mentions.locations)
- Gaps: Additional computed fields optional (tokenEstimate, topics)
- Priority: High (done)

## Watch (NDJSON) [[AI-IMP-006-watch-ndjson-handshake-and-events {COMPLETE}]]
- Contract reference: `contracts/global-conventions.md`
- Status: Complete (first event handshake `{event:watch_start, protocolVersion:1}`; standardized event envelopes)
- Gaps: None
- Priority: Done

## Lua API
- Contract reference: `contracts/global-conventions.md` (hooks + ctx)
- Status: overlay semantics and versioning to implement
- Gaps: sandbox, `--no-lua`, `luaApiVersion` exposure in info
- Priority: Medium

## Implementation Order (status)
1) Loader/ResolvedConfig emitter (snapshot complete; Lua overlay pending)
2) Info/Validate JSON surfaces to contract shapes — Complete
3) Unified index writing (edges.kind/locations, computed fields) — Complete
4) Graph/Path outputs to contract shapes — Complete
5) Search envelope + filters (note/todo/kanban kinds) + GTD enrichments — Complete
6) AI Get neighbors/policies; GTD hints — Complete; AI Index plan/apply basics — Pending
7) Watch NDJSON handshake and event envelope — Complete

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
  - Impl: envelope `{results:[...]}` with typed items and deterministic ordering; emit GTD enrichments.
  - CI gate: validate `search --format json` → `contracts/v1/cli/search_result.schema.json`.

- Phase 4 (AI surfaces)
  - Impl: `ai get` neighbors/style policies; emit GTD hints; `ai index plan/apply` basics.
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

---
title: Bridge & Build Plan v2
status: draft
created_date: 2025-09-14
---

# Bridge & Build Plan v2

Purpose: track the next phase of work post‑v1 to close remaining gaps, align surfaces with ADR-003d, and add the AI Index and Lua overlay capabilities. Contracts remain authoritative.

## Goals
- Implement Lua overlay and versioning with `--no-lua` switch; expose `luaApiVersion` and provenance in outputs where applicable.
- Deliver `ai index plan` and `ai index apply` per contracts (`contracts/v1/cli/ai_index_plan.schema.json`, `contracts/v1/cli/ai_index_apply_report.schema.json`).
- Remove or deprecate legacy `topics`/`group` surfaces and groups JSON emission per ADR-003d.
- Gate capability flags accurately (e.g., advertise `aiIndex` only when plan/apply available).

## Scope
- Config/Lua
  - Load order: defaults → TOML → repo Lua overlay (`.cli-rag.lua`) → user Lua (`~/.config/cli-rag/config.lua`).
  - `--no-lua` disables overlays completely.
  - Minimal stable hooks: `id_generator`, `render_frontmatter`, `validate` (shape per `contracts/global-conventions.md`).
  - Update resolved snapshot to include overlay provenance and keep camelCase alignment to `contracts/v1/config/resolved_config.json`.

- AI Index
  - `ai index plan`: compute clusters over graph edges; deterministic IDs; include `sourceIndexHash`; write JSON to `--output`.
  - `ai index apply`: persist authoritative cache at `.cli-rag/cache/ai-index.json`; optional tag writes (additive) with schema check; emit apply report JSON.
  - Exit codes and hashing per global conventions.

- Surfaces cleanup
  - Remove or deprecate `topics` and `group` commands; migrate docs to tags + AI Index.
  - Update help/completions accordingly.

## Non‑Goals
- Search ranking overhaul beyond minor tuning.
- Advanced token estimates or embeddings; out of scope for v2.

## Deliverables
- Lua overlay loader with `--no-lua`.
- New subcommands: `ai index plan`, `ai index apply`.
- Accurate `info` capability flags (toggle `aiIndex` true only when supported).
- Removal/deprecation path for groups surfaces.
- CI additions: jsonschema validation for ai index plan/apply outputs; Lua overlay smoke test (with and without `--no-lua`).

## CI & Contracts
- Add gates validating:
  - `ai index plan` JSON → `contracts/v1/cli/ai_index_plan.schema.json`.
  - `ai index apply` JSON → `contracts/v1/cli/ai_index_apply_report.schema.json`.
  - `info` reflects correct capability flags.
  - Resolved snapshot still validates after overlay.

## Risks & Mitigations
- Lua security: keep hooks minimal and sandboxed; allow opt‑out with `--no-lua`.
- Backwards compat: additive changes; retain protocolVersion=1.

## Milestones
1) Lua overlay loader + `--no-lua` + CI smoke.
2) ai index plan implementation + CI schema gate.
3) ai index apply implementation + cache write + CI gate.
4) Capability gating + groups deprecation + doc updates.


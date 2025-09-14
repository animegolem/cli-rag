---
title: Bridge & Build Plan v2
status: draft
created_date: 2025-09-14
---

# Bridge & Build Plan v2

Purpose: track the next phase of work post‑v1 to close remaining gaps, align surfaces with ADR-003d, and add the AI Index and Lua overlay capabilities. Contracts remain authoritative.

## Goals
- Finalize Lua versioning exposure (overlay + `--no-lua` + hooks implemented); expose `luaApiVersion` and provenance where applicable.
- Deliver `ai index plan` and `ai index apply` per contracts (`contracts/v1/cli/ai_index_plan.schema.json`, `contracts/v1/cli/ai_index_apply_report.schema.json`).
- Remove or deprecate legacy `topics`/`group` surfaces and groups JSON emission per ADR-003d.
- Gate capability flags accurately (e.g., advertise `aiIndex` only when plan/apply available).
- Config versioning: read `config_version` from TOML when present; default sensibly; expose in `info` and `resolved.json` (source-of-truth from config rather than hardcoded).

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
- Lua version exposure and provenance in info/resolved (overlay already implemented).
- `ai index plan` (DONE) and `ai index apply`.
- Accurate `info` capability flags (toggle `aiIndex` true only when supported).
- Removal/deprecation path for groups surfaces.
- CI additions: jsonschema validation for ai index plan/apply outputs; Lua overlay smoke test (with and without `--no-lua`).

## CI & Contracts
- Add gates validating:
  - `ai index plan` JSON → `contracts/v1/cli/ai_index_plan.schema.json`. (ADD)
  - `ai index apply` JSON → `contracts/v1/cli/ai_index_apply_report.schema.json`. (AFTER IMPLEMENT)
  - `info` reflects correct capability flags.
  - Resolved snapshot still validates after overlay.

## Risks & Mitigations
- Lua security: keep hooks minimal and sandboxed; allow opt‑out with `--no-lua`.
- Backwards compat: additive changes; retain protocolVersion=1.

## Milestones
1) Lua version exposure + CI smoke for overlays on/off.
2) ai index plan implementation + CI schema gate. (DONE)
3) ai index apply implementation + cache write + CI gate. (NEXT)
4) Capability gating + groups deprecation + doc updates.

---
title: Bridge & Build Plan v2
status: draft
created_date: 2025-09-14
---

# Bridge & Build Plan v2

Purpose: track the next phase of work post‑v1 to close remaining gaps, align surfaces with ADR-003d, and add the AI Index and Lua overlay capabilities. Contracts remain authoritative.

## Goals
- Lua versioning exposure complete (overlay + `--no-lua` + hooks implemented); `luaApiVersion` exposed.
- AI Index: plan and apply implemented and validated in CI (contracts).
- Surfaces cleanup: legacy `topics`/`group` removed per ADR-003d.
- Capability flags accurate (`aiIndex` true).
- Config versioning: `config_version` read from TOML; exposed in `info` and `resolved.json`.
- Dogfooding: repo consumers use `.cli-rag.toml` + managed templates when creating new ADR/IMP/EPIC notes.

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
- Lua version exposure in info/resolved (DONE) and overlay smoke in CI (DONE).
- `ai index plan` and `ai index apply` (DONE), with CI schema validation for both (DONE).
- Capability flags accurate in info (DONE).
- Removal of groups surfaces (DONE).

## CI & Contracts
- Gates in place:
  - `ai index plan` JSON → `contracts/v1/cli/ai_index_plan.schema.json`.
  - `ai index apply` JSON (dry + non-dry) → `contracts/v1/cli/ai_index_apply_report.schema.json`.
  - `info` reflects correct capability flags.
  - Resolved snapshot validates; overlay smoke tests run (enabled vs `--no-lua`).

## Risks & Mitigations
- Lua security: keep hooks minimal and sandboxed; allow opt‑out with `--no-lua`.
- Backwards compat: additive changes; retain protocolVersion=1.

## Milestones
1) Lua version exposure + CI smoke for overlays on/off. (DONE)
2) ai index plan implementation + CI schema gate. (DONE)
3) ai index apply implementation + cache write + CI gate. (DONE)
4) Capability gating + groups removal + doc updates. (DONE)
5) Dogfooding migration: nested `.cli-rag.toml`, schema imports, and `cli-rag ai new` adoption. (DONE)

### Pending (nice-to-haves)
- Search scoring tweaks; extended diagnostics spans; additional computed fields in index (non-blocking).

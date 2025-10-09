---
node_id: AI-IMP-007
tags:
  - IMP-LIST
  - Implementation
  - lua
  - config
  - cli
kanban_status: done
depends_on:
  - ADR-003d
  - ADR-006
confidence_score: 0.78
created_date: 2025-09-14
close_date:
---

# AI-IMP-007-lua-overlay-loader-and-no-lua

## Summary of Issue #1
Introduce a Lua overlay system with deterministic load order and an opt-out flag. Load overlays from repo `.cli-rag.lua` and user `~/.config/cli-rag/config.lua`, with load order: defaults → TOML → repo Lua → user Lua. Add a global `--no-lua` flag (and `CLI_RAG_NO_LUA=1`) to disable overlays. Expose `luaApiVersion` and overlay provenance in `info` and persist overlay metadata in `.cli-rag/resolved.json`. Outcome: overlays load safely when present, can be disabled, and resolved snapshot reflects final effective config with provenance.

### Out of Scope 
- Executing generation/validation hooks (covered by a separate ticket).
- Lua sandboxes beyond minimal environment scoping.
- Back-compat versioning/migrations for Lua modules.

### Design/Approach  
- Add `--no-lua` (global) to CLI; also honor `CLI_RAG_NO_LUA=1`.
- New module `src/config/lua.rs` for overlay discovery and loading; returns an overlay struct with optional fields mirroring TOML.
- Load order: start with parsed TOML, then merge overlay tables (shallow key overwrite) from repo file then user file.
- Record overlay provenance in `ResolvedConfig` (additionalProperties allowed): `{ overlays: { enabled, repoPath?, userPath? } }`.
- `info` continues to expose `luaApiVersion`; add a boolean `overlaysEnabled` inside `capabilities` when Lua is active.
- Keep hooks inert for now; only load/merge configuration-like fields so downstream tickets can use them.

### Files to Touch
- `src/cli.rs`: add `--no-lua` global flag.
- `src/config/mod.rs` and `src/config/loader.rs`: invoke overlay loader after TOML parse; merge results.
- `src/config/lua.rs`: new (overlay discovery/parse; minimal table mapping; helpers).
- `src/commands/validate_cmd.rs`: include overlay provenance in resolved snapshot.
- `src/commands/info.rs`: add `capabilities.overlaysEnabled` and reflect overlay paths.
- `contracts/global-conventions.md`: note `--no-lua` flag and load order (doc update).
- Tests: basic unit test for `--no-lua` disabling overlay merge; fixture-based path checks.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [x] CLI: Add `--no-lua` (bool) and environment var `CLI_RAG_NO_LUA` handling in `src/cli.rs`.
- [x] Config: Create `src/config/lua.rs` with functions to locate repo/user overlays and parse them (parse-as-data approach; no hooks yet).
- [x] Loader: After TOML load in `load_config`, call overlay loader unless disabled; store overlay metadata on config (merge of values deferred to hooks ticket).
- [x] Resolved snapshot: Include `{ overlays: { enabled, repoPath?, userPath? } }` in `.cli-rag/resolved.json`.
- [x] Info: Add `capabilities.overlaysEnabled` (true when overlays are active).
- [x] Docs: Update `contracts/global-conventions.md` to document load order and `--no-lua` (already documented; verified).
- [x] Tests: Add a integration test that sets up `.cli-rag.lua`, runs `validate` and asserts `resolved.json.overlays.enabled==true` and paths present.
- [x] Tests: Verify `--no-lua` and `CLI_RAG_NO_LUA=1` both disable overlays and resolved snapshot reflects `enabled==false`.
kanban_statusline: "Added --no-lua, overlay discovery, caps + resolved snapshot; tests in place"

### Acceptance Criteria
**GIVEN** a repo with `.cli-rag.lua` and no env flags, **WHEN** `cli-rag validate --format json` runs, **THEN** `.cli-rag/resolved.json` contains `overlays.enabled=true` and `repoPath` is the repo file path.
**AND** `cli-rag info --format json` includes `capabilities.overlaysEnabled=true`.

**GIVEN** `CLI_RAG_NO_LUA=1` or `--no-lua`, **WHEN** running the same commands, **THEN** overlays are not loaded/merged and `overlays.enabled=false` in the resolved snapshot; `capabilities.overlaysEnabled=false`.

### Issues Encountered 
(to be completed during implementation)

---
node_id: AI-IMP-008
tags:
  - IMP-LIST
  - Implementation
  - lua
  - validation
  - authoring
kanban_status: done
depends_on:
  - AI-IMP-007
  - ADR-003d
confidence_score: 0.72
created_date: 2025-09-14
close_date: 2025-09-14
---

# AI-IMP-008-lua-hooks-validation-and-new

## Summary of Issue #1
Wire minimal Lua hooks into validation and authoring to enable customization without changing core code. Add hooks: `validate(frontmatter, body, ctx) -> diagnostics[]`, `id_generator(schema, ctx) -> { id }`, and `render_frontmatter(schema, title?, ctx) -> map`. Ensure hooks are optional, deterministic, and safe to disable with `--no-lua`. Outcome: projects can extend validate and `new` behavior via Lua with a stable v1 API.

### Out of Scope 
- Complex sandboxes or networked Lua; advanced security beyond basic environment restriction.
- Multi-file Lua modules resolution and dependency management.
- Rich template rendering; only minimal frontmatter generation here.

### Design/Approach  
- Define stable hook signatures per `contracts/global-conventions.md` and expose `luaApiVersion=1`.
- Validation: call `lua.validate` after core checks; map returned diagnostics to our codes where possible (prefix with `LUA_` when not mappable); merge severity.
- Authoring/new: before file creation, if `id_generator` exists, request an ID; if `render_frontmatter` exists, merge with template/TOML defaults (Lua takes precedence).
- Respect `--no-lua` and environment disable.
- Determinism: no time-based defaults inside Lua unless provided by ctx; ensure same inputs yield same outputs.

### Files to Touch
- `src/commands/validate_cmd.rs`: integrate optional Lua diagnostics pass.
- `src/commands/new.rs`: integrate `id_generator` and `render_frontmatter` before writing file; add `--no-lua` propagation.
- `src/config/lua.rs`: expose an API to call hooks if present.
- `contracts/global-conventions.md`: document exact hook signatures and error mapping.
- Tests: fixtures with simple Lua returning a warning; generator that prefixes IDs.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [x] Conventions: Finalize minimal hook signatures and document under `contracts/global-conventions.md`.
- [x] Lua bridge: Add helpers in `src/config/lua.rs` to load overlay state and expose merged overlay table.
- [x] Validate: Call `lua.validate` if present; merge diagnostics into report and surface in JSON/NDJSON outputs (severity respected).
- [x] New: Call `id_generator` and `render_frontmatter` if present; enforce determinism (no collisions); update file naming accordingly.
- [x] Flags: Ensure `--no-lua` disables all hook calls across commands.
- [x] Tests (validate): Lua warning appears in `validate --format json`, with message/code prefix via `LUA[CODE]: msg` mapping to code.
- [x] Tests (new): With Lua generator, returned ID is used and frontmatter merged; without Lua, behavior unchanged.
kanban_statusline: "validate() + new() hooks wired; --no-lua disables hooks; tests added"

### Acceptance Criteria
**GIVEN** a repo with `.cli-rag.lua` implementing `validate`, **WHEN** running `validate --format json`, **THEN** additional diagnostics from Lua are included with appropriate severity and codes.

**GIVEN** `.cli-rag.lua` implementing `id_generator` and `render_frontmatter`, **WHEN** running `new --schema ADR --title X`, **THEN** the created note uses the Lua-provided ID/frontmatter; without Lua or with `--no-lua`, behavior matches current defaults.

### Issues Encountered 
(to be completed during implementation)

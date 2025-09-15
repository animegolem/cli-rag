---
node_id: AI-IMP-013
tags:
  - IMP-LIST
  - Implementation
  - config
  - versioning
  - info
kanban_status: done
depends_on:
  - ADR-001
  - docs/RAG/BRIDGE_PLAN_V2.md
confidence_score: 0.85
created_date: 2025-09-14
close_date: 2025-09-15
---

# AI-IMP-013-config-versioning-and-exposure

## Summary of Issue #1
Tie reported `configVersion` to a real source of truth in user configuration instead of hardcoding "0.1". Parse `config_version` when present (per contracts examples), provide a sensible default if absent, and expose it in both `info --format json` and `.cli-rag/resolved.json`. Outcome: users and adapters see an accurate configuration version that can evolve alongside Lua overlay/versioning work.

### Out of Scope 
- Implementing Lua overlay itself (AI-IMP-007) and Lua hook semantics (AI-IMP-008).
- Migration tooling for bumping config versions.

### Design/Approach  
- Schema: extend `Config` to optionally include a `config_version: Option<String>` (snake_case in TOML).
- Loader: read `config_version` from top-level TOML (ignore in imported schema files); default to a constant (e.g., "0.1") when missing.
- Info: set `config.version` from the loaded value; remove hardcoded string.
- Resolved snapshot: set `configVersion` from the same value consistently.
- Keep compatibility: absence of the field keeps behavior unchanged.

### Files to Touch
- `src/config/schema.rs`: add `config_version: Option<String>` to `Config`.
- `src/config/loader.rs`: ensure the field is parsed and defaulted.
- `src/commands/info.rs`: source `version` from `cfg.config_version.unwrap_or("0.1".into())`.
- `src/commands/validate_cmd.rs`: source `configVersion` in resolved snapshot from the same value.
- Tests: unit/integration verifying the value flows to both outputs; default when omitted.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [ ] Schema: Add optional `config_version` to `Config` and serde derive.
- [ ] Loader: Preserve and default the version; document precedence (TOML only; overlays may adjust later in AI-IMP-007).
- [ ] Info: Replace hardcoded version string with loaded value.
- [ ] Resolved snapshot: Replace hardcoded `configVersion` with loaded value.
- [ ] Tests: With a temp `.cli-rag.toml` containing `config_version = "1.0"`, assert `info` and `resolved.json` reflect `1.0`.
- [ ] Tests: Without the field, assert default (e.g., `0.1`).

### Acceptance Criteria
**GIVEN** a `.cli-rag.toml` containing `config_version = "1.0"`, **WHEN** running `cli-rag info --format json`, **THEN** the JSON has `config.version == "1.0"` and `.cli-rag/resolved.json.configVersion == "1.0"` after `validate`.

**GIVEN** a `.cli-rag.toml` without `config_version`, **WHEN** running the same commands, **THEN** both surfaces report the default version value and behavior remains unchanged.

### Issues Encountered 
(to be completed during implementation)

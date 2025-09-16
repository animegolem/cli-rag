---
node_id: AI-IMP-014
tags:
  - IMP-LIST
  - Implementation
  - config
  - loader
  - contracts
kanban_status: backlog
depends_on:
  - AI-EPIC-001
confidence_score: 0.82
created_date: 2025-09-15
close_date:
---

# AI-IMP-014-nested-user-config-loader

## Summary of Issue #1
The contracts define a nested user config shape (`[config.scan|authoring|graph|templates]`), while the implementation reads flat top-level keys. Converge the loader to accept the nested shape (and continue to accept flat keys during alpha), mapping to internal `Config` and emitting a consistent `resolved.json`.

### Out of Scope
- Full schema DSL overhaul or migrations; only shape convergence and mapping.

### Design/Approach
- Extend TOML loader to probe for `[config.*]` tables; if present, map nested values to current `Config` fields (e.g., `config.scan.filepaths → bases`, `config.scan.index_path → index_relative`, etc.).
- Preserve existing env/CLI overrides. Keep flat key support for alpha; nested takes precedence when both are present.
- Update resolved snapshot to include camelCase fields unchanged (already aligned to contracts).

### Files to Touch
- `src/config/loader.rs`: parse nested tables; precedence; mapping helpers.
- `src/config/schema.rs`: no changes expected; internal struct remains.
- `tests/`: add integration exercising nested vs flat inputs; precedence; env overrides.
- `.github/workflows/ci.yml`: add a nested config acceptance check (contracts job).

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE>

- [ ] Add nested parse path in loader; map scan/authoring/graph/templates to internal `Config`.
- [ ] Precedence: CLI/env overrides > nested > flat; document in comments.
- [ ] Tests: nested-only config; flat-only config; both present; env `CLI_RAG_FILEPATHS` still works.
- [ ] CI: add step to run validate using a nested-only config and assert resolved snapshot path/fields.

### Acceptance Criteria
GIVEN a `.cli-rag.toml` using only nested `[config.scan]` and `[config.graph]`, WHEN running `cli-rag validate --format json`, THEN no loader errors occur, the unified index is written to the mapped `index_path`, and `.cli-rag/resolved.json` fields match `contracts/v1/config/resolved_config.json`.

### Issues Encountered
{LOC|20}


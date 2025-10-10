---
node_id: AI-IMP-012
tags:
  - IMP-LIST
  - Implementation
  - ci
  - contracts
  - lua
  - ai-index
kanban_status: done
depends_on:
  - AI-IMP-007
  - AI-IMP-009
  - AI-IMP-010
confidence_score: 0.77
created_date: 2025-09-14
close_date: 2025-09-15
---

# AI-IMP-012-ci-gates-for-ai-index-and-lua-smoke

## Summary of Issue #1
Extend CI to validate new surfaces against contracts and prevent regressions. Add jsonschema checks for `ai index plan` and `ai index apply` outputs, and a Lua overlay smoke test that verifies `--no-lua` disables overlays and resolved snapshot reflects overlay provenance. Outcome: CI catches contract drift and overlay regressions early.

### Out of Scope 
- Performance load tests; multi-OS matrix specific to these commands (leverage existing matrix).

### Design/Approach  
- Add steps under the “Contracts Compliance” job to run `ai index plan` and validate JSON against `contracts/v1/cli/ai_index_plan.schema.json`.
- Add steps to run `ai index apply --dry-run` with a small plan; validate JSON against `contracts/v1/cli/ai_index_apply_report.schema.json`.
- Lua: create a minimal `.cli-rag.lua` that sets a sentinel in overlay; run `validate --format json` and check `.cli-rag/resolved.json.overlays.enabled==true`; then run with `--no-lua` and verify disabled.
- Keep steps hermetic using a temp workdir, similar to existing CI fixtures.

### Files to Touch
- `.github/workflows/ci.yml`: add plan/apply schema validations and Lua smoke.
- `scripts/ci-fixtures-check.sh`: optionally mirror new checks for local runs.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [x] CI: Add ai index plan → jsonschema validation step with a tiny graph.
- [x] CI: Add ai index apply (dry-run) → jsonschema validation.
- [x] CI: Add Lua overlay smoke (on/off) and resolved snapshot assertions.
- [ ] Local: Optionally extend `scripts/ci-fixtures-check.sh` to replicate the new gates.

### Acceptance Criteria
**GIVEN** a PR, **WHEN** CI runs, **THEN** `ai index plan` and `ai index apply` steps pass jsonschema validation, and the Lua overlay smoke tests pass (both enabled and `--no-lua`).

### Issues Encountered 
(to be completed during implementation)

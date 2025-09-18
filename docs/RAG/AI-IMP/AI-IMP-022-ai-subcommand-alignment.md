---
node_id: AI-IMP-022
tags:
  - IMP-LIST
  - Implementation
  - ai
  - cli
  - contracts
kanban_status: completed
depends_on:
  - AI-EPIC-003
  - ADR-003d
confidence_score: 0.82
created_date: 2025-09-18
close_date: 2025-09-18
---

# AI-IMP-022-ai-subcommand-alignment

## Summary of Issue #1
The AI command layout is inconsistent: `ai-index-plan` and `ai-index-apply` live at the top level, while `ai new` is nested under the `ai` namespace. This increases cognitive load, complicates help/completions, and diverges from ADR-003d’s intent. Scope: expose `ai index plan` and `ai index apply` under the `ai` namespace, keep the existing top-level commands as soft-deprecated aliases (no behavior change) for one release, and refresh docs/completions. Done state: the CLI presents a coherent `cli-rag ai …` surface; help/README/CI examples use the new form; top-level aliases print a deprecation notice and exit 0. {LOC|20}

### Out of Scope 
- Changing JSON output shapes or exit codes.
- Removing the legacy top-level commands in this iteration (only deprecate).
- Template/prompt parity and output-path keys (tracked separately). {LOC|10}

### Design/Approach  
- Add an `ai index` subcommand with `plan` and `apply` leaf commands. Wire these to the existing implementations used by `ai-index-plan`/`ai-index-apply`.
- Mark `ai-index-plan` and `ai-index-apply` as deprecated: print a single-line warning directing users to `ai index plan|apply` then proceed normally (exit 0, identical behavior/flags/output).
- Update `--help` trees and shell completions so discoverability favors `cli-rag ai …`.
- Update README usage and CI workflow snippets to prefer `ai index …` (keep acceptance schemas unchanged).
- Add integration tests that exercise both the new subcommands and the legacy aliases to lock behavior. {LOC|25}

### Files to Touch
- `src/cli.rs`: add `Ai::Index::{Plan,Apply}` subcommands; mark `AiIndexPlan/AiIndexApply` as deprecated aliases (help text only).
- `src/bin/cli-rag.rs`: route `Ai::Index` to existing `ai_index_plan`/`ai_index_apply` runners; print deprecation messages for legacy commands.
- `src/commands/ai_index_plan.rs`, `src/commands/ai_index_apply.rs`: no behavior change; ensure functions remain reusable from multiple entry points.
- `tests/integration_ai_index_plan.rs`, `tests/integration_ai_index_apply.rs`: add cases for `ai index plan|apply` and keep existing alias tests.
- `README.md`: replace examples with `cli-rag ai index …`.
- `.github/workflows/ci.yml`: switch to `ai index …` invocations; keep schemas the same.
- `src/commands/completions.rs`: regenerate mappings if needed. {LOC|25}

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [x] CLI: add `Ai::Index::{Plan,Apply}` subcommands in `src/cli.rs` with flags mirroring current commands.
- [x] CLI: keep `AiIndexPlan/AiIndexApply` top-level variants but tag help as “deprecated; use ai index …”.
- [x] Bin wiring: route `Ai::Index::Plan` to `commands::ai_index_plan::run` and `Ai::Index::Apply` to `commands::ai_index_apply::run`.
- [x] Deprecation notice: print one-line warning for legacy top-level commands; preserve exit codes and outputs.
- [x] Tests: extend integration tests to call `ai index plan|apply` and assert identical JSON (shape + key fields) vs legacy.
- [x] Tests: ensure exit codes unchanged for success, dry-run, and hash mismatch cases.
- [x] Completions: regenerate to include `ai index` path; verify `--help` tree shows the new hierarchy.
- [x] README: update examples and usage to `ai index …`.
- [x] CI workflow: switch invocations to the new subcommands (schemas unchanged).
- [x] Changelog: note deprecation and new preferred usage.

### Acceptance Criteria
**Scenario:** Using the unified AI namespace for index operations
GIVEN the repository builds `cli-rag`
WHEN a user runs `cli-rag ai index plan --min-cluster-size 2 --output /tmp/plan.json`
THEN the command exits 0 and writes a plan conforming to `contracts/v1/cli/ai_index_plan.schema.json`
AND running `cli-rag ai index apply --from /tmp/plan.json --dry-run` exits 0 and emits a report conforming to `contracts/v1/cli/ai_index_apply_report.schema.json`.

**Scenario:** Legacy aliases remain functional with deprecation warning
GIVEN the same environment
WHEN a user runs `cli-rag ai-index-plan …` or `cli-rag ai-index-apply …`
THEN a single-line deprecation message prints to stderr directing to `ai index …`
AND the JSON outputs and exit codes are identical to the `ai index …` variants.

**Scenario:** Help/completions reflect the new structure
GIVEN the binary is built with completions
WHEN a user runs `cli-rag --help` and `cli-rag ai --help`
THEN the help tree shows `index plan|apply` under `ai`
AND shell completions include `ai index plan` and `ai index apply` subpaths.

### Issues Encountered 
{LOC|20}

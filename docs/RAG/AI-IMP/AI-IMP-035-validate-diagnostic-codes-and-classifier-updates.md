---
node_id: AI-IMP-035
tags:
  - Implementation
  - diagnostics
  - validation
kanban_status: completed
depends_on:
  - AI-IMP-033
  - AI-IMP-034
  - ADR-003d
confidence_score: 0.85
created_date: 2025-10-09
close_date: 2025-10-10
---

# AI-IMP-035-validate-diagnostic-codes-and-classifier-updates

## Summary of Issue
Stabilize and extend `validate` diagnostic codes so downstream tools can rely on predictable codes and messages.
- Add explicit mapping for LINK_MIN_OUT / LINK_MIN_IN, EDGE_REQUIRED_MISSING, EDGE_ID_NOT_FOUND, EDGE_CROSS_SCHEMA_DISALLOWED.
- Keep messages single-line and deterministic.

### Out of Scope
- Changing the validate_result schema; only code/message mapping and message text where needed.

### Design/Approach
- `src/commands/validate_cmd.rs` contains `classify_code` which infers codes from message text. Extend it for the new error strings emitted by AI-IMP-033/034.
- Where useful, adjust producers to prefix messages with canonical tokens to simplify classification (without exceeding 1 line; no quotes gymnastics).
- Verify JSON output against `contracts/v1/cli/validate_result.schema.json`.

### Files to Touch
- `src/commands/validate_cmd.rs`: extend `classify_code` and message parsing for new codes.
- Producers in `src/validate.rs` / `src/validate/schema_rules/apply.rs`: ensure standardized phrasing used by the classifier.
- `src/validate/tests.rs`: add assertions for codes in JSON output.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE>

- [x] Update `classify_code` with explicit branches for LINK_MIN_OUT, LINK_MIN_IN, EDGE_REQUIRED_MISSING, EDGE_ID_NOT_FOUND, EDGE_CROSS_SCHEMA_DISALLOWED.
- [x] Normalize emitting sites to use stable substrings the classifier expects.
- [x] Add tests that call `validate_cmd::run(..., OutputFormat::Json, ...)` and assert `.diagnostics[*].code` contains the expected values.
- [x] Confirm no regressions for existing codes (E200/E220/E212/E213/E214/E231/E240/W250 etc.).

### Acceptance Criteria
**GIVEN** violations produced by AI-IMP-033 and AI-IMP-034
**WHEN** running `cli-rag validate --format json`
**THEN** diagnostics include the new stable codes and match severities per configuration.

### Issues Encountered
- Initial `classify_code` ordering returned legacy code `E220` before the new edge-specific mapping; resolved by moving edge checks ahead of the generic "missing required" branch.
- Cross-schema diagnostics only fire for edge kinds explicitly declared in config; added a no-op `[schema.validate.edges.depends_on]` entry in tests to exercise the classifier path.

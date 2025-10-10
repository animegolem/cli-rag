---
node_id: AI-LOG-003
tags:
  - AI-log
  - diagnostics
  - validation
closed_tickets:
  - AI-IMP-035
created_date: 2025-10-10
related_files:
  - src/commands/validate_cmd.rs
  - tests/integration_validate_json.rs
confidence_score: 0.82
---

# 2025-10-10-LOG-AI-validate-diagnostic-codes

## Work Completed
- Reordered `classify_code` so edge diagnostics map to the new stable codes rather than legacy fallbacks, covering missing-required, unknown-id, and cross-schema violations.
- Extended integration coverage to parse JSON `validate` output, asserting the presence of LINK_MIN_* and EDGE_* codes plus expected severities.

## Tests
- `cargo fmt`
- `cargo test -- --nocapture`

## Follow Ups
- None; ready to move into AI-IMP-036 for docs/default updates.

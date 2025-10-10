---
node_id: AI-LOG-005
tags:
  - AI-log
  - development-summary
  - init
  - templates
  - validation
closed_tickets:
  - AI-IMP-033
  - AI-IMP-035
  - AI-IMP-036
created_date: 2025-10-10
related_files:
  - src/commands/init.rs
  - src/config/template.rs
  - tests/integration_validate_json.rs
  - contracts/v1/config/user_config/templates/ADR.toml
  - contracts/v1/config/user_config/templates/AI-ADR.toml
confidence_score: 0.84
---

# 2025-10-10-LOG-AI-init-adr-wiring

## Work Completed
- Implemented interactive `init` with a Project preset that writes a contracts-aligned `.cli-rag.toml` (docs/RAG defaults) and imports `.cli-rag/templates/ADR.toml`.
- Ported explanatory comments from contracts into the generated `.cli-rag.toml` to support the "ctrl+F and tweak" workflow.
- Added an ADR MVP integration test: initialize preset → create minimal ADR → run `validate --format json`; asserted deterministic diagnostics.
- Drafted AI-IMP-037 for the init preset feature, AI-IMP-038 to wire ADR MVP, and AI-IMP-039 to add the AI-ADR template.
- Confirmed LINK_MIN_* and EDGE_* diagnostics mapping and end-to-end tests pass.

## Session Commits
- Updated `src/commands/init.rs` to implement presets, dry-run/json, overwrite/backup, and to write ADR template.
- Replaced legacy init config constant with contracts-aligned scaffold (`src/config/template.rs`).
- Extended integration coverage in `tests/integration_validate_json.rs` for ADR MVP and existing diagnostics.
- Added AI-ADR contract template file and drafted new implementation tickets under `docs/RAG/AI-IMP/`.

## Issues Encountered
- ADR template required a `node_id` vs. runtime `id` consistency; ensured tests include both fields for now (runtime expects `id`, template comments mention `node_id`).
- Required `depends_on` on ADR led to an expected `EDGE_REQUIRED_MISSING` in MVP; test asserts readable error + isolation warning.
- Needed to reorder classifier branches earlier (captured in AI-IMP-035) to avoid legacy fallbacks overriding new edge codes.

## Tests Added
- ADR MVP integration test (`init_project_preset_and_validate_single_adr`) validating OK shape and stable diagnostics.
- Retained prior validate JSON tests for headings/LOC, numeric bounds, edge/wikilink policies; full suite passes.

## Next Steps
- Implement AI-IMP-039: add AI-ADR template consumption path in tests (import template, `ai new start` returns AI prompt/body).
- After AI-ADR, iteratively add AI-LOG, AI-EPIC, AI-IMP templates and matching tests; keep them off the default preset until ready.
- Begin README refresh once ADR + AI-ADR flows are proven stable; consider surfacing prompts in CLI help.

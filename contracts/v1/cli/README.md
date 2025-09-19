# CLI response schemas (v1)

This directory hosts JSON Schemas for machine‑readable CLI outputs. All `--json` responses should validate against schemas here.

Primary schemas (aligned with current implementation):
- `info.schema.json`
- `validate_result.schema.json`
- `search_result.schema.json`
- `graph.schema.json`
- `path.schema.json`
- `ai_get.schema.json`
- `ai_index_plan.schema.json`
- `ai_index_apply_report.schema.json`
- `ai_new_start.schema.json`
- `ai_new_submit_result.schema.json`
- `ai_new_cancel.schema.json`
- `ai_new_list.schema.json`

Conventions are defined in `contracts/global-conventions.md`.
Contract changes require discussion and an entry in `contracts/changelog.md`.


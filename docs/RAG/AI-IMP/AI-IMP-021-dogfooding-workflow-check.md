---
node_id: AI-IMP-021
id: AI-IMP-021
tags:
  - IMP-LIST
  - dogfooding
kanban_status: backlog
kanban_statusline: Seed the repo with contract-aligned config + templates.
depends_on:
  - AI-EPIC-001
confidence_score: 0.7
created_date: 2025-09-17
close_date:
status: in-progress
---

# AI-IMP-021: Dogfooding Workflow Check

## Summary
Ensure the repository actively uses the nested `.cli-rag.toml`, schema imports, and `cli-rag new` flows so that our dogfooding efforts mirror the CI contracts.

## Acceptance Criteria
- `.cli-rag.toml` migrated to the nested structure and tracked in git.
- ADR/IMP/EPIC templates live under `.cli-rag/templates/` and are exercised via `cli-rag new`.
- `cli-rag validate --format json` runs clean after creating the seed notes.

## Notes
{{LOC|40}}

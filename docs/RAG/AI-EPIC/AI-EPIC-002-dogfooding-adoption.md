---
node_id: AI-EPIC-002
id: AI-EPIC-002
tags:
  - EPIC
  - dogfooding
kanban_status: backlog
date_created: 2025-09-17
date_completed:
AI_IMP_spawned:
  - AI-IMP-021
---

# AI-EPIC-002-dogfooding-adoption

## Problem Statement/Feature Scope
We need to anchor the repository on the contracts-aligned config so that ongoing work uses the same schema and authoring flows that CI enforces.

## Proposed Solution(s)
- Maintain the nested `.cli-rag.toml` and template imports under version control.
- Encourage contributors to rely on `cli-rag new` with the managed schemas for ADR/IMP/EPIC notes.
- Track progress through AI-IMP-021 and follow-up implementation tasks.

## Success Metrics
- All new docs are created via `cli-rag new`.
- `validate --format json` is clean on every branch.
- CI continues to gate the dogfooding flows.

## Requirements
- Migrate config/templates into the repo (`AI-IMP-020`).
- Create initial ADR/IMP/EPIC notes via `cli-rag new`.
- Update Bridge Plan and README to document the dogfooding milestone.

{{LOC|100}}

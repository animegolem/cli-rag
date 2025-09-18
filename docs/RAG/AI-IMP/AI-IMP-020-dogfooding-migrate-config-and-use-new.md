---
node_id: AI-IMP-020
tags:
  - IMP-LIST
  - Implementation
  - dogfooding
  - authoring
kanban_status: backlog
depends_on:
  - AI-EPIC-001
  - AI-IMP-014
  - AI-IMP-015
confidence_score: 0.84
created_date: 2025-09-15
close_date:
---

# AI-IMP-020-dogfooding-migrate-config-and-use-new

## Summary of Issue #1
Adopt the tool in this repo using the new nested config and authoring flows. Migrate `.cli-rag.toml` to nested shape, create schemas for ADR/IMP/EPIC, and use `new` for daily work. Ensure validate is clean and CI passes.

### Out of Scope
- Long-tail template polish; we’ll iterate while using.

### Design/Approach
- Convert local `.cli-rag.toml` to nested shape; adjust paths as needed.
- Create ADR/IMP/EPIC schema TOMLs under `.cli-rag/templates/`; wire import.
- Use `new` to create at least one ADR, one IMP, and one EPIC; validate; run CI.

### Files to Touch
- `.cli-rag.toml` (repo root).
- `.cli-rag/templates/*.toml` (ADR, IMP, EPIC schema definitions).
- Docs: README (example references); Bridge Plans (note dogfooding milestone).

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE>

- [x] Migrate `.cli-rag.toml` to nested shape; run validate.
- [x] Add ADR/IMP/EPIC schemas and import; run validate; update search examples if needed.
- [x] Create initial notes via `new`; ensure validate/search/graph/path remain green.
- [x] Update Bridge Plan with dogfooding status.

### Acceptance Criteria
GIVEN the repo config is nested and schemas are present, WHEN running `validate` and CI, THEN all checks pass; WHEN creating notes via `new`, THEN validate remains ok and new items appear in search.

### Issues Encountered
{LOC|20}

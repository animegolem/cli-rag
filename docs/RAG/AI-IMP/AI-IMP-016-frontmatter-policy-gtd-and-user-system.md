---
node_id: AI-IMP-016
tags:
  - IMP-LIST
  - Implementation
  - validate
  - frontmatter
  - gtd
kanban_status: done
depends_on:
  - AI-EPIC-001
confidence_score: 0.78
created_date: 2025-09-15
close_date: 2025-09-17
---

# AI-IMP-016-frontmatter-policy-gtd-and-user-system

## Summary of Issue #1
Support schema-modeled frontmatter: declare user-controlled fields and a small system GTD subset (kanban_status, kanban_statusline, due_date). Validate presence/shape and surface GTD fields in search outputs when present.

### Out of Scope
- Full system/user taxonomy and arbitrary custom validators; only minimal GTD + user fields.

### Design/Approach
- Extend SchemaCfg to allow `user_frontmatter` (array of strings) and a `[schema.frontmatter.gtd]` table describing allowed values.
- Validate: unknown keys policy remains; ensure declared user fields when required (optional vs required semantics documented).
- Search: include kanban fields if present (already supported); adjust tests for deterministic presence.

### Files to Touch
- `src/config/schema.rs`: add optional user/system frontmatter model (minimal subset).
- `src/validate/rules.rs`: enforce presence/shape (enum for kanban_status; bool/string for statusline; date for due_date).
- `tests/`: integration tests covering validate and search GTD emitters.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE>

- [x] Schema: add user_frontmatter and frontmatter.gtd subset to SchemaCfg.
- [x] Validate: enforce shapes and optional presence per schema.
- [x] Tests: validate errors for wrong shapes and acceptance for correct ones.
- [x] Search tests: confirm kanban fields appear when present.

### Acceptance Criteria
GIVEN a schema declaring `frontmatter.gtd.kanban_status=[...]` WHEN a note contains an invalid status, THEN `validate` emits an error; WHEN valid, THEN `search --kanban` reflects the values in results.

### Issues Encountered
{LOC|20}

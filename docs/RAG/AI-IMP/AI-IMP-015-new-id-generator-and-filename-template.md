---
node_id: AI-IMP-015
tags:
  - IMP-LIST
  - Implementation
  - authoring
  - new
  - templates
kanban_status: done
depends_on:
  - AI-EPIC-001
confidence_score: 0.8
created_date: 2025-09-15
close_date: 2025-09-16
---

# AI-IMP-015-new-id-generator-and-filename-template

## Summary of Issue #1
Implement `new` authoring features required by contracts/templates: id generation strategies (increment, datetime, uuid) and `filename_template` interpolation with common filters. Outcome: `cli-rag new --schema ADR --title X` emits deterministic filenames and IDs per schema config.

### Out of Scope
- Advanced template prompts and rich body rendering (keep minimal).

### Design/Approach
- Extend schema config to accept `id_generator { strategy, prefix?, padding? }` and `filename_template`.
- Implement strategies:
  - increment: scan index or filesystem to compute next N with optional prefix and zero padding.
  - datetime: use now UTC with a fixed format.
  - uuid: v4.
- Template filters: kebab-case, snake_case, PascalCase, camelCase, SCREAMING_SNAKE_CASE, date:"%Y-%m-%d" on `now`.

### Files to Touch
- `src/commands/new.rs`: wire generation and template rendering.
- `src/config/schema.rs`: add optional fields to SchemaCfg as needed (filename_template); id_generator may be parsed via a nested struct.
- `tests/`: integration tests for each strategy and template filters.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE>

- [x] Parse id_generator and filename_template from schema.
- [x] Implement strategies and next-ID logic (increment considers existing IDs in index or files).
- [x] Implement template filters with deterministic behavior; unit-test them.
- [x] Integration: verify produced path and ID; ensure validate passes after writing.

### Acceptance Criteria
GIVEN a schema with `id_generator={strategy="increment",prefix="ADR-",padding=3}` and `filename_template="{{id}}-{{title|kebab-case}}.md"`, WHEN running `new --schema ADR --title "Circuit Breaker"`, THEN a file `ADR-XYZ-circuit-breaker.md` is created with ID assigned and validate succeeds.

### Issues Encountered
{LOC|20}

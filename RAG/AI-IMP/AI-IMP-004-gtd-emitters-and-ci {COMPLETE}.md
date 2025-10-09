---
node_id: AI-IMP-004
tags:
  - IMP-LIST
  - Implementation
  - search
  - ai_get
  - ci
kanban_status: done
confidence_score: 0.78
created_date: 2025-09-12
---

# AI-IMP-004-gtd-emitters-and-ci

## Sub-Issue #1: Emit enriched TODO fields in search
Populate `dueDate`, `source`, `span`, and `priorityScore` for `kind=todo` results; add `kanbanStatusLine` to `kind=note` when known.

### Files to Touch
- `src/commands/search.rs` (emission logic)
- `src/index.rs` or extraction path (if needed for spans)
- `contracts/global-conventions.md` (ensure span semantics referenced)
- `tests/integration_search_gtd.rs` (new)

### Implementation Checklist
- [x] Map TODO rank to `priorityScore` 1–10 (1–100 → 1–10; low/medium/high/urgent mapping).
- [x] Detect `dueDate` from GTD box body; set `source` accordingly.
- [x] Include `span` [start,end) in byte offsets when body-derived.
- [x] Emit optional `kanbanStatusLine` for `kind=note` when present in FM.

## Sub-Issue #2: ai_get GTD context and neighbors
Expose optional GTD fields on root and neighbors per contracts.

### Files Touched
- `src/commands/get.rs` (ai_get builder)
- CI validator for ai_get JSON

### Implementation Checklist
- [x] Root: include `kanbanStatus`, `kanbanStatusLine`, `dueDate` when present in FM.
- [x] Neighbors: include `kanbanStatus`, `kanbanStatusLine`, `dueDate` when derivable.

## Sub-Issue #3: CI validators
Add schema checks for the updated contracts once emitters land (optional fields remain optional).

### Files Touched
- `.github/workflows/ci.yml`

### Implementation Checklist
- [x] Validate `search --kind kanban,todo --format json` payloads against `contracts/v1/cli/search_result.schema.json`.
- [x] Validate `ai get --format json` payloads against `contracts/v1/cli/ai_get.schema.json`.

### Acceptance Criteria
CI passes with new validators; integration tests demonstrate presence of fields when available and absence when not. Completed.

## Issues Encountered
N/A at ticket creation.

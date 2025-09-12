---
node_id: AI-IMP-004
tags:
  - IMP-LIST
  - Implementation
  - search
  - ai_get
  - ci
kanban_status: backlog
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
- [ ] Map TODO@N to `priorityScore` 1–10 (Lua policy or parser helper).
- [ ] Detect `dueDate` from body or frontmatter; set `source` accordingly.
- [ ] Include `span` [start,end] when body-derived.
- [ ] Emit optional `kanbanStatusLine` for `kind=note` when present in FM.

## Sub-Issue #2: ai_get GTD context and neighbors
Expose optional GTD fields on root and neighbors per contracts.

### Files to Touch
- `src/commands/get.rs` (ai_get builder)
- `tests/integration_ai_get_gtd.rs` (new)

### Implementation Checklist
- [ ] Root: include `kanbanStatus`, `kanbanStatusLine`, `dueDate` when present in FM.
- [ ] Neighbors: include `kanbanStatus`, `dueDate` when derivable.

## Sub-Issue #3: CI validators
Add schema checks for the updated contracts once emitters land (optional fields remain optional).

### Files to Touch
- `.github/workflows/ci.yml`

### Implementation Checklist
- [ ] Validate `search --json --todo/--kanban` payloads against `contracts/v0.1/cli/search_result.schema.json`.
- [ ] Validate `ai get --json` payloads against `contracts/v0.1/cli/ai_get.schema.json`.

### Acceptance Criteria
CI passes with new validators; integration tests demonstrate presence of fields when available and absence when not.

## Issues Encountered
N/A at ticket creation.


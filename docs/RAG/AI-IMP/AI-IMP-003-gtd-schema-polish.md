---
node_id: AI-IMP-003
tags:
  - IMP-LIST
  - Implementation
  - contracts
  - gtd
kanban_status: backlog
confidence_score: 0.82
created_date: 2025-09-12
---

# AI-IMP-003-gtd-schema-polish

## Sub-Issue #1: Contracts patch (GTD fields)
Update contracts to normalize kanban casing and enrich TODO/ai_get shapes.

### Files to Touch
- `contracts/v0.1/cli/search_result.schema.json`
- `contracts/v0.1/cli/ai_get.schema.json`
- `contracts/v0.1/cli/info.schema.json`
- `contracts/global-conventions.md`
- `contracts/changelog.md`

### Implementation Checklist
- [ ] search_result: add `kanbanStatusLine?` on note; ensure kanban item uses `kanbanStatusLine` casing.
- [ ] search_result (todo): add `dueDate?`, `source? (body|frontmatter)`, `span? [start,end]`, `priorityScore? (1–10)`.
- [ ] ai_get (neighbors): add optional `kanbanStatus`, `dueDate`.
- [ ] ai_get (root): add optional `kanbanStatus`, `kanbanStatusLine`, `dueDate`.
- [ ] info (capabilities): add optional `gtdTasks`, `kanban` booleans.
- [ ] conventions: document kebab-case CLI flags; keep JSON camelCase, TOML/Lua/frontmatter snake_case.
- [ ] Update `contracts/changelog.md` with entry for this change.

### Acceptance Criteria
`jsonschema` validates example payloads reflecting the new optional fields; existing payloads without new fields still validate.

## Issues Encountered
N/A at ticket creation.


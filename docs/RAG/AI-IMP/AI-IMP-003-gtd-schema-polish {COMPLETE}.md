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
- `contracts/v1/cli/search_result.schema.json`
- `contracts/v1/cli/ai_get.schema.json`
- `contracts/v1/cli/info.schema.json`
- `contracts/global-conventions.md`
- `contracts/changelog.md`

### Implementation Checklist
- [x] search_result: add `kanbanStatusLine?` on note; ensure kanban item uses `kanbanStatusLine` casing.
- [x] search_result (todo): add `dueDate?`, `source? (body|frontmatter)`, `span? [start,end]`, `priorityScore? (1–10)`.
- [x] ai_get (neighbors): add optional `kanbanStatus`, `dueDate`.
- [x] ai_get (root): add optional `kanbanStatus`, `kanbanStatusLine`, `dueDate`.
- [x] info (capabilities): add optional `gtdTasks`, `kanban` booleans.
- [x] conventions: document kebab-case CLI flags; keep JSON camelCase, TOML/Lua/frontmatter snake_case.
- [x] Update `contracts/changelog.md` with entry for this change.

### Acceptance Criteria
`jsonschema` validates example payloads reflecting the new optional fields; existing payloads without new fields still validate.

## Issues Encountered
- Search envelope alignment required updating existing tests to use `{results: [...]}` rather than a bare array.
- Clippy flagged `too_many_arguments` on `search::run`; added a targeted allow to keep the handler signature simple while filters are experimental.
- Rustfmt preferred multi-line formatting for chained expressions and assertions; adjusted code and tests to satisfy pre-commit checks.
- CI schema paths standardized under `contracts/v1/...` to match the v1 contracts folder.

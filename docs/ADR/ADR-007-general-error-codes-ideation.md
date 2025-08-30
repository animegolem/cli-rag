---
id: ADR-007
tags:
  - errors
  - error_codes
status: draft
depends_on: ADR-001, AD-003b
created_date: 2025-08-27
last_modified: 2025-08-27
related_files: []
---

# general-error-codes

## Objective
<!-- A concise statement explaining the goal of this decision. -->

A parking lot for error code ideas as they come up in llm chats or otherwise 

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

```
1. **File matches multiple schemas**

   * e.g., `docs/ADR-024.md` matches both `["ADR-*.md"]` and `["**/*.md"]`.
   * **Policy:** default = **error** (simple & safe). Optional future knob: `schema.priority` (higher wins).
   * Error: `E200: File docs/ADR-024.md matched multiple schemas: ADR, CatchAll. Disambiguate or set priority.`

2. **Duplicate IDs across files** (when `identity_source = frontmatter`)

   * Two notes both claim `id: ADR-024`.
   * **Policy:** **error** (authoritative ID must be unique).
   * Error: `E210: Duplicate id ADR-024 in docs/ADR-024.md and notes/ADR-024.md.`

3. **Missing/invalid ID for ID-managed schemas**

   * **Policy:** **error** if schema requires `id`; otherwise warn/skip.
   * Error: `E220: Missing required frontmatter key 'id' for schema ADR in docs/ADR-XXX.md.`

4. **Unresolved references** (`depends_on` points to unknown id)

   * **Policy:** choose severity per schema (`error|warning`).
   * Error: `E230: depends_on → ADR-999 not found (referenced by ADR-024).`

5. **Multiple schemas apply to the same note type accidentally** (design-time)

   * This is really a variant of #1; your strict error covers it.

6. **Cycles in the graph** (if disallowed)

   * **Policy:** usually warn, optionally error if schema says DAG only.
   * Error: `E240: Cycle detected ADR-001 → ADR-007 → ADR-001 (schema ADR requires DAG).`

7. **File outside declared scan roots**

   * **Policy:** warn or ignore, your call.
   * Error: `E250: File notes/ADR-024.md is outside [scan.filepaths].`
```

```
### 3. Template Variable Conflicts

What happens if someone names a frontmatter field "title" or "id"? You might need a precedence rule or namespace separation.
## Missing Pieces

1. **Conflict Resolution**: If multiple schemas match the same file pattern, which wins?
```

error on conflict, explain the merge error in stderr

by the opposite hand successes should just be silent unless we have a very good reason. fire writes "this will update x notes (list) y/n " is probably sane when we add file operations. 



```
## Template Variable Collision Still Unaddressed

Your AI-MEM template uses `{{title}}` but what if the AI sets a frontmatter field called "title"? You need explicit precedence:

```toml
[schema.new.template]
variable_precedence = ["system", "frontmatter", "computed"]
```

worth thinking through if a conflict is just an error. less overhead on the user frankly to just get "change x" vs setting inheritance logic. 

## Decision
<!-- What is the change that we're proposing and/or doing? -->

Accept the above at least in principal and work to expand the list. 

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

A more predictable and informative cli. 

## Updates
<!-- Changes that happened when the rubber met the road -->


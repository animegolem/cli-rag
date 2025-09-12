# Contracts Change Log

## 2025-09-12: Add protocolVersion to graph/path

### Reason for change
- Align with global convention: all top-level JSON responses include `protocolVersion`.

### Overview of change
- cli/graph.schema.json: require `protocolVersion` (integer, min 1) at top-level.
- cli/path.schema.json: require `protocolVersion` (integer, min 1) at top-level.
- Implementations updated to emit `protocolVersion` in outputs.

## 2025-09-12: GTD schema polish and casing normalization

### Reason for change
- Reduce ambiguity around kanban field casing across surfaces.
- Enrich TODO items to support Agenda-like views without over-specifying UI semantics.
- Expose minimal GTD context in ai_get and advertise capabilities for UI adapters.
- Formalize CLI flag casing to avoid drift.

### Overview of change
- search_result.schema.json
  - note.kind=note: add optional `kanbanStatusLine`.
  - note.kind=kanban: standardize `kanbanStatusLine` casing (was sometimes `kanbanStatusline` in docs).
  - todo.kind: add optional `dueDate` (date), `source` ("body"|"frontmatter"), `span` ([start,end]), `priorityScore` (1–10).
- ai_get.schema.json
  - neighbors[]: add optional `kanbanStatus` and `dueDate`.
  - root: add optional `kanbanStatus` and `kanbanStatusLine` (high-level descriptor context) and optional `dueDate`.
- info.schema.json
  - capabilities: allow optional `gtdTasks` and `kanban` booleans.
- global-conventions.md
  - Document that CLI long flags use kebab-case (e.g., `--graph-format`).

Notes: These additions are optional fields; the only casing normalization is aligning on `kanbanStatusLine`. Contracts remain the source of truth.

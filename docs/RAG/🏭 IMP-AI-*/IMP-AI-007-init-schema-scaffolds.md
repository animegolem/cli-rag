---
node_id: IMP-AI-007
tags:
  - cli
  - init
  - templates
kanban_status:
  - planned
kanban_statusline: Add init --schema scaffolding with optional --separate to templates/.
depends_on:
  - ADR-003c
  - ADR-001
blocked_by: []
created_date: 2025-08-30
related_files:
  - src/commands/init.rs
  - src/config.rs
---

# IMP-AI-007-init-schema-scaffolds

### **Goal**
Add `init --schema <NAME>` with optional `--separate` to scaffold `[[schema]]` blocks either inline in `.adr-rag.toml` or in `.adr-rag/templates/<NAME>.toml` and wire `import` automatically.

### **Context**
Accelerating first-time setup reduces friction and ensures new projects follow the current config DSL. ADR-003c specifies UX and flags; ADR-001 provides schema examples and comments that we can reuse for the scaffold content.

### **Implementation Plan**
- [ ] CLI: extend `init` parser to accept `--schema <NAME>` and `--separate`.
- [ ] Inline mode: append a commented `[[schema]]` template block to `.adr-rag.toml` (non-destructive append; keeps existing content).
- [ ] Separate mode: write `.adr-rag/templates/<NAME>.toml` with the block; update `.adr-rag.toml` to add the import if missing.
- [ ] Content: include `config_version`, `name`, `file_patterns`, and commented validator knobs; include a minimal note template stub.
- [ ] Tests: both modes create files correctly; idempotent re-run; import list accuracy.

### **Acceptance Criteria**
- Running `adr-rag init --schema ADR` appends a commented ADR schema to `.adr-rag.toml` without overwriting existing content.
- Running with `--separate` writes a template file and updates `import` accordingly.
- Generated scaffolds include `config_version` and pass `validate` after minimal edits.

### **Takeaway**
To be completed upon ticket closure (notes on defaults and additional templates to ship).

---
node_id: IMP-AI-003
tags:
  - config
  - imports
kanban_status:
  - completed
kanban_statusline: Imports supported; schemas-only enforced; tests green.
depends_on:
  - ADR-001
  - ADR-006
blocked_by: []
created_date: 2025-08-30
related_files:
  - src/config.rs
---

# IMP-AI-003-schema-imports-support

### **Goal**
Support `import = ["..."]` for external schema files and enforce that imports define only `[[schema]]` blocks; merge schemas deterministically.

### **Context**
Large repos benefit from splitting schema definitions into separate files. ADR-006 prescribes strict import rules to avoid configuration creep. We need reliable import resolution, clear errors for illegal keys (E110), and duplicate-name detection (E120).

### **Implementation Plan**
- [x] Add `import = ["path/to/file.toml", ...]` to top-level config; resolve relative to the config file’s directory.
- [x] For each import, parse TOML and assert only `[[schema]]` tables exist; return E110 with offending top-level keys if violated.
- [x] Concatenate schemas from project and imports; enforce unique `name` across the combined set (E120).
- [x] Order of precedence: project-defined schemas first, then imports in list order (documented).
- [x] Tests: valid import, illegal key failure, duplicate-name failure, and relative/absolute path resolution.

### **Acceptance Criteria**
- A config with `import` entries loads successfully and exposes all schemas (unit test).
- Imports containing keys other than `[[schema]]` produce E110 with source file and key names (unit test).
- Duplicate schema names across project/imports produce E120 (unit test).
- Behavior validated by unit tests (cargo test all green).

### **Takeaway**
Import resolution supports relative and absolute paths; added absolute-path fallback when glob expansion yields no matches. E110 reports illegal top-level keys with file path, and E120 catches duplicate schema names post-merge. Tests isolate work under `tmp/` to avoid interference.

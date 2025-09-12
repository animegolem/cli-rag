---
node_id: IMP-AI-009
tags:
  - refactor
  - config
  - modularity
kanban_status:
  - completed
kanban_statusline: Completed — config split into loader/schema/defaults/template with stable facade.
depends_on:
  - ADR-001
  - IMP-AI-006
blocked_by: []
created_date: 2025-08-31
related_files:
  - src/config/
  - src/cli.rs
---

# IMP-AI-009-split-config-module

### **Goal**
Extract `src/config.rs` into smaller, focused modules (loader/discovery, schema/types, defaults/constants, template emitter) to improve readability and maintenance, without changing behavior.

### **Context**
The config module has grown beyond 600 lines as we added versioning hooks, import handling, discovery, defaults, and templates. With the unified index groundwork (IMP-AI-006), it’s a natural time to reduce surface area and make future changes safer (e.g., deprecation paths, versioned parsing).

### **Implementation Plan**
- [x] Create `src/config/` directory and move logic behind a `mod` facade (`src/config/mod.rs`).
- [x] `loader.rs`: `find_config_upwards`, `load_config`, env processing, import resolution, invariants.
- [x] `schema.rs`: `SchemaCfg`, `SchemaRule`, `DefaultsCfg`, serde derives, helpers.
- [x] `defaults.rs`: default_* functions (file patterns, statuses, ignore globs, etc.).
- [x] `template.rs`: `TEMPLATE` and `write_template`.
- [x] Maintain existing public API in `config` (reexport types and functions), so other modules remain unchanged.
- [x] No behavior changes; ensure tests pass unchanged.

### **Acceptance Criteria**
- Code compiles; unit/integration tests pass without modifications. ✅
- Monolith replaced; `src/config/*` under 350 LOC each. ✅
- Public API compatibility maintained (no changes to callers). ✅

### **Takeaway**
Cleaner boundaries in config make future work (versioned parsing, upgrade helpers, and deprecation messaging) easier and reduce merge churn.

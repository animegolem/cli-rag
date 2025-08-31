---
node_id: IMP-AI-009
tags:
  - refactor
  - config
  - modularity
kanban_status:
  - planned
kanban_statusline: Split config loader/template/constants into focused submodules post-unified-index.
depends_on:
  - ADR-001
  - IMP-AI-006
blocked_by: []
created_date: 2025-08-31
related_files:
  - src/config.rs
  - src/cli.rs
---

# IMP-AI-009-split-config-module

### **Goal**
Extract `src/config.rs` into smaller, focused modules (loader/discovery, schema/types, defaults/constants, template emitter) to improve readability and maintenance, without changing behavior.

### **Context**
The config module has grown beyond 600 lines as we added versioning hooks, import handling, discovery, defaults, and templates. With the unified index groundwork (IMP-AI-006), it’s a natural time to reduce surface area and make future changes safer (e.g., deprecation paths, versioned parsing).

### **Implementation Plan**
- [ ] Create `src/config/` directory and move logic behind a `mod` facade (`src/config/mod.rs`).
- [ ] `loader.rs`: `find_config_upwards`, `load_config`, env processing, import resolution, invariants.
- [ ] `schema.rs`: `SchemaCfg`, `SchemaRule`, `DefaultsCfg`, serde derives, helpers.
- [ ] `defaults.rs`: default_* functions (file patterns, statuses, ignore globs, etc.).
- [ ] `template.rs`: `TEMPLATE` and `write_template`.
- [ ] Maintain existing public API in `config` (reexport types and functions), so other modules remain unchanged.
- [ ] No behavior changes; ensure tests pass unchanged.

### **Acceptance Criteria**
- Code compiles; unit/integration tests pass without modifications.
- `src/config.rs` size reduced substantially by delegating to submodules.
- Public API compatibility maintained (no changes to callers).

### **Takeaway**
Cleaner boundaries in config make future work (versioned parsing, upgrade helpers, and deprecation messaging) easier and reduce merge churn.


---
node_id: IMP-AI-001
tags:
  - config
  - loader
  - errors
kanban_status:
  - completed
kanban_statusline: Enforce single top-level config and unique schema names with clear errors.
depends_on:
  - ADR-001
  - ADR-006
blocked_by: []
created_date: 2025-08-30
related_files:
  - src/config.rs
  - src/commands/doctor.rs
---

# IMP-AI-001-config-loader-invariants

### **Goal**
Enforce a single top-level `.adr-rag.toml` per repo and ensure all `[[schema]]` names are unique across project and imports, with deterministic discovery and clear, machine-readable errors.

### **Context**
Multiple project configs or duplicate schema names lead to ambiguous behavior and hard-to-debug validation. ADR-006 specifies strict invariants (E100/E110/E120) to keep configuration simple and safe. Locking this down early stabilizes downstream features (validation, indexing, watch, NVIM/MCP).

### **Implementation Plan**
- [x] Discovery: detect multiple `.adr-rag.toml` files in the upward chain; if more than one is found (and no explicit `--config`/`ADR_RAG_CONFIG` is set) fail with E100 listing the paths.
- [x] Duplicates: after reading project (+imports future), enforce unique `[[schema]]` names; raise E120 on duplicates.
- [x] Doctor/Status: include invariant checks (`invariants_ok`, `invariants_errors`) in JSON/plain output.
- [x] Warnings: when `init` creates a child config beneath an existing parent, warn about shadowing.
- [x] Tests: fixtures for multiple-config trees (E100) and duplicate schema names across imports (E120).

### **Acceptance Criteria**
- Running any command with two project configs in scope exits non-zero with E100 and prints the conflicting paths.
- Duplicate schema names across project/imports exit with E120, listing the colliding names and files.
- `doctor --json` includes `config`, `invariants_ok`, and any `errors:[{code,..}]` when violations exist.
- `init` prints a shadowing warning when appropriate; no silent overrides occur.

### **Takeaway**
E100 triggers early before parse when multiple configs are found in the upward chain (unless explicit path/env override). E120 surfaces duplicate schema names with a clear message. Doctor now exposes invariants status. Tests create isolated dirs under `tmp/` and assert both errors without relying on external state.

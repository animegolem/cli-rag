---
node_id: IMP-AI-004
tags:
  - validation
  - errors
  - graph
kanban_status:
  - completed
kanban_statusline: Completed — unified initial error codes; multi-schema E200; per-schema cycle policy; JSON/NDJSON codes + locations.
depends_on:
  - ADR-006
  - ADR-007
blocked_by: []
created_date: 2025-08-30
related_files:
  - src/validate.rs
  - src/commands/validate_cmd.rs
---

# IMP-AI-004-validator-error-codes-and-policies

### **Goal**
Unify validator error codes and JSON shapes, add multi-schema match detection (E200), and implement per-schema cycle detection policy (warn|error) with clear diagnostics.

### **Context**
Downstream tooling (NVIM, MCP, CI) relies on stable, machine-readable diagnostics. Today, messages are plain strings. We need a consistent `{severity, code, msg, file?, span?, field?}` shape and new detection for ambiguous schema matches and cycles, as outlined in ADR-006/007.

### **Implementation Plan**
- [x] Initial error code mapping in `validate --json|--ndjson` (E200, E212, E213, E214, E220, E221/W221, E225, E230, E231, E240/W240, W250).
- [x] Multi-schema detection by filename patterns; emit E200 with list of schema names.
- [x] Cycle detection over `depends_on`; add per-schema `cycle_policy = "warn"|"error"|"ignore"` and emit W/E240 accordingly.
- [x] Attach optional `{path,line?}` locations to issues where derivable.
- [x] Tests: cycle policy warn vs error; get --format ai integration test from ADR-AI-004 complements surfaces.

### **Acceptance Criteria**
- `validate --json|--ndjson` emits issues with `code` for common error/warn cases. ✅
- Overlapping schema patterns for a file produce E200 with matching schema names. ✅
- Cycle detection produces W/E240 and honors per-schema policy. ✅
- Issues include optional `{path,line?}` when available. ✅
- All tests green; plain output preserved. ✅

### **Takeaway**
- Introduced per-schema `cycle_policy` without breaking existing configs (defaults warn).
- Established initial error code mapping; can evolve incrementally and expand coverage.
- Locations in issues and path hops improve editor alignment for navigation.

### **Takeaway**
To be completed upon ticket closure (document final code table and examples).

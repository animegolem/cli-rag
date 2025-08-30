---
node_id: IMP-AI-004
tags:
  - validation
  - errors
  - graph
kanban_status:
  - planned
kanban_statusline: Unify error codes; add multi-schema match (E200) and cycle policy wiring.
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
- [ ] Define error code table (docstring/enum), e.g., E100/E110/E120 (config), E200 (multi-schema match), E220 (missing required), E230 (bad reference), E240 (cycle), etc.
- [ ] Update `validate_cmd` to emit issues with codes and optional fields; keep plain mode readable.
- [ ] Multi-schema detection: when building schema matches, compute all matches for each filename; if >1, record E200 for that file (and optionally exclude from further schema-specific checks).
- [ ] Cycle detection: build graph (initially over `depends_on`), perform DFS/BFS to find cycles; report per-schema policy (warn|error) with involved path `A → B → A`.
- [ ] Tests: E200 triggered by overlapping patterns, cycle reported with correct severity and shape, regression for existing messages.

### **Acceptance Criteria**
- `validate --json|--ndjson` emits issues with `code` populated for all errors and relevant warnings.
- Overlapping schema patterns for a file produce E200 with the list of matching schemas.
- Cycle detection produces E240 with the cycle path and honors schema policy (warn vs error).
- All tests green; plain mode retains clear, human-readable messages.

### **Takeaway**
To be completed upon ticket closure (document final code table and examples).

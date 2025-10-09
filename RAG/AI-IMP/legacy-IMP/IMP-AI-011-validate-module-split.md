---
node_id: IMP-AI-011
tags:
  - refactor
  - validation
  - modularity
kanban_status:
  - planned
kanban_statusline: Split validation into focused submodules without behavior changes.
depends_on:
  - IMP-AI-009
  - ADR-006
blocked_by: []
created_date: 2025-08-31
related_files:
  - src/validate.rs
  - src/commands/validate_cmd.rs
---

# IMP-AI-011-validate-module-split

### Goal
Reduce `src/validate.rs` (~600 LOC) into smaller, focused modules to improve readability and pave the way for error-code/cycle-policy work, without changing behavior or outputs.

### Context
Validation handles multiple concerns: ids/duplicates/conflicts, reference checks, schema assignment and unknown-key policy, rule validation (array/date/regex/min_items/allowed), and isolation warnings. Consolidation makes it hard to evolve (IMP-AI-004). Following the module split pattern from config and discovery will de-risk upcoming features.

### Implementation Plan
- [ ] Create `src/validate/` with a `mod.rs` facade re-exporting `validate_docs` and `ValidationReport`.
- [ ] `report.rs`: define `ValidationReport` and helpers to accumulate errors/warnings consistently.
- [ ] `schema_match.rs`: pre-compile schema globsets, assign schema per doc; surface first-match mapping (feature parity).
- [ ] `ids.rs`: id presence, duplicate/conflict detection and formatting.
- [ ] `refs.rs`: existence checks for `depends_on`/`supersedes`/`superseded_by`.
- [ ] `rules.rs`: required keys, unknown policy, and per-field rule checks (type=array/date, allowed values, regex, min_items, refers_to_types).
- [ ] `isolation.rs`: isolated/orphan warnings (no edges) logic.
- [ ] Keep signatures and messages stable; update `use` paths in callers.

### Acceptance Criteria
- Code compiles; all unit/integration tests pass unchanged.
- Public API unchanged: `validate_docs(cfg, &docs) -> ValidationReport`.
- No behavior/output changes in plain/JSON/NDJSON modes.
- Each new module stays under 350 LOC; `src/validate.rs` becomes a thin facade (<=100 LOC).

### Takeaway
This sets up a clean surface for IMP-AI-004 (codes, multi-schema E200, cycle policy) by isolating concerns and reducing merge pressure.


---
node_id: IMP-AI-005
tags:
  - validation
  - frontmatter
  - edges
  - templates
kanban_status:
  - planned
kanban_statusline: Implement frontmatter/body/edges validator knobs per ADR-001 and ADR-AI-003.
depends_on:
  - ADR-001
  - ADR-AI-003
  - ADR-011
blocked_by: []
created_date: 2025-08-30
related_files:
  - src/validate.rs
  - src/model.rs
  - src/config.rs
---

# IMP-AI-005-validator-knobs-frontmatter-body-edges

### **Goal**
Implement declarative validator knobs for frontmatter, body, and edges per ADR-001/ADR-AI-003, enabling richer policies without bespoke code.

### **Context**
We want schemas to describe validation rules (allow_unknown, regex/range/type, heading enforcement, line-count caps, edge policies) so new note types can be added without modifying core code. This also lays groundwork for NVIM linting and `new` generation contracts.

### **Implementation Plan**
- [ ] Frontmatter:
  - [ ] Implement `allow_unknown` (bool + severity override) and field-level rules: `regex`, `integer{min,max}`, `float{min,max}`, `enum`.
  - [ ] Example defaults: enforce `tags` one-line regex; `related_files` extensions via regex array.
- [ ] Body:
  - [ ] Implement `heading_check` modes: `exact`, `missing_only`, `ignore` comparing to template headings.
  - [ ] Implement `line_count.scan_policy` honoring `{{LOC|N}}` markers: `on_creation` (checked in `new`) and `on_validate` (checked in `validate`).
- [ ] Edges:
  - [ ] Support `graph_edges = ["depends_on", ...]` in schema config; auto-validate refs for all listed fields.
  - [ ] Add `required_edges`, `detect_cycles` toggle, and `wikilinks.min_outgoing|min_incoming|severity` checks.
  - [ ] Add `cross_schema.allowed_targets` for edge targets; error/warn when violated.
- [ ] Tests: FM regex/enum, heading modes, LOC violations, required_edges missing, cross-schema target violation.

### **Acceptance Criteria**
- Declarative knobs are parsed and enforced; violations appear with codes/severity in `validate --json`.
- A sample schema using each knob fails/passes as expected under tests.
- Body checks honor configured modes and LOC policies in both `new` and `validate` pathways.

### **Takeaway**
To be completed upon ticket closure (notes on ergonomics and defaults).

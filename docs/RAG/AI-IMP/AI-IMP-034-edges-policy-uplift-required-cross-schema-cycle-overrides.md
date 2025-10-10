---
node_id: AI-IMP-034
tags:
  - Implementation
  - validation
  - edges
kanban_status: planned
depends_on:
  - AI-EPIC-XXX-cli-v1-authoring-contracts
  - ADR-003d
confidence_score: 0.8
created_date: 2025-10-09
close_date:
---

# AI-IMP-034-edges-policy-uplift-required-cross-schema-cycle-overrides

## Summary of Issue
Implement FR-10: add schema-driven edges policy and enforce it during `validate`.
- Config surface under `schema.validate.edges`:
  - Per-edge: `{ weight, required, cycle_detection }` for edge kinds like `depends_on`, `blocked_by`, `implements`, `supersedes`, `superseded_by`.
  - Cross-schema whitelist: `cross_schema.allowed_targets = ["ADR", ...]`.
- Validation behaviors:
  - Required: the FM field exists (string or array) and all referenced IDs exist.
  - Cross-schema: each referenced ID’s schema is in allowedTargets (if configured), for all configured edge kinds.
  - Cycle overrides: for `depends_on`, override schema-level cycle policy with the per-edge `cycle_detection` when set.

### Out of Scope
- Introducing new edge extraction beyond existing FM keys (no additional parsing in body).
- Changing cycle detection algorithm (only severity policy overrides).

### Design/Approach
- Config types:
  - Extend `SchemaValidateCfg` with `edges`: per-edge map + `cross_schema` + `wikilinks` (the latter is used by AI-IMP-033).
  - Use serde to deserialize snake_case TOML → Rust structs; no loader special cases needed beyond type additions.
- Required logic:
  - Accept scalar string or array in FM. Normalize to Vec<String>.
  - Empty array or empty string counts as missing (violation) when `required` != "ignore".
  - For each ID, check existence in `id_to_docs`.
- Cross-schema whitelist:
  - Build `doc_schema` map (already present) and verify target schema ∈ allowedTargets.
  - Apply to all edge kinds present under `validate.edges`.
- Cycle overrides:
  - When computing cycle diagnostics in `validate.rs`, read per-edge cycle policy for `depends_on` and prefer it over `schema.cycle_policy`.
- Diagnostics (single-line, deterministic):
  - EDGE_REQUIRED_MISSING (severity from per-edge required)
  - EDGE_ID_NOT_FOUND (severity from per-edge required)
  - EDGE_CROSS_SCHEMA_DISALLOWED (severity = error; or inherit per-edge required? Use `schema.validate.severity` fallback if not specified)

### Files to Touch
- `src/config/schema.rs`: add `SchemaEdgesCfg`, `EdgeKindPolicy`, `CrossSchemaCfg`, and wire into `SchemaValidateCfg`.
- `src/validate/schema_rules/apply.rs`: add required and cross-schema checks using new config types.
- `src/validate.rs`: read per-edge cycle policy for depends_on when assigning severity to cycles.
- `src/commands/validate_cmd.rs`: extend classifier for new EDGE_* codes.
- `src/validate/tests.rs`: add tests covering scalar/array FM, missing IDs, cross-schema violations, and cycle overrides.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE>

- [ ] Define config structs: `EdgeKindPolicy { weight: f64, required: String, cycle_detection: String }`, `CrossSchemaCfg { allowed_targets: Vec<String> }`, and `WikilinksCfg { min_outgoing, min_incoming, severity }`.
- [ ] Add `edges: SchemaEdgesCfg` to `SchemaValidateCfg` (optional field).
- [ ] Deserialize snake_case keys with serde; ensure backward compatibility (no panic if missing).
- [ ] In `apply_schema_validation`, for each doc and for each configured edge kind:
      - [ ] Resolve FM field (string or array) → Vec<String>.
      - [ ] If `required` is error/warning and empty/missing → diagnostic EDGE_REQUIRED_MISSING with correct severity.
      - [ ] For each value, if unknown ID → diagnostic EDGE_ID_NOT_FOUND with correct severity.
      - [ ] If `cross_schema.allowed_targets` present and target schema not in whitelist → EDGE_CROSS_SCHEMA_DISALLOWED (severity error unless conventions specify otherwise; fall back to schema.validate.severity when needed).
- [ ] In cycle detection: prefer per-edge `cycle_detection` for depends_on if set, else fallback.
- [ ] Extend classifier in `validate_cmd` to map new messages to codes.
- [ ] Add unit tests for: required missing, scalar FM, bad ID, cross-schema violation, per-edge cycle override.
- [ ] Run `cargo test` and verify `--format json` envelope matches contracts.

### Acceptance Criteria
**GIVEN** a schema with `validate.edges.depends_on.required = "error"`
**WHEN** a note has no `depends_on` field
**THEN** `cli-rag validate --format json` includes an error with `code=EDGE_REQUIRED_MISSING` for that note.

**GIVEN** a schema with `validate.edges.implements.required = "warning"`
**WHEN** the note lists an unknown ID
**THEN** a `warning` diagnostic with `code=EDGE_ID_NOT_FOUND` is emitted.

**GIVEN** `validate.edges.cross_schema.allowed_targets = ["ADR"]`
**WHEN** an `IMP` note depends_on `LOG-001`
**THEN** a diagnostic with `code=EDGE_CROSS_SCHEMA_DISALLOWED` is emitted.

**GIVEN** per-edge `cycle_detection = "error"` for depends_on
**WHEN** a cycle is detected
**THEN** the cycle diagnostic is emitted as `error` regardless of schema-level default.

### Issues Encountered
Capture any edge normalization pitfalls, ambiguous severity interactions, or perf concerns here.


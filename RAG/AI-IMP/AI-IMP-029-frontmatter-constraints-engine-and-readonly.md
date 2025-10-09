---
node_id: AI-IMP-029
tags:
  - IMP-LIST
  - Implementation
  - validation
  - frontmatter
kanban_status: planned
depends_on: [AI-EPIC-XXX-cli-v1-authoring-contracts, AI-IMP-028]
confidence_score: 0.84
created_date: 2025-10-09
close_date:
--- 

# AI-IMP-029-frontmatter-constraints-engine-and-readonly

## {Summary of Issue #1}
`validate`/`submit` enforce only a subset of field constraints (required, unknown policy, regex, array+min_items, cross-type refs). Contracts specify richer constraints (`enum`, `globs`, `integer{min,max}`, `float{min,max}`) and readonly semantics for fields like `id`/`created_date`. We must add these validators and block readonly changes during `ai new submit`. Outcome: frontmatter constraints are enforced with severity as configured; StartResponse exposes the constraints envelope. {LOC|20}

### Out of Scope 
- Body heading policies and wikilinks (separate tickets).
- Template precedence wiring (Phase 1). {LOC|10}

### Design/Approach  
- Extend `SchemaRule` to include `enum`, `globs`, `integer{min,max}`, `float{min,max}`; preserve `regex`, `array`, `min_items`, `refers_to_types`.
- In `validate`, apply each rule with severity mapping to error/warn.
- Readonly: define a readonly set per schema (e.g., `id`, `created_date`, `last_modified`) and enforce in `ai new submit`: payload FM values cannot override readonly fields. Return structured diagnostics on violation.
- StartResponse.frontmatter: populate `allowed`, `readonly`, `enums`, and numeric bounds based on the effective schema.
- Deterministic error text; exit code 2 on submit violations. {LOC|25}

### Files to Touch
- `src/config/schema.rs`: extend rule struct for enum/globs/integer/float; readonly list source (schema/system set).
- `src/validate/rules.rs`: implement new validators; severity application; deterministic messages.
- `src/commands/ai_new/payload.rs`: block readonly FM overrides on submit and produce diagnostics.
- `src/commands/ai_new/store.rs`: enrich StartResponse.frontmatter constraints.
- Tests: `tests/integration_validate_json.rs` (constraints), `tests/integration_ai_new.rs` (readonly block, enum/globs/numbers). {LOC|25}

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [ ] Add `enum`, `globs`, `integer`, `float` fields to `SchemaRule`; parse from TOML.
- [ ] Implement enum checking for string(s); globs for string/array (use globset).
- [ ] Implement integer/float bounds with inclusive min/max; type-check before validating bounds.
- [ ] Add readonly list for system FM; block overrides during submit; add diagnostics with code `READONLY_FIELD`.
- [ ] Populate StartResponse.frontmatter with allowed/readonly/enums/bounds.
- [ ] Tests: positive/negative cases for each validator; submit fails on readonly and enum/globs/bounds violations; validate emits correct severity.
- [ ] Run `cargo fmt`, `clippy`, full tests.

### Acceptance Criteria
**Scenario:** Enum + globs validation
GIVEN a schema with `status.enum=[draft,proposed]` and `related_files.globs=["*.md"]`
WHEN validating a note with `status=legacy` or `related_files=["x.rs"]`
THEN validate reports an error (status) and an error/warn per severity for globs.

**Scenario:** Numeric bounds
GIVEN a schema with `priority.integer={min:0,max:100}` and `confidence_score.float={min:0.0,max:1.0}`
WHEN validating out-of-range values
THEN diagnostics are emitted with configured severities.

**Scenario:** Readonly enforcement on submit
GIVEN readonly fields `id` and `created_date`
WHEN `ai new submit` payload tries to override them
THEN submit exits 2 with structured diagnostics and no file written.

**Scenario:** StartResponse envelope
GIVEN `ai new start`
WHEN inspecting the JSON
THEN `frontmatter.allowed`, `frontmatter.readonly`, and enumerations/bounds are present and accurate. 

### Issues Encountered 
{LOC|20}


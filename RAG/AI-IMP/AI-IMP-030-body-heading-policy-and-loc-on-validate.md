---
node_id: AI-IMP-030
tags:
  - IMP-LIST
  - Implementation
  - validation
  - body
kanban_status: planned
depends_on: [AI-EPIC-XXX-cli-v1-authoring-contracts, AI-IMP-028]
confidence_score: 0.83
created_date: 2025-10-09
close_date:
--- 

# AI-IMP-030-body-heading-policy-and-loc-on-validate

## {Summary of Issue #1}
We only enforce per-heading LOC during `ai new submit` (on_creation) and do not implement `heading_check` (exact|missing_only|ignore), `max_count`, or `line_count.scan_policy=on_validate`. We must add these body policies and use the resolved template source (Lua→TOML→.md) as the heading baseline. Outcome: validate warns/errors according to policy; submit retains on_creation checks; on_validate re-checks updates made after creation. {LOC|20}

### Out of Scope 
- Wikilinks enforcement and per-edge policies (separate task).
- Frontmatter constraints engine (Phase 2). {LOC|10}

### Design/Approach  
- Resolve expected headings from the same source used by start (Lua→TOML→`.md`). Cache primary headings in drafts or recompute on validate using schema resolution.
- Implement `heading_check` modes:
  - `exact`: headings set must equal expected (order-sensitive or relaxed? default exact order).
  - `missing_only`: all expected headings must be present; extra headings allowed.
  - `ignore`: skip heading structure checks.
- Implement `max_count` to cap number of headings.
- Implement `line_count.scan_policy`:
  - `on_creation`: only on submit (existing behavior).
  - `on_validate`: recompute LOC per heading and report violations on validate.
- Severity per policy is applied according to configured severity in schema.validate.body. {LOC|25}

### Files to Touch
- `src/commands/ai_new/store.rs`: ensure heading list captured for start; optionally persist source hints.
- `src/validate/rules.rs`: add body heading structure and LOC-on-validate checks.
- `src/commands/ai_new/payload.rs`: keep on_creation LOC logic; share helpers with validate for parity.
- Tests: add integration tests exercising exact/missing_only/ignore, max_count, and on_validate LOC failures. {LOC|25}

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [ ] Resolve expected headings from precedence; expose a helper to retrieve the template headings for a schema.
- [ ] Implement `heading_check` with deterministic messages (exact vs missing_only).
- [ ] Implement `max_count` for body headings.
- [ ] Implement `line_count.scan_policy=on_validate` with per-heading LOC checks during validate.
- [ ] Tests: verify each policy mode and LOC enforcement across submit vs validate.
- [ ] Run `cargo fmt`, `clippy`, full tests.

### Acceptance Criteria
**Scenario:** exact heading policy
GIVEN a schema with `heading_check=exact`
WHEN validating a note with missing or reordered headings
THEN validate emits errors with deterministic messages.

**Scenario:** missing_only
GIVEN a schema with `heading_check=missing_only`
WHEN validating a note missing one expected heading
THEN validate emits an error; extra headings do not fail.

**Scenario:** max_count and on_validate LOC
GIVEN a schema with `max_count=5` and `line_count.scan_policy=on_validate`
WHEN a note exceeds heading count or LOC per heading after edits
THEN validate emits warnings/errors per severity; submit remains unaffected if not in on_creation.

### Issues Encountered 
{LOC|20}


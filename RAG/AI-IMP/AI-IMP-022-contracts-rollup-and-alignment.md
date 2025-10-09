---
id: AI-IMP-022
tags:
  - IMP-LIST
  - Implementation
  - contracts
  - cli
  - schemas
kanban_status: planned
depends_on:
  - ADR-003d
confidence_score: 0.82
created_date: 2025-09-18
close_date: 
---

# AI-IMP-022-contracts-rollup-and-alignment

## Validate JSON lacks protocolVersion
Issue: `validate --format json` omits `protocolVersion` despite global convention that all top‑level JSON responses include it. Current `contracts/v1/cli/validate_result.schema.json` also omits it to match the implementation.

### Out of Scope
- Changing any other command envelopes or exit codes.
- Reworking NDJSON output shapes.

### Design/Approach
- Add `protocolVersion` to the validate JSON output; keep the rest of the shape identical.
- Update the schema to include `protocolVersion` (integer, min 1).
- Preserve backward compatibility by allowing CLI to accept legacy output during a short transition if needed (human mode unaffected).

### Files to Touch
- `src/commands/validate_cmd.rs`: include `protocolVersion` in the JSON envelope.
- `contracts/v1/cli/validate_result.schema.json`: add `protocolVersion` to `required` and `properties`.
- `contracts/changelog.md`: add entry for the change.

### Implementation Checklist
- [ ] Add `protocolVersion` to validate JSON output in `src/commands/validate_cmd.rs`.
- [ ] Extend schema to include `protocolVersion` (int >= 1).
- [ ] Update `contracts/changelog.md` with rationale and scope.
- [ ] Run `cargo fmt`, `cargo clippy -D warnings`.

### Acceptance Criteria
Given `cli-rag validate --format json` runs on a repo,
When the command completes,
Then the JSON includes `protocolVersion` at the top level,
And the output validates against `contracts/v1/cli/validate_result.schema.json`.

## Duplicate `cache` key in info schema
Issue: `contracts/v1/cli/info.schema.json` declares `cache` twice (duplicate key). Last definition wins, but this is an error‑prone footgun.

### Out of Scope
- Changing the info envelope fields or capabilities content.

### Design/Approach
- Remove the duplicate `cache` block; keep a single definition matching implementation.

### Files to Touch
- `contracts/v1/cli/info.schema.json`: dedupe `cache` definition.
- `contracts/changelog.md`: add entry for the fix.

### Implementation Checklist
- [ ] Remove duplicate `cache` block.
- [ ] Verify `info` output continues to validate against the schema.
- [ ] Update changelog.

### Acceptance Criteria
Given the schema file,
When linted or inspected,
Then only one `cache` property definition exists,
And `cli-rag info --format json` validates against the schema.

## Validate diagnostics: allow null path
Issue: Validate JSON emits `"path": null` when a location cannot be derived. Current schema constrains `path` to `string` (if present), causing a validation mismatch.

### Out of Scope
- Changing diagnostic codes or adding new fields beyond nullability fixes.

### Design/Approach
- Allow `path` to be `string|null`.
- Keep `field`, `nodeId`, and `span` optional; do not adjust unless implementation changes.

### Files to Touch
- `contracts/v1/cli/validate_result.schema.json`: set `path` to `{"type": ["string","null"]}`.
- `contracts/changelog.md`: log the nullability adjustment.

### Implementation Checklist
- [ ] Update `path` type to `string|null`.
- [ ] Validate a sample output from `validate` against the schema.
- [ ] Update changelog.

### Acceptance Criteria
Given validate JSON with `path: null` diagnostics,
When validated against the schema,
Then it passes without errors.

## Issues Encountered
N/A yet. Document any blockers or deviations during implementation.


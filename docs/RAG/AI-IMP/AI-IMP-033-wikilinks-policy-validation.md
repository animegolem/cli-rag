---
node_id: AI-IMP-033
tags:
  - Implementation
  - validation
  - wikilinks
kanban_status: completed
depends_on:
  - AI-IMP-034
  - AI-EPIC-XXX-cli-v1-authoring-contracts
  - ADR-003d
confidence_score: 0.82
created_date: 2025-10-09
close_date: 2025-10-10
---

# AI-IMP-033-wikilinks-policy-validation

## Summary of Issue
Implement FR-9: enforce a schema-driven wikilinks policy during `validate`.
- Enforce min_outgoing and min_incoming counts of `[[ID]]` links per note with a configurable severity.
- Count outgoing by scanning each note; compute incoming globally from all notes.
- Do not alter the unified index format; checks run on-demand in validate.

### Out of Scope
- Changing index shape or adding wikilink edges to graph/commands.
- Editor integrations; watch UI beyond existing NDJSON events.

### Design/Approach
- Config: rely on `schema.validate.edges.wikilinks { min_outgoing, min_incoming, severity }` (added in AI-IMP-034).
- Collection:
  - Outgoing: reuse mention regex from `src/index.rs` and count UNIQUE target IDs per note (Obsidian-like backlink semantics). Multiple links to the same target in one note count as 1. Dedupe across the entire body.
  - Incoming: accumulate a map `targetId -> uniqueReferrersCount` while scanning all notes (count unique notes linking to the target).
- Validation rules:
  - For each note N, if `outgoing(N) < min_outgoing` → diagnostic LINK_MIN_OUT.
  - For each note N, if `incoming(N) < min_incoming` → diagnostic LINK_MIN_IN.
  - Severity resolved from wikilinks.severity; fallback to `schema.validate.severity`; default "error".
- Reporting: emit deterministic, single-line messages; update `validate_cmd` classifier to stable codes.

### Files to Touch
- `src/validate.rs`: orchestrate wikilinks scan and checks; integrate into existing validate flow.
- `src/validate/body.rs`: none (new logic is link-level, not headings/LOC).
- `src/commands/validate_cmd.rs`: add code classification for LINK_MIN_*; ensure JSON fields align with contracts.
- `src/index.rs`: reference regex only (no changes expected).
- `src/validate/tests.rs`: new tests for thresholds and severities.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE>

- [x] Add wikilink scan utility in `src/validate.rs` (collect outgoing per note; incoming global map).
- [x] Read config for current note’s schema: `schema.validate.edges.wikilinks` (guard if missing).
- [x] Compute `outgoing_count` as the number of unique target IDs linked from the note (dedupe across the whole note body).
- [x] Compute `incoming_count` from global map.
- [x] Push diagnostics when below thresholds with codes `LINK_MIN_OUT` / `LINK_MIN_IN` and configured severity.
- [x] Update `src/commands/validate_cmd.rs` classifier to map these messages to codes.
- [x] Add unit tests: (a) min_outgoing=1 with zero links → error/warn, (b) min_incoming=1 for an orphan note, (c) severity fallback behavior.
- [x] Verify `contracts/v1/cli/validate_result.schema.json` compliance (severity/code/msg/path present).
- [x] Run `cargo test` locally for the new tests.

### Acceptance Criteria
**GIVEN** a schema with `validate.edges.wikilinks.min_outgoing = 1` and severity `warning`
**WHEN** a note has zero `[[ID]]` links
**THEN** `cli-rag validate --format json` returns `ok=false` and includes a `warning` diagnostic with `code=LINK_MIN_OUT` for that note.

**GIVEN** a schema with `validate.edges.wikilinks.min_incoming = 1`
**WHEN** a note is not referenced by any other note via `[[ID]]`
**THEN** a diagnostic with `code=LINK_MIN_IN` is emitted with configured severity.

**GIVEN** no wikilinks block is present in schema
**WHEN** `validate` runs
**THEN** no LINK_MIN_* diagnostics are emitted.

### Issues Encountered
Document parsing/perf issues, edge false-positives, or ambiguity about counting semantics here.

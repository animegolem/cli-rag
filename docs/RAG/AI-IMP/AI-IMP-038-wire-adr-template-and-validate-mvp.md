---
node_id: AI-IMP-038
tags:
  - IMP-LIST
  - Implementation
  - templates
  - validation
kanban_status: planned
depends_on:
  - ADR-001-cli-rag.toml
  - AI-IMP-036
confidence_score: 0.84
created_date: 2025-10-10
close_date:
--- 

# AI-IMP-038-wire-adr-template-and-validate-mvp

## {Summary of Issue #1}
We need to wire the human ADR template into the repo flow and prove the MVP loop: init → create ADR note → validate → write unified index. The Project preset should already scaffold `.cli-rag/templates/ADR.toml` (contract-aligned with inline comments). This ticket ensures `validate` consumes that template/rules and that a basic ADR file round-trips with deterministic diagnostics/OK status. {LOC|20}

### Out of Scope 
- AI-ADR wiring (separate preset later)
- Additional schemas (AI-LOG/AI-EPIC/AI-IMP/etc.)
- README rewrite (handled once MVP lands) {LOC|10}

### Design/Approach  
- Use the updated human ADR contract template verbatim for `.cli-rag/templates/ADR.toml`.
- Create minimal ADR example in tests (temp dir) under `docs/RAG/ADR/ADR-001.md`.
- Run `cli-rag validate --format json`; assert OK path and, in a second case, a predictable warning/error (e.g., LINK_MIN_OUT if configured). {LOC|25}

### Files to Touch
- tests/integration_validate_json.rs: add an ADR-MVP test that initializes the preset, writes an ADR, runs validate, and asserts JSON shape/fields.
- (no code changes expected in runtime unless a gap appears while testing). {LOC|25}

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [ ] Add integration test: `init_preset_project_then_validate_single_adr_ok`.
- [ ] In the test, `cli-rag init --preset project --silent`; write `docs/RAG/ADR/ADR-001.md` with minimal frontmatter+headings.
- [ ] Run `cli-rag validate --format json`; assert `ok=true`, `docCount>=1`, and `protocolVersion` present.
- [ ] Add a second test `init_then_validate_adr_missing_link_warns` if `wikilinks.min_outgoing=1` in ADR template: assert `LINK_MIN_OUT` with severity `warning`.
- [ ] Ensure `.cli-rag/templates/ADR.toml` written by preset matches contracts (spot check key comment blocks in the test if reasonable).
- [ ] `cargo fmt`, full test suite.
 
### Acceptance Criteria
**Scenario:** ADR MVP OK path
GIVEN a repo initialized with the Project preset
WHEN an ADR `ADR-001.md` with required headings/frontmatter is created and `validate --format json` is run
THEN `ok=true` with `docCount>=1` and diagnostics (if any) follow the contracts schema.

**Scenario:** ADR MVP wikilink minimum (if configured)
GIVEN the ADR template config requires at least 1 outgoing wikilink
WHEN the ADR contains zero wikilinks
THEN validate returns a `warning` diagnostic with `code=LINK_MIN_OUT` for that note. {LOC|25}

### Issues Encountered 
{LOC|20}

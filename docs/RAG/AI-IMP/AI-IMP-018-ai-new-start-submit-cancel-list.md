---
node_id: AI-IMP-018
tags:
  - IMP-LIST
  - Implementation
  - ai-new
  - authoring
kanban_status: backlog
depends_on:
  - AI-EPIC-001
  - ADR-003d
confidence_score: 0.76
created_date: 2025-09-15
close_date:
---

# AI-IMP-018-ai-new-start-submit-cancel-list

## Summary of Issue #1
Implement minimal `ai new` flows aligned to ADR-003d: start (reserve ID/filename, emit contract), submit (validate structured sections/frontmatter before writing), cancel (drop draft), list (inspect drafts). Outcome: agents can prepare content off-disk and commit atomically.

### Out of Scope
- Rich validation policies beyond existing schema rules; multi-draft workflows; editor integrations.

### Design/Approach
- Draft store at `.cli-rag/drafts/<draftId>.json` with TTL.
- Surfaces:
  - `ai new start --schema ADR [--title]`: returns draft envelope with constraints and noteTemplate (string), mirrors ADR.
  - `ai new submit --draft <ID> --stdin|--sections @file|--from-file <md>`: validates and writes; marks draft complete.
  - `ai new cancel --draft <ID>`; `ai new list`.
- Emit JSON surfaces contract-style; exit codes per global conventions.

### Files to Touch
- `src/cli.rs` + `src/bin/cli-rag.rs`: add ai new subcommands.
- `src/commands/`: new module for ai_new with start/submit/cancel/list handlers.
- `tests/`: integration tests for start→submit and cancel flows.
- `.github/workflows/ci.yml`: optional smoke (start/cancel) gated behind time.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE>

- [ ] Define JSON contracts minimally consistent with ADR; wire clap surfaces.
- [ ] Implement draft store; TTL handling; ID strategy.
- [ ] Validate submit shapes via existing parser/validator; write file atomically.
- [ ] Tests for happy path and error exits (2/4/5 codes where applicable).

### Acceptance Criteria
GIVEN `ai new start --schema ADR --title X`, WHEN called, THEN returns a JSON envelope with `draftId`, `noteTemplate`, and constraints; WHEN `ai new submit --draft dft_x --sections @p.json`, THEN the file is written and `validate` passes.

### Issues Encountered
{LOC|20}


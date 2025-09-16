---
node_id: AI-IMP-019
tags:
  - IMP-LIST
  - Implementation
  - ci
  - contracts
kanban_status: backlog
depends_on:
  - AI-EPIC-001
  - AI-IMP-014
  - AI-IMP-015
  - AI-IMP-018
confidence_score: 0.8
created_date: 2025-09-15
close_date:
---

# AI-IMP-019-ci-gates-for-new-ai-new-and-nested-config

## Summary of Issue #1
Extend the contracts compliance workflow to validate nested config acceptance and authoring surfaces: `new` id generation + filename_template and minimal `ai new` start/cancel smoke. Keep matrices lean.

### Out of Scope
- Heavy authoring scenarios; limit to quick smoke checks.

### Design/Approach
- Add steps to create a nested config fixture, run `validate --format json`, check resolved snapshot and index path.
- Add steps to run `new --schema ADR --title Test`, assert file created and validate passes.
- Add steps to run `ai new start` then `ai new cancel`, assert JSON shapes and absence of drafts.

### Files to Touch
- `.github/workflows/ci.yml`: add steps under contracts job.
- `scripts/ci-fixtures-check.sh`: optional local mirror.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE>

- [ ] CI: nested config fixture and validation.
- [ ] CI: `new` smoke: create note then validate.
- [ ] CI: `ai new` smoke: start then cancel.
- [ ] Docs: brief note in README for CI surfaces.

### Acceptance Criteria
CI passes with added steps across ubuntu-latest; other OS in the main matrix unaffected.

### Issues Encountered
{LOC|20}


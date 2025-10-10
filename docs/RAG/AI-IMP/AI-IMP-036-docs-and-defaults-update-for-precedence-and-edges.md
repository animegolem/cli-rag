---
node_id: AI-IMP-036
tags:
  - Implementation
  - docs
  - templates
kanban_status: completed
depends_on:
  - AI-IMP-033
  - AI-IMP-034
  - AI-EPIC-XXX-cli-v1-authoring-contracts
confidence_score: 0.78
created_date: 2025-10-09
close_date: 2025-10-10
---

# AI-IMP-036-docs-and-defaults-update-for-precedence-and-edges

## Summary of Issue
Complete FR-11 for authoring docs & defaults:
- Document template precedence (Lua → TOML → repo `.md` → fallback) and variables (`{{filename}}`, `{{schema.name}}`, `{{now | date:"..."}}`, `{{frontmatter}}`, `{{LOC|N}}`).
- Document edges & wikilinks policies and cross-schema whitelist semantics (allow-all by default; whitelist when specified).
- Ensure examples in ADRs and templates align with the implemented behavior.

### Out of Scope
- Any changes to contract schemas or CLI JSON shapes.

### Design/Approach
- Update comments/examples in `contracts/v1/config/user_config/templates/ADR.toml` to reflect settled semantics.
- Add a brief “Authoring Precedence” and “Edges Policy” section to `RAG/ADR/ADR-001-cli-rag.toml.md` and reference ADR-003d for CLI.
- Optional: Add a short README snippet under `RAG/ADR/` summarizing cross-schema whitelist behavior for quick operator reference.

### Files to Touch
- `contracts/v1/config/user_config/templates/ADR.toml` (comments only).
- `RAG/ADR/ADR-001-cli-rag.toml.md`.
- `RAG/ADR/ADR-003d-v1.2-locked-CLI-commands.md` (minor notes on validate flags and expectations).

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE>

- [x] Update ADR template comments for edges, wikilinks, and precedence.
- [x] Add cross-schema whitelist clarifications (default allow-all; whitelist limits targets).
- [x] Add brief sections to ADR-001 and ADR-003d aligning CLI behavior with contracts.
- [x] Proofread for casing policy (snake_case in TOML; JSON camelCase naming when referenced).

### Acceptance Criteria
**GIVEN** a reader unfamiliar with the code
**WHEN** they review ADR-001, ADR-003d, and ADR template comments
**THEN** they can correctly configure template precedence, wikilinks minimums, required edges, and cross-schema whitelist without consulting the source.

### Issues Encountered
Initial template comments lacked `{{filename}}` / `{{schema.name}}` references and did not spell out the wikilink counting semantics; updated to match the implemented behavior and noted the cross-schema allow-all default.

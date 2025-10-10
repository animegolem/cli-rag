---
node_id: AI-IMP-039
tags:
  - IMP-LIST
  - Implementation
  - templates
  - authoring
kanban_status: planned
depends_on:
  - ADR-001-cli-rag.toml
  - AI-IMP-036
confidence_score: 0.85
created_date: 2025-10-10
close_date:
--- 

# AI-IMP-039-add-ai-adr-template

## {Summary of Issue #1}
Add the AI-ADR authoring template (contract-aligned) and prove it can be consumed when a project imports it, without yet wiring it into the default Project preset. This provides the prompt/sections for `ai new` while we keep the human ADR as the only preset schema for now. {LOC|20}

### Out of Scope 
- Changing the interactive init menu (AI-ADR remains off by default)
- Adding AI-ADR to the Project preset imports
- Broader schemas (AI-LOG/AI-EPIC/AI-IMP) {LOC|10}

### Design/Approach  
- Place the contract-aligned template at `contracts/v1/config/user_config/templates/AI-ADR.toml` (exists, refine as needed).
- In tests, simulate a project that imports `.cli-rag/templates/AI-ADR.toml` and copy the contract file into that location to validate behavior.
- Ensure `ai new start --schema AI-ADR` returns the AI prompt + body template per precedence (TOML source), and that `submit` drives the validate-on-creation behavior once rules are present. {LOC|25}

### Files to Touch
- tests/integration_ai_new.rs: add a case that imports AI-ADR and asserts that `start` returns the AI-specific instructions and template.
- (optional) docs/RAG/ADR/ADR-001-cli-rag.toml.md: short note referencing AI-ADR as the AI-only schema (no preset change). {LOC|25}

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [ ] Confirm `contracts/v1/config/user_config/templates/AI-ADR.toml` content matches intended AI prompt and sections.
- [ ] Integration test: scaffold a temp project with `config.templates.import = [".cli-rag/templates/AI-ADR.toml"]`; copy contract AI-ADR TOML into place.
- [ ] Run `cli-rag ai new start --schema AI-ADR --title Test --format json`; assert returned `instructions` contain the AI directives and `noteTemplate` matches the AI-ADR body.
- [ ] (Optional) `submit` a minimal draft and assert validate response shape (OK vs. readable errors) without writing to preset.
- [ ] `cargo fmt`, full test suite.
 
### Acceptance Criteria
**Scenario:** AI-ADR template available when imported
GIVEN a repo that imports `.cli-rag/templates/AI-ADR.toml`
WHEN `cli-rag ai new start --schema AI-ADR --title Test --format json` runs
THEN `instructions` reflect the AI prompt and `noteTemplate` reflects the AI note sections defined in the AI-ADR template. {LOC|25}

### Issues Encountered 
{LOC|20}

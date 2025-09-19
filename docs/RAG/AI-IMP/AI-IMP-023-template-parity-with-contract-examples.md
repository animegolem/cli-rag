---
node_id: AI-IMP-023
tags:
  - IMP-LIST
  - Implementation
  - templates
  - contracts
  - authoring
kanban_status: planned
depends_on:
  - AI-EPIC-003
confidence_score: 0.86
created_date: 2025-09-18
close_date:
---

# AI-IMP-023-template-parity-with-contract-examples

## Summary of Issue #1
Our Markdown templates (`.cli-rag/templates/{ADR,IMP,EPIC}.md`) are minimal and lack the contract-aligned prompting and guidance shown in `contracts/v1/config/user_config/templates/ADR.toml` (hidden comments, section scaffolds, and `((frontmatter))`). Scope: bring the repo templates to parity with the contract examples so that AI-first authoring (`cli-rag ai new start`) yields high-quality, structured drafts without manual boilerplate. Done when `cli-rag ai new start --schema ADR --title Test --format json` exposes the contract guidance blocks via `.noteTemplate` and all generated notes validate with no new errors. {LOC|20}

### Out of Scope 
- Changes to JSON outputs, exit codes, or retrieval behavior.
- New Lua hooks or prompt orchestration (we keep current hook surface).
- File placement logic (handled in AI-IMP-024). {LOC|10}

### Design/Approach  
- Source of truth: mirror the “template” content and guidance patterns from the contract example into our Markdown files, preserving:
  - Hidden instructional comments (HTML comments) and section headings.
  - `((frontmatter))` injection point (already supported by the renderer) and `{{LOC|N}}` caps (already supported).
  - Standard variables `{{id}}`, `{{title}}`, dates via `{{date}}` (supported by the renderer).
- Keep template tone concise and professional; align headings with ADR-003d examples in docs/RAG/templates.
- Add a short README note documenting how TOML schema links to Markdown bodies in this repo (TOML defines id/filename/frontmatter rules; Markdown defines the authored content).
- Optional: add an `ai new start --format json | jq '.noteTemplate'` example to README to show expected output for onboarding. {LOC|25}

### Files to Touch
- `.cli-rag/templates/ADR.md`: replace body with contract-parity scaffold (hidden guidance, sections, ((frontmatter)), LOC caps).
- `.cli-rag/templates/IMP.md`: same as ADR, tailored to IMP headings.
- `.cli-rag/templates/EPIC.md`: same as ADR, tailored to EPIC headings.
- `README.md`: brief note on template linkage and how to preview via `ai new start`.
- Tests: ensure `ai new start` responses contain expected headings/comments (non-brittle contains checks). {LOC|25}

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [ ] Update `.cli-rag/templates/ADR.md` with contract guidance: hidden comments, section headers, `((frontmatter))`, and `{{LOC|100}}` caps.
- [ ] Update `.cli-rag/templates/IMP.md` with IMP-specific sections and LOC caps.
- [ ] Update `.cli-rag/templates/EPIC.md` with EPIC-specific sections per docs/RAG/templates/AI-EPIC.md.
- [ ] README: document TOML↔Markdown template linkage and demonstrate preview via `ai new start --format json`.
- [ ] Tests: extend AI new coverage to assert that `noteTemplate` contains expected headings and guidance markers.
- [ ] CI: no schema changes; ensure tests pass across OS matrix.

### Acceptance Criteria
**Scenario:** Contract-parity ADR template
GIVEN the repository is configured with the new templates
WHEN running `cli-rag ai new start --schema ADR --title Test --format json`
THEN the JSON output's `.noteTemplate` field includes the hidden guidance comments and required section headings
AND frontmatter is injected via `((frontmatter))`
AND the note validates without new errors.

**Scenario:** Contract-parity IMP and EPIC templates
GIVEN the same environment
WHEN running `cli-rag ai new start --schema IMP … --format json` and `cli-rag ai new start --schema EPIC … --format json`
THEN the `.noteTemplate` payloads contain their schema-specific sections and guidance
AND validate passes.

### Issues Encountered 
{LOC|20}

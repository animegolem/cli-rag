---
node_id: AI-IMP-025
tags:
  - IMP-LIST
  - Implementation
  - deprecation
  - authoring
  - ai
kanban_status: planned
depends_on:
  - AI-EPIC-003
  - AI-IMP-022
confidence_score: 0.84
created_date: 2025-09-18
close_date:
---

# AI-IMP-025-retire-legacy-new-command

## Summary of Issue #1
The legacy `new` command duplicates authoring behavior while the project is moving to AI-first flows (`ai new start|submit|cancel|list`). Maintaining both increases docs burden and user confusion. Scope: remove `new` entirely, rely on `ai new …`, and update documentation/tests accordingly. Done when the binary no longer recognizes `new`, README and CI examples prefer AI flows, and integration tests cover the new destination behavior through the AI pipeline. {LOC|20}

### Out of Scope 
- Changing template rendering, ID generation, or JSON outputs.
- Building interactive wizards. {LOC|10}

### Design/Approach  
- CLI: delete the `new` subcommand; Clap will surface the standard "unrecognized subcommand" error for attempts to use it.
- AI workflow: rely exclusively on `ai new start|submit|cancel|list` for note authoring; ensure destination logic lives there.
- Documentation: update README and templates section to recommend `ai new` flows; include worked examples for `start`, `submit` (stdin/file), `cancel`, `list`.
- Tests: migrate destination-related coverage to the AI flow.
- CI: no special handling beyond existing AI new coverage. {LOC|25}

### Files to Touch
- `src/cli.rs`, `src/bin/cli-rag.rs`, and `src/commands/mod.rs`: remove the `new` subcommand wiring.
- Delete `src/commands/new.rs` and related integration tests; port necessary helpers into shared modules.
- `README.md`: replace authoring examples with `ai new` flows; keep a short migration note.
- Documentation (IMP/EPIC plans, changelog) to reflect the removal. {LOC|25}

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [ ] Remove `cli-rag new` subcommand and associated code paths.
- [ ] README: replace authoring walkthroughs with AI-first examples; keep a short migration section.
- [ ] Tests: ensure AI new coverage exercises destination and template behavior.
- [ ] Changelog: document removal and new guidance.

### Acceptance Criteria
**Scenario:** Legacy command removed
GIVEN a configured repo
WHEN a user runs `cli-rag new --schema ADR --title X`
THEN the CLI exits with an "unrecognized subcommand" error and hints at `cli-rag ai new`.

**Scenario:** AI workflow remains functional
GIVEN the same repo
WHEN a user runs `cli-rag ai new start --schema ADR --title X --format json`
THEN the command succeeds and returns the template guidance payload (as covered in AI-IMP-023/024).

### Issues Encountered 
{LOC|20}

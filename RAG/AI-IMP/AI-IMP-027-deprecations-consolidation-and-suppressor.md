---
node_id: AI-IMP-027
tags:
  - IMP-LIST
  - Implementation
  - cli
  - deprecations
kanban_status: completed
depends_on: [AI-EPIC-003]
confidence_score: 0.85
created_date: 2025-10-08
close_date: 2025-10-08
---

# AI-IMP-027-deprecations-consolidation-and-suppressor

## Summary of Issue #1
Remove the legacy `ai-index-plan` / `ai-index-apply` commands now that the unified `cli-rag ai index` workflow is established. Scope: delete alias wiring from the CLI, refresh help/completions/tests to list only the canonical hierarchy, and update documentation so migration guidance reflects the clean surface. Done when invoking the old aliases yields an unknown-command error and README/help/completions reference only the `cli-rag ai index` routes. {LOC|20}

### Out of Scope 
- Removing or altering JSON output shapes or exit codes.
- Changes to command functionality beyond deprecation handling. {LOC|10}

### Design/Approach  
- Drop alias variants from the Clap command tree and binary dispatch, letting Clap report them as unknown commands.
- Regenerate shell completions and tighten help coverage so only `cli-rag ai index plan|apply` are presented.
- Refresh README migration notes to remove alias references. {LOC|25}

### Files to Touch
- `src/bin/cli-rag.rs`: remove alias dispatch.
- `src/cli.rs`: delete alias definitions from the command enum.
- `README.md`: update migration guidance to reference only the canonical commands.
- `tests/integration_help.rs`: ensure help output matches the streamlined surfaces. {LOC|25}

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [x] Remove alias command variants from `src/bin/cli-rag.rs` and `src/cli.rs`.
- [x] Update README to reflect unified `ai` namespace with no alias references.
- [x] Extend `tests/integration_help.rs` to assert unified surfaces.
- [x] Run `cargo fmt`, `clippy`, and full tests locally.

### Acceptance Criteria
**Scenario:** Legacy aliases rejected
GIVEN the CLI is built with the updated command tree
WHEN invoking `cli-rag ai-index-plan`
THEN Clap reports the command as unknown.

**Scenario:** Help/completions reflect unified surfaces
GIVEN the CLI is built with updated metadata
WHEN invoking `cli-rag --help` and generating completions
THEN only canonical `ai index plan|apply` and `ai new …` entries appear. 

### Issues Encountered 
None yet. Document any help-format caveats across shells when implemented. {LOC|20}

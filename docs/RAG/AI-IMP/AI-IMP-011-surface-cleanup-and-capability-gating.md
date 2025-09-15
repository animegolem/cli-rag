---
node_id: AI-IMP-011
tags:
  - IMP-LIST
  - Implementation
  - cleanup
  - cli
  - docs
kanban_status: done
depends_on:
  - ADR-003d
  - docs/RAG/BRIDGE_PLAN_V2.md
confidence_score: 0.81
created_date: 2025-09-14
close_date: 2025-09-15
---

# AI-IMP-011-surface-cleanup-and-capability-gating

## Summary of Issue #1
Align CLI surfaces with ADR-003d by deprecating/removing legacy `topics` and `group` commands and groups JSON emission paths. Gate `info` capabilities to avoid over-advertising: set `aiIndex` false until plan/apply are implemented; introduce `overlaysEnabled` based on Lua status. Update docs, help text, and completions accordingly. Outcome: a clean, accurate CLI surface aligned with contracts and ADR decisions.

### Out of Scope 
- Implementing ai index plan/apply and Lua hooks (tracked in separate tickets).
- Migration tools for existing groups files (manual for alpha).

### Design/Approach  
- Soft deprecate in one release (hide from help, keep code callable), then remove in the next; given alpha, we may remove directly.
- Remove `topics` and `group` command files and references from CLI enum; delete groups emission code paths.
- Info: toggle `capabilities.aiIndex` to false until commands exist; add `capabilities.overlaysEnabled` derived from overlay loader.
- Docs: update ADR references and BRIDGE_PLAN_V1/V2 notes; adjust CI scripts if they reference removed commands (they currently do not).

### Files to Touch
- `src/cli.rs`: remove command variants and wiring for `Topics` and `Group`.
- `src/commands/topics.rs`, `src/commands/group.rs`: delete or mark deprecated; remove from build.
- `src/index.rs`: remove groups-related writes if any remain.
- `src/commands/info.rs`: gate capability flags; add overlaysEnabled.
- `docs/RAG/BRIDGE_PLAN_V1.md` and V2: mark cleanup complete when done.
- `Justfile` and CI: ensure no references to deprecated commands.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [ ] CLI: Remove `Topics` and `Group` variants; update help/completions.
- [ ] Commands: Remove `src/commands/topics.rs` and `src/commands/group.rs`; adjust module exports.
- [ ] Index: Remove groups JSON write paths if present; keep unified index only.
- [ ] Info: Set `capabilities.aiIndex=false` until plan/apply land; add `capabilities.overlaysEnabled`.
- [ ] Docs: Update BRIDGE plans and ADR notes (groups deprecated/removed); adjust README if needed.
- [ ] CI: Run workflow locally and in CI to verify no references remain; ensure all schema gates still pass.

### Acceptance Criteria
**GIVEN** a build of the CLI, **WHEN** running `cli-rag --help`, **THEN** `topics` and `group` are no longer present; attempting to run them results in unknown command.

**GIVEN** `cli-rag info --format json`, **WHEN** ai index is not yet implemented, **THEN** `capabilities.aiIndex` is false; **AND** `overlaysEnabled` is present and accurate once overlay loader lands.

### Issues Encountered 
(to be completed during implementation)

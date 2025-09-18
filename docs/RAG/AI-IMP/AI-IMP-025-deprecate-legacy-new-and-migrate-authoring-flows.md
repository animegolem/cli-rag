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

# AI-IMP-025-deprecate-legacy-new-and-migrate-authoring-flows

## Summary of Issue #1
The legacy `new` command duplicates authoring behavior while the project is moving to AI-first flows (`ai new start|submit|cancel|list`). Maintaining both increases docs burden and user confusion. Scope: soft-deprecate `new` with a clear warning and doc steer; provide migration guidance and examples that rely on `ai new …`. Done when `new` emits a deprecation notice by default, README and CI examples prefer AI flows, and integration tests cover the message without breaking existing pipelines. {LOC|20}

### Out of Scope 
- Removing `new` entirely in this iteration (deprecation only).
- Changing template rendering, ID generation, or JSON outputs.
- Building interactive wizards. {LOC|10}

### Design/Approach  
- Behavior: invoking `cli-rag new …` prints a one-line deprecation notice on stderr that points to `cli-rag ai new …` (or to the repo docs). It proceeds to complete successfully (exit 0) with identical side effects for one release window.
- Escape hatch: allow suppressing the notice via `CLI_RAG_SILENCE_DEPRECATIONS=1` for CI-only noise reduction (documented, not promoted).
- Documentation: update README and templates section to recommend `ai new` flows; include worked examples for `start`, `submit` (stdin/file), `cancel`, `list`.
- CI: leave current tests green; optionally add a smoke assertion that `new` prints the deprecation string to stderr. {LOC|25}

### Files to Touch
- `src/commands/new.rs`: emit deprecation message to stderr before continuing; no behavior change.
- `README.md`: replace authoring examples with `ai new` flows; keep a short migration note from `new` → `ai new`.
- `tests/integration_new.rs`: assert deprecation message presence (stderr) and that exit code and outputs remain unchanged.
- `.github/workflows/ci.yml`: no required changes, optional: add a brief run of `ai new start/cancel` already present in contracts job. {LOC|25}

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [ ] Emit deprecation message on `cli-rag new` to stderr: "Deprecated: use `cli-rag ai new …`".
- [ ] Ensure message prints once per invocation and does not change exit codes or outputs.
- [ ] Add `CLI_RAG_SILENCE_DEPRECATIONS` guard; covered in docs but not recommended.
- [ ] README: replace authoring walkthroughs with AI-first examples; keep a short migration section.
- [ ] Tests: add integration test asserting stderr contains deprecation text and return code is 0.
- [ ] Changelog: document deprecation and the timeline for removal.

### Acceptance Criteria
**Scenario:** Warning without behavior change
GIVEN a configured repo
WHEN a user runs `cli-rag new --schema ADR --title X`
THEN stderr contains "Deprecated: use `cli-rag ai new`"
AND the command exits 0 and writes the note as before.

**Scenario:** Suppressing warning (explicit opt-out)
GIVEN the env `CLI_RAG_SILENCE_DEPRECATIONS=1`
WHEN running the same command
THEN no deprecation message prints
AND behavior remains identical.

### Issues Encountered 
{LOC|20}


---
node_id: LOG-AI-2025-09-17a
tags:
  - AI-log
  - development-summary
  - ai-new
  - workflows
closed_tickets:
  - AI-IMP-018
created_date: 2025-09-17
related_files:
  - src/cli.rs
  - src/bin/cli-rag.rs
  - src/commands/ai_new/**
  - tests/integration_ai_new.rs
  - .github/workflows/ci.yml
  - contracts/v1/cli/ai_new_*.schema.json
confidence_score: 0.7
---

# 2025-09-17-LOG-AI-ai-new-draft-workflow

## Work Completed
Implemented the `ai new` draft lifecycle (start, submit, cancel, list) per ADR-003d, wiring Clap and the binary dispatcher through a new `ai` command hierarchy. Added a modular `src/commands/ai_new/` stack to manage draft storage, payload validation, and note assembly, reusing shared ID-generation helpers. Published contract schemas for the new JSON outputs, updated CI with an ai-new smoke step, and introduced integration coverage for start→submit and cancel/list flows. Closed AI-IMP-018 and cleaned up indentation in the Lua watcher smoke test so the NDJSON handshake check passes.

## Session Commits
- `bc4baab feat: add ai new draft workflow` — introduced the ai-new command family, modularized command handling, published contract schemas, added integration tests, and extended CI with an ai-new smoke step plus the YAML syntax checker hook.
- `a04f177 Fix CI ai new smoke indentation` — corrected Python indentation inside the watch NDJSON handshake script, restoring CI stability.

## Issues Encountered
Resolved a manual merge conflict in `.github/workflows/ci.yml` when integrating the new smoke step with upstream edits. CI initially failed due to mis-indented heredoc blocks in the watcher smoke test; adjusting Python indentation fixed the runtime error. The YAML checker currently skips if PyYAML is missing; noted the need to ensure the dependency is provisioned in all environments.

## Tests Added
Created `tests/integration_ai_new.rs` covering happy-path start→submit and cancel/list flows to ensure drafts are persisted, validated, and cleaned up. Verified the suite with targeted `cargo test integration_ai_new -- --nocapture` and a full `cargo test` run.

## Next Steps
Monitor CI once the branch lands to confirm the ai-new smoke step remains stable. Follow up by documenting the new CLI surfaces for users and evaluating whether PyYAML should be bundled or enforced for the YAML check. Coordinate with owners of AI-IMP-019 to extend CI assertions if additional draft metadata needs validation.

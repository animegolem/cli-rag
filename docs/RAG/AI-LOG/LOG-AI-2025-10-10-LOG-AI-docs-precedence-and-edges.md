---
node_id: AI-LOG-004
tags:
  - AI-log
  - documentation
  - validation
closed_tickets:
  - AI-IMP-036
created_date: 2025-10-10
related_files:
  - contracts/v1/config/user_config/templates/ADR.toml
  - RAG/ADR/ADR-001-cli-rag.toml.md
  - RAG/ADR/ADR-003d-v1.2-locked-CLI-commands.md
confidence_score: 0.78
---

# 2025-10-10-LOG-AI-docs-precedence-and-edges

## Work Completed
- Documented the authoring precedence ladder (Lua → TOML → repo templates → fallback) and expanded variable coverage (`{{filename}}`, `{{schema.name}}`, filtered `{{now}}`) in both the contracts template and ADR-001 reference.
- Clarified how wikilink thresholds count unique targets/referrers and how cross-schema allowlists default to allow-all when unset; synced ADR-003d’s `validate` section and JSON example with the new LINK_MIN_* diagnostics.

## Tests
- Documentation-only change; no automated tests required.

## Next Steps
- None for this thread; book is closed on the authoring epic docs.

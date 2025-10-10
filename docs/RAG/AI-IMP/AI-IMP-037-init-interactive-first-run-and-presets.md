---
node_id: AI-IMP-037
tags:
  - Implementation
  - init
  - onboarding
  - presets
kanban_status: planned
depends_on:
  - AI-ADR-005
confidence_score: 0.86
created_date: 2025-10-10
close_date:
---

# AI-IMP-037-init-interactive-first-run-and-presets

## Summary of Issue
Implement an interactive first‑run `init` that writes a contract‑aligned `.cli-rag.toml`, scaffolds the ADR template, and cleanly handles overwrite/backup. Add a “Project Manager” preset (dogfood) as the default interactive choice. Options 2 and 3 (generic preset and fully commented example) should return a friendly “Not implemented” message for now. The written files must mirror `contracts/v1/config/user_config/cli-rag.toml` and the ADR template comments updated in AI-IMP-036, so a user can “ctrl+F and tweak”.

## Design/Approach
- Interactive by default: `cli-rag init` opens a simple menu:
  1. Project Manager preset (dogfood) — writes `.cli-rag.toml` and imports ADR template
  2. Generic preset — Not implemented (return exit 1 with message)
  3. Fully commented example (TOML/Lua) — Not implemented (return exit 1)
- Safe defaults:
  - Destinations under `docs/RAG` by default.
  - `[config.scan] filepaths = ["docs/RAG"]`, `index_path = ".cli-rag/index.json"`, `hash_mode = "mtime"`, `index_strategy = "content"`, standard `ignore_globs`, `ignore_symlinks = true`.
  - Graph defaults per contracts.
  - `[config.templates] import = [".cli-rag/templates/ADR.toml"]`.
- Template scaffolding:
  - Write `.cli-rag/templates/ADR.toml` using the updated contract template (AI-IMP-036). Include precedence, variables, wikilinks policy (min_out=1, min_in=0, severity="warning"), and edge policy examples. Do not set `cross_schema.allowed_targets` (allow‑all).
- Overwrite behavior:
  - If `.cli-rag.toml` exists: Offer View diff → Back up (`.bak`) → Overwrite → Cancel.
  - Never write outside repo; always relative paths.
- No index writes: `init` never writes the index; users run `validate` to build it and pick up pre‑existing notes.
- No seed notes: Do not write example notes.

## CLI Behavior & Flags
- Keep existing flags working; add interactive on no‑flags:
  - `--preset project` bypasses prompts and writes the project preset.
  - `--dry-run` prints would‑be files.
  - `--silent` suppresses opening the editor.
  - `--json` emits `{ protocolVersion, preset, created:[], updated:[], warnings:[]? }` (new).
  - For now, `--schema` and `--separate` are accepted but ignored for the project preset; print a warning (future use).
  - Selecting options 2 or 3 prints a friendly NI message and exits 1.

## Files to Touch
- `src/commands/init.rs`:
  - Add interactive menu when invoked without `--preset`.
  - Add writers that produce a contracts‑aligned `.cli-rag.toml` (docs/RAG defaults) and copy/render ADR template to `.cli-rag/templates/ADR.toml`.
  - Implement backup/overwrite flow and `--dry-run`/`--json` outputs.
- `contracts/v1/config/user_config/cli-rag.toml`: use as the source of truth for field names/order.
- Tests:
  - `tests/integration_init.rs`: add cases for interactive bypass (env‑driven), `--preset project --json`, existing‑config overwrite/backup, and NI flows.

## Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete you MUST verify behavior with integration tests and ensure generated files match contracts field names and defaults.
</CRITICAL_RULE>

- [ ] Add interactive menu for `cli-rag init` with no flags.
- [ ] Implement Project preset writer (contracts‑aligned `.cli-rag.toml`, docs/RAG defaults).
- [ ] Scaffold `.cli-rag/templates/ADR.toml` from our updated contract template (AI-IMP-036).
- [ ] Overwrite guard: diff → backup `.bak` → overwrite → cancel.
- [ ] `--dry-run` prints; `--json` emits created/updated paths and preset.
- [ ] Options 2/3: print NI message and exit 1.
- [ ] Tests for: preset project JSON; overwrite/backup; NI branches; editor suppression; lines/fields parity with contracts.
- [ ] Run `cargo fmt`, `clippy`, and full tests.

## Acceptance Criteria
GIVEN a repo without config
WHEN `cli-rag init --preset project --json` runs
THEN it returns protocolVersion, preset=project, and lists `.cli-rag.toml` and `.cli-rag/templates/ADR.toml` under created; file contents match contracts defaults (docs/RAG destinations).

GIVEN `.cli-rag.toml` already exists
WHEN running `cli-rag init` interactively
THEN the tool offers diff/backup/overwrite; selecting backup creates `.cli-rag.toml.bak` and rewrites `.cli-rag.toml`.

GIVEN a user chooses preset 2 or 3
WHEN running `cli-rag init`
THEN the command prints “Not implemented (coming in 1.0)” and exits with code 1.

## Notes / Open Questions
- Expose the recommended authoring prompt from the ADR template via `cli-rag help templates` later (not in scope here).
- If `$EDITOR` is unset, default to `nvim`; respect `--silent`.

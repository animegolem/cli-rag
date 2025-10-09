---
node_id: AI-EPIC-001
id: AI-EPIC-001
tags:
  - EPIC
  - AI
  - planning
  - contracts
  - dogfooding
date_created: 2025-09-15
date_completed:
kanban_status: in-progress
AI_IMP_spawned:
  - AI-IMP-014
  - AI-IMP-015
  - AI-IMP-016
  - AI-IMP-017
  - AI-IMP-018
  - AI-IMP-019
  - AI-IMP-020
  - AI-IMP-021
---

# AI-EPIC-001-contracts-convergence-and-dogfooding

## Problem Statement/Feature Scope
We have contracts that define a richer user_config TOML and schema capabilities than the current implementation supports. The config loader still uses flat top-level keys, and schema features such as id_generator, filename templates, GTD/frontmatter modeling, and custom validators are not wired. To move from “works in CI” to active dogfooding, we need to converge implementation to the contracts, expose the intended authoring surfaces, and adopt the tool in this repo.

## Proposed Solution(s)
Converge the loader and surfaces to match contracts while keeping backward compatibility as a courtesy (optional, given alpha). Key push areas:
- Loader shape convergence: accept nested `[config.scan|authoring|graph|templates]` structure in addition to current top-level keys; map to the existing internal Config and ResolvedConfig.
- Schema capability uplift: implement id_generator strategies (increment, datetime, uuid) and filename_template interpolation used by `new`; add schema-level frontmatter policies for “user_frontmatter” and “system_frontmatter” (GTD subset initially: kanban_status, kanban_statusline, due_date).
- Overlay provenance: persist overlay paths in resolved snapshot; maintain `overlays.enabled` and smoke tests.
- AI authoring: design and implement the “ai new” sub-commands (start/submit/cancel/list) per ADR-003d; initially keep minimal, contract-aligned shapes.
- Documentation pass: align README/init template to new config shape and schema examples; add an EPIC/IMP dogfooding flow.
- CI: extend contracts checks to validate `new`/`ai new` outputs and the nested config path acceptance.

## Path(s) Not Taken
- Full-blown schema DSL and validation engine rework is out of scope for this EPIC; we will implement the minimal features to satisfy contracts and dogfooding, deferring advanced validators to a later phase.
- Back-compat shims for every past key name are not necessary at alpha; we will document the shape and keep only the common, practical aliases for now.

## Success Metrics
- By end of sprint N: `cli-rag init --print-template` emits a config aligned to contracts (includes config_version, filepaths, index_relative; no groups) and `load_config` accepts nested user_config TOML.
- By end of sprint N+1: `new --schema ADR --title X` yields filenames via filename_template and ids via the configured generator; resolved.json and info reflect configVersion and overlays; CI gates cover `new` and nested config.
- Dogfooding: Repo maintainers can create ADR/IMP/EPIC notes via `new` and validate with zero errors; CI passes on PRs using the new config.

## Requirements

### Functional Requirements
- [ ] FR-1: Loader accepts nested user_config TOML (contracts/v1/config/user_config/cli-rag.toml shape) and maps to internal Config without breaking existing flat TOML.
- [ ] FR-2: `new` supports id_generator strategies: increment, datetime, uuid; configurable prefix/padding.
- [ ] FR-3: `new` supports `filename_template` with basic filters (kebab-case, snake_case, date) consistent with template docs.
- [ ] FR-4: Schema frontmatter policy: enforce presence/shape of “user_frontmatter” fields and system GTD subset (kanban_status, kanban_statusline, due_date) during validate.
- [ ] FR-5: Resolved snapshot includes overlay provenance and correct `configVersion`; info reflects the same.
- [ ] FR-6: Implement minimal `ai new` (start/submit/cancel/list) surfaces per ADR-003d with contract-shaped outputs.
- [ ] FR-7: Update `init` to scaffold separate schema files consistent with contract templates (ADR/IMP/EPIC), and refresh README examples.
- [ ] FR-8: Extend CI contracts job to validate `new`/`ai new` outputs and nested config acceptance.

### Non-Functional Requirements
- Keep deterministic outputs and existing exit codes; add tests alongside new features; maintain clippy/fmt gates.
- Clear error messages for invalid nested config keys and unsupported schema features.

## Implementation Breakdown
- [ ] AI-IMP-014: Loader accepts nested user_config TOML; map to internal Config; add tests; docs.
- [ ] AI-IMP-015: `new` id_generator (increment/datetime/uuid) + filename_template filters; tests.
- [ ] AI-IMP-016: Schema frontmatter modeling (user/system + GTD subset) and validate rules; update search/GTD emitters if needed; tests.
- [ ] AI-IMP-017: Overlay provenance in resolved.json; info/readme updates; CI smoke stays green.
- [ ] AI-IMP-018: `ai new` minimal flows (start/submit/cancel/list) per ADR-003d; contract-aligned outputs; tests.
- [ ] AI-IMP-019: CI gates for `new`/`ai new` and nested config; update contracts job; fixtures.
- [ ] AI-IMP-020: Dogfooding: migrate this repo’s config to nested shape; create EPIC/IMP/ADR schemas; use `new` in daily work; update docs.

---
node_id: AI-EPIC-XXX-cli-v1-authoring-contracts
tags:
  - EPIC
  - AI
  - CLI
  - contracts
  - authoring
date_created: 2025-10-09
date_completed:
kanban-status: planned
AI_IMP_spawned:
  - AI-IMP-XXX-template-precedence-and-prompt-wiring
  - AI-IMP-XXX-frontmatter-constraints-engine
  - AI-IMP-XXX-body-heading-policy-and-loc-on-validate
  - AI-IMP-XXX-wikilinks-and-edges-policy
  - AI-IMP-XXX-docs-and-default-destination-updates
---

# AI-EPIC-XXX-cli-v1-authoring-contracts

## Problem Statement/Feature Scope
The CLI’s AI authoring flow does not yet fully implement the contract features defined under `contracts/v1/config/user_config/templates/ADR.toml` and related overlay examples (`ADR-concept.lua`/`.fnl`). We currently consume repo-local `.cli-rag/templates/{Schema}.toml` (schema rules) and `.md` (body scaffolds), and wire a subset of Lua hooks, but we do not honor TOML/Lua template prompts/bodies, several field-level validators (enum, globs, numeric bounds), nor heading policy and wikilinks rules. We must close these gaps to reach a solid v1 authoring experience aligned with the contracts. {LOC|10}

## Proposed Solution(s)
Implement a contracts-first authoring pipeline with explicit precedence and full validator coverage:
- Template precedence for instructions and body: Lua (`template_prompt`/`template_note`) → TOML (`[schema.new.template.prompt|note]`) → repo `.md` scaffold → minimal fallback.
- Canonical template variables use `{{...}}`; tokens such as `{{frontmatter}}` and `{{LOC|N}}` must be supported.
- Extend variable DSL for body templates: support `{{filename}}`, `{{schema.name}}`, and `{{now | date:"%Y-%m-%d"}}` alongside existing filters.
- Elevate Lua overlay API to spec: `ctx.util`, `ctx.clock`, `ctx.schema`, `ctx.config`; wire `template_prompt`/`template_note` hooks.
- Add field-level validators and readonly enforcement in submit; expose the constraints envelope in StartResponse.
- Add body policies (heading_check, max_count, LOC scan_policy) and wikilinks min_out/min_in rules.
- Standardize defaults and docs to `RAG/ADR` for ADR destinations while allowing configuration overrides. {LOC|25}

## Path(s) Not Taken
- Building a new templating DSL: we continue with Markdown scaffolds and optional Lua template generation.
- Implementing GUI/TUI integration in this epic: scope is CLI-only; visual mode remains future work. {LOC|10}

## Success Metrics
- `ai new start` surfaces instructions from Lua/TOML when present; `.noteTemplate` reflects the resolved template with comments and constraints.
- `ai new submit` enforces readonly FM fields and field-level constraints (enum, globs, numeric bounds) and rejects on contract violations (exit 2).
- `validate` enforces heading policy (exact|missing_only|ignore), max_count, and LOC when scan_policy=on_validate, and applies wikilinks min_out/min_in with configured severity.
- Lua overlay hooks (`template_prompt`/`template_note`) work with enriched ctx; precedence is deterministic and documented.
- Defaults and README reflect `RAG/ADR` destinations; all tests and CI pass across platforms. {LOC|15}

## Requirements

### Functional Requirements
- [ ] FR-1 Template precedence (instructions): Lua `template_prompt(ctx)` → TOML `[schema.new.template.prompt].template` → fallback generic.
- [ ] FR-2 Template precedence (note body): Lua `template_note(ctx)` → TOML `[schema.new.template.note].template` → repo `.cli-rag/templates/{Schema}.md` → minimal fallback.
- [ ] FR-3 Canonical token support: render `{{frontmatter}}`, `{{id}}`, `{{title}}`, `{{schema.name}}`, `{{now|date:"..."}}`, and `{{filename}}` in body and filename templates.
- [ ] FR-4 Lua API completeness: provide `ctx.util` (case helpers), `ctx.clock` (`today_iso()`, `now_iso()`), `ctx.schema` (resolved schema table), and `ctx.config` (resolved config subset); keep `ctx.index.next_numeric_id`.
- [ ] FR-5 Frontmatter constraints: implement per-field `enum`, `globs`, `integer{min,max}`, `float{min,max}` with severity overrides; maintain existing `regex`, `array`, `min_items`, `refers_to_types`.
- [ ] FR-6 Readonly enforcement: block changes to readonly fields (e.g., `id`, `created_date`) at submit; return structured diagnostics.
- [ ] FR-7 StartResponse fidelity: include `headingStrictness`, `frontmatter.allowed`, `frontmatter.readonly`, `frontmatter.enums`, and numeric bounds.
- [ ] FR-8 Body heading policy: implement `heading_check = exact|missing_only|ignore`, `max_count`, and `line_count.scan_policy = on_creation|on_validate` using headings from the resolved template source.
- [ ] FR-9 Wikilinks policy: parse body `[[wikilinks]]`, enforce `min_outgoing` and `min_incoming` with severity, and report per-note diagnostics in validate.
- [ ] FR-10 Edges policy uplift: support per-edge `required`, per-edge `cycle_detection` (override schema-level), and `cross_schema.allowed_targets`.
- [ ] FR-11 Defaults/docs: standardize examples/presets to `RAG/ADR` for ADR; document destination precedence and template precedence.

### Non-Functional Requirements
- Contracts-first: when examples differ, the `contracts/` spec is authoritative; behavior and docs reflect the spec.
- Deterministic outputs: preserve stable ordering and exit codes; fail with exit 2 on contract violations.
- Backwards compatibility: maintain existing JSON envelopes while shifting templates to `{{frontmatter}}`. {LOC|20}

## Implementation Breakdown
- Phase 1: Authoring template ingestion
  - Lua/TOML precedence for prompt/note; token support (`{{frontmatter}}`); add `{{filename}}`; enrich Lua ctx.
- Phase 2: Frontmatter constraints
  - enum/globs/integer/float + readonly enforcement; populate StartResponse frontmatter constraints.
- Phase 3: Body policies
  - heading_check, max_count, LOC on validate; headings derive from resolved template.
- Phase 4: Wikilinks and edges policy
  - Parse wikilinks; enforce min_out/min_in; per-edge required/cycle overrides; cross-schema targets.
- Phase 5: Docs & defaults
  - Update README and examples for precedence and `RAG/ADR` defaults; outline operator guidance in `info`/docs. {LOC|25}

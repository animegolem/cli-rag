---
node_id: AI-IMP-028
tags:
  - IMP-LIST
  - Implementation
  - authoring
  - templates
  - lua
kanban_status: in-progress
depends_on: [AI-EPIC-XXX-cli-v1-authoring-contracts]
confidence_score: 0.86
created_date: 2025-10-09
close_date:
--- 

# AI-IMP-028-template-precedence-and-prompt-wiring

## Summary of Issue #1
`ai new start` does not honor contracts-defined template sources. Instructions are generic and `.noteTemplate` only uses repo `.md`. We must implement deterministic precedence for prompt/body: Lua → TOML → repo scaffold → fallback, standardize tokens to `{{...}}`, and expose `{{filename}}` for body templates. Outcome: start returns `instructions` and `noteTemplate` per precedence with all variables expanded (including `{{filename}}`), and repo scaffolds can be replaced by Lua/TOML when present. {LOC|20}

### Out of Scope 
- Frontmatter constraint engine (enum/globs/numeric) and readonly checks (Phase 2).
- Heading policy and LOC-on-validate (Phase 3).
- Wikilinks/body edges enforcement (later phase). {LOC|10}

### Design/Approach  
- Precedence (instructions): `overlay.hooks.template_prompt(ctx)` → TOML `[schema.new.template.prompt].template` → fallback generic.
- Precedence (note body): `overlay.hooks.template_note(ctx)` → TOML `[schema.new.template.note].template` → repo `.cli-rag/templates/{Schema}.md` → minimal fallback.
- Token policy: canonical `{{...}}`. Drop `((frontmatter))`; accept only `{{frontmatter}}` going forward. Migrate existing repo templates accordingly.
- Variables: render `{{id}}`, `{{title}}`, `{{schema.name}}`, `{{now|date:"..."}}`, `{{filename}}` (new). Compute `filename` at start using the active `filename_template`.
- Lua ctx enrichment (minimal subset for Phase 1): add `ctx.util` (case helpers) and `ctx.clock` (`today_iso()`, `now_iso()`). Add `ctx.schema` + `ctx.config` in Phase 2 if needed by tests.
- Keep deterministic behavior: if multiple sources provide fields, precedence applies; do not merge bodies. {LOC|25}

### Files to Touch
- `src/commands/ai_new/start.rs`: apply source precedence; compute `filename`; render body with canonical tokens; populate `instructions`.
- `src/commands/new_helpers.rs`: extend renderer to support `{{filename}}` and `{{frontmatter}}` (remove `((frontmatter))`).
- `src/commands/lua_integration.rs`: add optional `template_prompt`/`template_note` calls; add `ctx.util` and `ctx.clock` helpers.
- `.cli-rag/templates/*.md`: update internal tokens from `((frontmatter))` to `{{frontmatter}}`.
- Tests: `tests/integration_ai_new.rs` (new cases for Lua-only, TOML-only, scaffold fallback, and `{{filename}}` rendering). {LOC|25}

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [x] Add `template_prompt`/`template_note` hooks to Lua integration; expose `ctx.util` and `ctx.clock`.
- [x] Parse TOML prompt/note templates under `[schema.new.template]` when present.
- [x] Compute `filename` in `ai new start`; include as a template variable for body rendering.
- [x] Replace `((frontmatter))` support with `{{frontmatter}}`; migrate repo `.md` scaffolds.
- [x] Implement precedence resolution for `instructions` and `noteTemplate`.
- [x] Extend renderer to handle `{{schema.name}}`, `{{now|date:"..."}}` in body consistently.
- [x] Tests: Lua wins over TOML; TOML wins over `.md`; fallback works; `{{filename}}` is populated; token migration validated.
- [x] Run `cargo fmt`, `clippy`, full test suite.

### Acceptance Criteria
**Scenario:** Lua prompt/body precedence
GIVEN a repo with an overlay defining `template_prompt` and `template_note`
WHEN running `cli-rag ai new start --schema ADR --title Test --format json`
THEN `.instructions` equals the Lua prompt and `.noteTemplate` equals the Lua body with `{{id}}`, `{{title}}`, and `{{filename}}` expanded.

**Scenario:** TOML prompt/body precedence
GIVEN no Lua overrides and TOML defines `[schema.new.template.prompt|note]`
WHEN running start
THEN `.instructions` and `.noteTemplate` are sourced from TOML with variables expanded.

**Scenario:** Scaffold fallback
GIVEN no Lua/TOML templates
WHEN running start
THEN `.noteTemplate` uses the repo `.md` scaffold and `.instructions` uses the generic fallback.

**Scenario:** Canonical tokens
GIVEN a template containing `{{frontmatter}}`
WHEN running start
THEN the rendered frontmatter block is injected; `((frontmatter))` is no longer supported.

### Issues Encountered 
Lua overlay test initially returned the fallback prompt because the inline Lua string literal was malformed; adjusted fixtures to use Lua long strings so `template_note` loads successfully.

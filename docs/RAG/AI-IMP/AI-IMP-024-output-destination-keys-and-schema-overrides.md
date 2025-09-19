---
node_id: AI-IMP-024
tags:
  - IMP-LIST
  - Implementation
  - authoring
  - config
  - paths
kanban_status: planned
depends_on:
  - AI-EPIC-003
  - AI-IMP-022
confidence_score: 0.84
created_date: 2025-09-18
close_date:
---

# AI-IMP-024-output-destination-keys-and-schema-overrides

## Summary of Issue #1
File placement currently piggybacks on `filename_template` (e.g., prefixing `AI-IMP/…`). This is brittle and leaks directory structure into templates. Scope: introduce explicit destination keys so authorship is declarative: a global mapping under `[config.authoring.destinations]` (schema→relative path) and a per-schema override `[schema.new] output_path`. `cli-rag ai new submit` should write to the resolved destination without requiring folder segments in `filename_template`. Done when ADR/IMP/EPIC notes land in the right folders using only `{{id}}-{{title}}.md`-style templates. {LOC|20}

### Out of Scope 
- Moving or auto-migrating existing files.
- Changing ID generation or template rendering semantics.
- Editor integrations or UI prompts. {LOC|10}

### Design/Approach  
- Config additions:
  - Global: `[config.authoring.destinations]` table mapping schema name to a path relative to the repo config directory (e.g., `ADR="docs/RAG/ADRs"`).
  - Schema-level: `[schema.new] output_path = "docs/RAG/AI-IMP"` override (wins over global).
- Precedence: `schema.new.output_path` > `config.authoring.destinations[SCHEMA]` > first configured base.
- Validation rules: destination must resolve inside one of the configured bases; if not, exit 4 with a clear error.
- Backward compatibility: keep supporting path segments in `filename_template`, but document the new keys as the recommended approach.
- Docs: update README, template examples, and CI snippets to remove folder segments from `filename_template` once keys are in place. {LOC|25}

### Files to Touch
- `contracts/global-conventions.md` (if documenting new config knobs).
- `contracts/v1/config/user_config/cli-rag.toml` (example destinations mapping in comments).
- `src/config/schema.rs` (add `output_path` to `SchemaNewCfg`; add `destinations` to config model).
- `src/config/loader.rs` (parse nested keys; normalize into final `Config`).
- `src/commands/ai_new/start.rs` (resolve destination using the precedence when reserving drafts).
- `src/commands/ai_new/submit.rs` (write final note to the resolved destination; guard path traversal).
- Integration tests for `ai new` covering global mapping, per-schema overrides, and path traversal.
- `README.md` and `.github/workflows/ci.yml` (update examples/snippets once implemented). {LOC|25}

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [ ] Model: add `authoring.destinations` (HashMap<String,String>) to `Config`; add `output_path` (Option<String>) to `SchemaNewCfg`.
- [ ] Loader: parse nested TOML for these keys; normalize to absolute paths at runtime by joining with config dir; store relative for serialization.
- [ ] AI draft flow: implement destination resolution precedence in `ai new start/submit`; ensure final path is within a configured base (canonicalize + prefix check). Exit 4 on violation with actionable message.
- [ ] Back-compat: if no destination keys are set, behavior remains unchanged; if both keys and folder segments in `filename_template` are present, the segments append under the resolved destination.
- [ ] Tests: 
  - Global mapping writes ADR to `docs/RAG/ADRs`, IMP to `docs/RAG/AI-IMP`.
  - Per-schema `output_path` overrides global mapping.
  - Path traversal protection rejects `../../outside`.
- [ ] README: document the keys, precedence order, examples; steer users away from hardcoding folders in `filename_template`.
- [ ] CI: adapt contracts job or add a small step exercising destination keys (optional if covered by tests).

### Acceptance Criteria
**Scenario:** Global destinations mapping
GIVEN `[config.authoring.destinations] ADR="docs/RAG/ADRs" IMP="docs/RAG/AI-IMP"`
WHEN running `cli-rag ai new start --schema ADR --title One --format json` and submitting the draft
THEN the note is written under `docs/RAG/ADRs/`
AND `cli-rag validate --format json` remains OK.

**Scenario:** Per-schema override wins
GIVEN the global mapping above and `[schema.new] output_path="docs/RAG/CustomADR"` in ADR schema
WHEN running `cli-rag ai new start --schema ADR --title Two --format json` and submitting the draft
THEN the note is written under `docs/RAG/CustomADR/`.

**Scenario:** Path traversal guarded
GIVEN an invalid `output_path="../../etc"`
WHEN running `cli-rag ai new start --schema ADR --title Four --format json`
THEN the command exits 4 with a clear message about invalid destination outside configured bases.

### Issues Encountered 
{LOC|20}

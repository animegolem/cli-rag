---
node_id: AI-ADR-005
tags:
  - init
  - onboarding
  - presets
status: proposed
depends_on:
  - ADR-001-cli-rag.toml
  - ADR-003d-v1.2-locked-CLI-commands
created_date: 2025-10-10
related_files:
  - contracts/v1/config/user_config/cli-rag.toml
  - contracts/v1/config/user_config/templates/ADR.toml
  - src/commands/init_cmd.rs
confidence_score: 0.86
---

# AI-ADR-005-init-interactive-first-run-and-presets

## Objective
We need an approachable first‑run experience that writes a contract‑aligned `.cli-rag.toml` and minimal templates without forcing users to learn every option up front. The goal is to support a “ctrl+F and tweak” workflow while keeping behavior deterministic for dogfooding and CI. This ADR defines the interactive `init` flow, safe defaults, and near‑term scope (what is intentionally not implemented yet).

## Decision
1) Interactive init (default when running `cli-rag init` with no flags)
- Welcome menu with three choices:
  - 1. Project Manager preset (dogfood) — Recommended
  - 2. Generic preset — Not implemented (NI), prints friendly message, exit 1
  - 3. Fully commented example (TOML/Lua) — Not implemented (NI), exit 1
- If `.cli-rag.toml` exists: offer View diff → Back up as `.cli-rag.toml.bak` → Overwrite → Cancel.

2) Project Manager preset output (aligned to contracts)
- Writes `.cli-rag.toml` mirroring `contracts/v1/config/user_config/cli-rag.toml` fields:
  - `[config] config_version = "0.1"`
  - `[config.scan] filepaths = ["docs/RAG"], index_path = ".cli-rag/index.json", hash_mode = "mtime", index_strategy = "content", standard ignore_globs, ignore_symlinks = true
  - `[config.authoring] editor = $EDITOR|nvim, background_watch = true`
  - `[config.graph]` and `[config.graph.ai]` use contract defaults
  - `[config.templates] import = [".cli-rag/templates/ADR.toml"]`
- Writes `.cli-rag/templates/ADR.toml` from the contract template with:
  - Documented template precedence (Lua → TOML → repo template → fallback)
  - Variables (`{{filename}}`, `{{schema.name}}`, `{{frontmatter}}`, filtered `{{now}}`, `{{LOC|N}}`)
  - `schema.validate.edges.wikilinks`: `min_outgoing = 1`, `min_incoming = 0`, `severity = "warning"`
  - Per‑edge policy examples; `cross_schema.allowed_targets` omitted (allow‑all)
- Init does not touch the index; users run `validate` to build the first index (and discover pre‑existing notes).
- No seed notes created.

3) Flags and machine modes
- `--preset project` bypasses prompts and writes the project preset.
- `--dry-run` prints would‑be files; `--silent` suppresses editor open.
- `--json` returns `{ protocolVersion, preset, created:[], updated:[], warnings:[]? }`.
- `--schema <NAME>` and `--separate` are accepted but currently ignored for this preset (future use).

4) Inline documentation and recommended prompts
- Inline comments in `.cli-rag.toml` and ADR template are phrased for “search-and‑edit” users; they mirror the contract docs.
- Recommended authoring prompts are included as comments in the template and will also surface in the CLI help/man outputs later.

5) Non‑goals (now)
- Generic preset and fully commented example (TOML/Lua) return NI (exit 1) with a friendly message.
- No `.cli-rag.lua` is written by default.

## Consequences
- Pros: Frictionless first run, contract‑aligned defaults, safe destinations under `docs/RAG`, and predictable validate/index flows. Clear on‑ramps for both humans (“ctrl+F”) and future AI flows.
- Cons: Preset breadth is intentionally limited; users wanting Lua or broader schemas must wait for 1.0 (or edit manually).
- Risks: Overwrite flow must be careful to back up prior configs; we mitigate via explicit confirmation and `.bak` creation.
- Follow‑ups: Implement presets 2/3; surface recommended prompts in CLI `--help` and docs; consider optional seed notes for ADRs.


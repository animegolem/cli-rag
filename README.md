# cli-rag

A CLI based system for creating and managing Obsidian compliant YAML front matter. This creates a simplified DAG that allows a local "region" to be called by an LLM.

You've found this way too early! Nothing here is ready for production. :) This will all be cleaned up over the next few days. But i would not use this right now.

## Commands (overview)

- `init` – scaffold `.cli-rag.toml` and templates
- `validate` – build unified index and run checks
- `info` – show config/index/cache and capabilities
- `search` – fuzzy browse with TODO/Kanban emitters
- `graph` / `path` – dependency views and shortest path
- `get` – AI-oriented neighborhood retrieval
- `ai index plan` – compute AI Index clusters and write a plan JSON
- `ai index apply` – apply a plan: write cache and optional tags

## CI Contracts Gates

The `contracts` job in `.github/workflows/ci.yml` now spins up a nested user config fixture, runs `validate --format json` to assert the resolved snapshot, exercises `new` id generators and filename templates, and checks `ai new start/list/cancel` flows against the schemas in `contracts/v1/cli/*.schema.json`. Extend this job when you add new contract surfaces so the smoke coverage stays representative.

## Dogfooding

- `.cli-rag.toml` in the repo root defines the nested config (scan bases, graph defaults, template imports).
- Schemas live under `.cli-rag/templates/{ADR,IMP,EPIC}.toml`; these TOML files define id generators, filename rules, and tracked frontmatter, while their paired Markdown files provide the authored content scaffolds and guidance comments.
- Use `cli-rag new --schema ADR|IMP|EPIC --title ...` when drafting new work so the generated notes stay schema-compliant.

Preview a schema's scaffold without writing a file:

```
cli-rag new --schema ADR --title "Template Parity" --print-body
```

### ai index plan

Compute communities (clusters) over the unified graph and emit a plan JSON for labeling/summarization.

Usage:

```
cli-rag --config ./.cli-rag.toml validate --format json
cli-rag --config ./.cli-rag.toml ai index plan \
  --edges depends_on,mentions \
  --min-cluster-size 3 \
  --output .cli-rag/cache/plan.json
```

Notes:
- Reads the unified index at `<config_dir>/<index_relative>`; run `validate` first.
- `sourceIndexHash` is `sha256:<hex>` of the unified index bytes.
- Clusters are connected components over the selected edge kinds; members and cluster IDs are deterministic.
- Output matches `contracts/v1/cli/ai_index_plan.schema.json`.

### ai index apply

Apply a plan to write the authoritative cache and optionally add tags to note frontmatter.

Usage:

```
cli-rag --config ./.cli-rag.toml ai index apply \
  --from .cli-rag/cache/plan.json \
  --write-frontmatter \
  --dry-run
```

Notes:
- Validates plan.sourceIndexHash against the current unified index (exit 2 on mismatch).
- Writes cache to `.cli-rag/cache/ai-index.json` by default.
- Legacy aliases `ai-index-plan` / `ai-index-apply` remain available for one release and print a deprecation warning.
- Tag writes: enable with `--write-frontmatter`. Additive and require an existing `tags` field in frontmatter; otherwise exit 4.
- Apply report matches `contracts/v1/cli/ai_index_apply_report.schema.json`.

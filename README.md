# cli-rag

A CLI based system for creating and managing Obsidian compliant YAML front matter. This creates a simplified DAG that allows a local "region" to be called by an LLM.

You've found this way too early! Nothing here is ready for production. :) This will all be cleaned up over the next few days. But i would not use this right now.

## Commands (overview)

- `init` – scaffold `.cli-rag.toml` and optional schema templates
- `info` – inspect resolved config, caches, overlays, and capabilities
- `validate` – rebuild the unified index and surface validation diagnostics
- `watch` – stream incremental validation/graph updates while editing
- `search` – fuzzy browse notes with TODO/Kanban emitters
- `get` – retrieve a note plus its neighbors for AI prompting contexts
- `graph` / `path` – render dependency graphs or the shortest path between notes
- `ai` – umbrella for AI-first workflows (`new` and `index` subcommands)

## Quickstart: AI authoring

1. Make sure your repository has a `.cli-rag.toml` configured (or run `cli-rag init` to scaffold one).
2. Rebuild the unified index and contracts snapshot:
   ```bash
   cli-rag validate --format json
   ```
3. Reserve a draft with schema-specific guidance:
   ```bash
   cli-rag ai new start --schema ADR --title "Circuit Breaker" --format json > start.json
   ```
4. Inspect the scaffold and prepare content (JSON or Markdown):
   ```bash
   jq -r '.noteTemplate' start.json
   ```
5. Submit the filled sections/frontmatter to finalize the note:
   ```bash
   cli-rag ai new submit --draft "$(jq -r '.draftId' start.json)" --sections payload.json
   ```

## CI Contracts Gates

The `contracts` job in `.github/workflows/ci.yml` spins up a nested user config fixture, runs `validate --format json` to assert the resolved snapshot, exercises schema id generators/filename templates through `ai new start`, and checks `ai new start/list/cancel` flows against the schemas in `contracts/v1/cli/*.schema.json`. Extend this job when you add new contract surfaces so the smoke coverage stays representative.

## Dogfooding

- `.cli-rag.toml` in the repo root defines the nested config (scan bases, graph defaults, template imports).
- Schemas live under `.cli-rag/templates/{ADR,IMP,EPIC}.toml`; these TOML files define id generators, filename rules, and tracked frontmatter, while their paired Markdown files provide the authored content scaffolds and guidance comments.
- Authoring linkage: the TOML and Markdown files share the same stem (e.g., `ADR.toml` + `ADR.md`). The schema TOML configures discovery/id/frontmatter/output rules, and `cli-rag ai new start` returns the Markdown scaffold via `.noteTemplate`, injecting frontmatter with `{{frontmatter}}` and respecting `{{LOC|N}}` caps.
- Configure `[config.authoring.destinations]` in `.cli-rag.toml` so each schema writes to the correct folder (e.g., `ADR = "docs/RAG/ADRs"`), keeping `filename_template` focused on the basename like `{{id}}-{{title|kebab-case}}.md`.
- Prefer the AI workflow: `cli-rag ai new start --schema ADR --title ...`, edit the generated draft, then `cli-rag ai new submit --draft <id> --sections note.md`.

Preview a schema's scaffold without writing a file:

```
cli-rag ai new start --schema ADR --title "Template Parity" --format json \
  | jq -r '.noteTemplate'
```

> **Migrating from `cli-rag new`** – The legacy `new` subcommand has been removed. Use `cli-rag ai new start|submit|cancel|list` for all authoring flows. Existing templates, Lua overrides, and filename rules continue to apply through the AI draft surfaces.

### ai new start / submit / cancel / list

Manage schema-guided drafts without writing files until you are ready:

```
# Reserve an ADR draft and inspect guidance
cli-rag --config ./.cli-rag.toml ai new start \
  --schema ADR \
  --title "Circuit Breaker" \
  --format json \
  > start.json

jq -r '.noteTemplate' start.json

# Prepare sections/frontmatter payload for submission
cat >payload.json <<'JSON'
{
  "frontmatter": {
    "tags": ["ai", "adr"],
    "priority": "high"
  },
  "sections": {
    "Objective": "Document fallback strategy for unstable dependencies.",
    "Context": "Service calls external dependency with intermittent timeouts...",
    "Decision": "Introduce a circuit breaker using library X...",
    "Consequences": "Allows graceful degradation but adds operational tuning.",
    "Updates": "Populate after rollout"
  }
}
JSON

# Finalize the draft using the reserved ID (or pipe with --stdin)
cli-rag --config ./.cli-rag.toml ai new submit \
  --draft "$(jq -r '.draftId' start.json)" \
  --sections payload.json

# List or cancel active drafts when needed
cli-rag ai new list
cli-rag ai new cancel --draft "$(jq -r '.draftId' start.json)"
```

> Tip: `cli-rag ai new cancel` without `--draft` will automatically cancel the lone active draft. If multiple drafts exist, the command returns a structured error listing the available IDs so you can choose explicitly.

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

## Shell Completions

Generate completions to streamline the new command layout:

```
cli-rag completions bash > ~/.local/share/bash-completion/cli-rag
cli-rag completions zsh  > ~/.zsh/completions/_cli-rag
```

Each invocation reflects the latest `ai new` and `ai index` subcommands. Re-run after upgrading the CLI to refresh the definitions.

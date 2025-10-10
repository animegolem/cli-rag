# cli-rag

Per‑repo notes graph with schema‑aware authoring, validation, and AI surfaces.

cli‑rag manages Obsidian‑style Markdown with YAML frontmatter, builds a unified
graph (IDs, dependencies, mentions), validates per‑schema rules, and exposes
structured outputs for human and AI workflows.

Status: alpha/dogfooding. Contracts live under `contracts/v1/**` and the CLI
aligns with ADR‑003d (v1.2 locked commands).

## Command Overview

- `init` – scaffold `.cli-rag.toml` and optional schema templates
- `info` – inspect resolved config, caches, overlays, and capabilities
- `validate` – rebuild the unified index and report diagnostics
- `watch` – stream incremental index/validation updates (NDJSON option)
- `search` – fuzzy search notes with filters
- `get` – retrieve a note plus neighbor metadata for AI contexts
- `cluster` – explore dependency clusters around a note
- `graph` / `path` – export graph or compute a shortest path
- `ai` – AI‑first workflows (`new` and `index` subcommands)

## Quickstart: AI authoring

1. Ensure your repo has a `.cli-rag.toml` (or run `cli-rag init`).
2. Rebuild the unified index and contracts snapshot:
   ```bash
   cli-rag validate --format json
   ```
3. Reserve a draft with schema‑specific guidance:
   ```bash
   cli-rag ai new start --schema ADR --title "Circuit Breaker" --format json > start.json
   ```
4. Inspect the scaffold and prepare content (JSON or Markdown):
   ```bash
   jq -r '.noteTemplate' start.json
   ```
5. Submit filled sections/frontmatter to finalize the note:
   ```bash
   cli-rag ai new submit --draft "$(jq -r '.draftId' start.json)" --sections payload.json
   ```

## CI Contracts Gates

The CI job validates CLI outputs against schemas in `contracts/v1/**`. It runs
`validate --format json`, exercises `ai new start|submit|list|cancel`, and
ensures emitted JSON conforms to the CLI schemas under `contracts/v1/cli/`.
Extend CI when adding new surfaces so coverage remains representative.

## Dogfooding

- `.cli-rag.toml` at repo root defines scan bases, graph defaults, and template
  imports. Default index path: `.cli-rag/index.json`.
- Schemas live under `.cli-rag/templates/*.toml`; paired Markdown files are
  authoring scaffolds. The TOML configures ids, filename rules, and tracked
  frontmatter; `ai new start` returns the Markdown scaffold as `.noteTemplate`.
- Configure `[config.authoring.destinations]` so each schema writes to the
  correct folder (e.g., `ADR = "docs/RAG/ADR"`). Keep `filename_template`
  focused on the basename like `{{id}}-{{title|kebab-case}}.md`.
- Prefer the AI workflow: `ai new start` → edit draft → `ai new submit`.

Preview a schema scaffold without writing a file:

```
cli-rag ai new start --schema ADR --title "Template Parity" --format json \
  | jq -r '.noteTemplate'
```

> Migrating from `cli-rag new`: the legacy `new` command has been removed.
> Use `cli-rag ai new start|submit|cancel|list` for authoring. Existing
> templates, Lua overlays, and filename rules still apply via AI drafts.
## Commands

### Global flags

- `--config <path>` choose config (defaults to discovering `.cli-rag.toml`)
- `--base <p1,p2,...>` additional scan bases (alias: `--filepaths`)
- `--no-lua` disable Lua overlays entirely
- `--format {plain,json,ndjson,ai}` output format when supported

### init

Scaffold `.cli-rag.toml` and optional schema templates, then open the config in
your editor (unless `--silent`). If a parent config exists, `init` warns to
avoid accidental shadowing.

Flags:
- `--path <p>` custom config path (default `.cli-rag.toml`)
- `--force` overwrite if config exists
- `--print-template` print example config to stdout only
- `--silent` do not open editor after writing
- `--schema <NAME>` add a schema to the config
- `--separate` write schema under `.cli-rag/templates/<NAME>.toml` and import it
- `--preset <project|generic>` choose preset non‑interactively (`generic` is not
  yet implemented and returns a JSON warning when `--json` is set)
- `--dry-run` preview changes; do not write
- `--json` emit a summary of created/updated files

### info

Show resolved config path/version, index/cache presence, and capability flags.
Use `--format json` for machine output.

### validate

Rebuild the unified index and run schema validation. Fails on errors.

Flags:
- `--dry-run` compute but do not write the index
- `--full-rescan` ignore incrementals and rescan all
- `--format json` structured report per `contracts/v1/cli/validate*.schema.json`

Validation includes:
- Edge rules: required edges and cycle detection with severity fallback
- Wikilinks: unique outgoing/incoming thresholds per schema
- Cross‑schema: optional allowlists for target schemas

### watch

Watch for file changes, incrementally update index, and emit events.

Flags:
- `--full-rescan` force a rebuild on first run
- `--debounce-ms <n>` debounce FS events (default 400)
- `--dry-run` do not write index
- `--json` emit NDJSON events (`validated`, `index_written`, etc.)

### search

Fuzzy search with basic filters. Outputs plain lists or JSON envelopes.

Flags:
- `--query <q>` substring query
- `--kind <k1,k2>` filter by item kind (e.g., note,todo)
- `--schema <s1,s2>` filter by schema(s)
- `--status <st1,st2>` filter by status values
- `--tag <t1,t2>` filter by tags

### get

Retrieve a note with its neighborhood for AI prompting.

Flags:
- `--id <ID>` note id to fetch
- `--include-dependents` include backlinks in neighbors
- `--neighbor-style <STYLE>` metadata|outline|full (JSON only)
- `--depth <n>` neighbor depth (JSON only)
- `--max-fanout <n>` neighbor fanout cap (JSON only)

### cluster

Explore clusters around a note (dependency neighborhoods).

Flags:
- `--id <ID>`
- `--depth <n>`
- `--include-bidirectional <bool>`

### path

Compute a shortest dependency path between notes.

Flags:
- `--from <ID>`
- `--to <ID>`
- `--max-depth <n>` (default 5)

### graph

Export a dependency graph.

Flags:
- `--id <ID>`
- `--depth <n>`
- `--include-bidirectional <bool>`
- `--graph-format {mermaid,dot,json}`

### ai new (start / submit / cancel / list)

Manage schema‑guided drafts without writing files until you are ready:

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

### completions

Generate shell completions:

```
cli-rag completions bash > ~/.local/share/bash-completion/cli-rag
cli-rag completions zsh  > ~/.zsh/completions/_cli-rag
```

Re‑run after upgrading to refresh definitions.

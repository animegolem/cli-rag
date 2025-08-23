# adr-rag (Repo-local ADR CLI)

A small CLI to navigate/search/doctor ADRs using a simple per-repo TOML config. No dependency on Obsidian; works by scanning markdown + front matter.

## Project Layout

- `src/lib.rs`: Library crate exporting modules
  - `config` (TOML config, defaults, schema sets)
  - `model` (front-matter parsing, AdrDoc, file metadata)
  - `discovery` (scan, incremental collection, index loading)
  - `index` (write JSON index and semantic groups)
  - `validate` (schema + rules, returns `ValidationReport`)
  - `graph` (BFS path, dependency cluster)
  - `util` (helpers like `try_open_editor`)
  - `watch` (debounced filesystem watcher orchestration)
  - `cli` (Clap definitions: `Cli`, `Commands`, `ValidateArgs`)
  - `commands/*` (one handler per subcommand)
- `src/bin/adr-rag.rs`: Thin binary that parses CLI and delegates to the library

## Commands
- `init` — Create `.adr-rag.toml` in the current repo and open it in an editor by default. Flags:
  - `--force` overwrite if exists
  - `--print-template` print template to stdout
  - `--silent` do not open the config after creating/detecting it
- `doctor` — Show resolved config, bases, discovery mode (index vs scan), and quick stats.
  - Reports per-type counts when schemas are defined, and unknown-key stats.
  - JSON: use global `--format json`.
- `search --query <substr>` — Fuzzy search by ID/title across discovered ADR files.
- `topics` — List semantic groups derived from front matter (`groups` in ADRs).
- `group --topic "<name>" [--include-content]` — Show ADRs in a group; optionally include full content.
- `get --id ADR-021 [--include-dependents]` — Print an ADR with dependencies (and dependents if requested).
- `cluster --id ADR-021 [--depth N] [--include-bidirectional]` — Traverse dependencies (and dependents) to depth.
- `graph --id ADR-021 [--depth N] [--include-bidirectional] [--format mermaid|dot|json]` — Export a dependency graph around an ADR.
- `path --from ADR-011 --to ADR-038 [--max-depth N]` — Find a dependency path if any.
 - `validate [--format json] [--dry-run] [--full-rescan] [--write-groups]` — Validate front matter/refs; on success writes indexes (unless `--dry-run`).
  - Incremental by default: only reparses changed files using mtime/size. Use `--full-rescan` to force scanning all.
  - Exits non-zero if validation fails.
 - `watch [--debounce-ms 400] [--dry-run] [--full-rescan]` — Watch bases for changes and incrementally validate + update indexes.
  - Debounces rapid events; writes on success (unless `--dry-run`).

## Config: `.adr-rag.toml`
Created by `adr-rag init`. Example:

```
# Repo-local ADR CLI config (adr-rag)

bases = [
  "docs/masterplan",
  # "docs/notes",
]

index_relative = "index/adr-index.json"
groups_relative = "index/semantic-groups.json"

file_patterns = ["ADR-*.md", "ADR-DB-*.md", "IMP-*.md"]
ignore_globs  = ["**/node_modules/**", "**/.obsidian/**"]
allowed_statuses = [
  "draft", "incomplete", "proposed", "accepted",
  "complete", "design", "legacy-reference", "superseded"
]

[defaults]
depth = 2
include_bidirectional = true
include_content = true

## Note Types (Schemas)
- Define `[[schema]]` blocks to validate different note types (e.g., ADR vs IMP).
- Unknown keys policy controls how unexpected front-matter is treated.
- Defaults: `unknown_policy = "ignore"`; required fields must be non-empty.

# Enabled defaults (edit as needed)
[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md", "ADR-DB-*.md"]
required = ["id", "tags", "status", "depends_on"]
unknown_policy = "ignore"

[schema.rules.status]
allowed = [
  "draft", "incomplete", "proposed", "accepted",
  "complete", "design", "legacy-reference", "superseded"
]
severity = "error"

[[schema]]
name = "IMP"
file_patterns = ["IMP-*.md"]
required = ["id", "tags", "depends_on", "status"]
unknown_policy = "ignore"

[schema.rules.status]
allowed = ["in-progress", "blocked", "on-hold", "cancelled", "done"]
severity = "error"
```

## Build & Run
- From repo root:
  - `cd tools/adr-rag`
  - `cargo build --release`
  - `./target/release/adr-rag init`
  - `./target/release/adr-rag doctor`
  - `./target/release/adr-rag doctor --format json`
  - `./target/release/adr-rag search -q sidecar`
  - `./target/release/adr-rag topics`
  - `./target/release/adr-rag group --topic "Tools & Execution"`
  - `./target/release/adr-rag get --id ADR-021`
  - `./target/release/adr-rag cluster --id ADR-021 --depth 3`
  - `./target/release/adr-rag graph --id ADR-021 --format mermaid`
  - `./target/release/adr-rag graph --id ADR-021 --format dot`
  - `./target/release/adr-rag graph --id ADR-021 --format json`
  - `./target/release/adr-rag path --from ADR-011 --to ADR-038`
  - `./target/release/adr-rag validate --format json --write-groups`

### Global Flags
- `--config <path>`: Explicit config path; otherwise searches upward for `.adr-rag.toml`.
- `--base <path1,path2>`: Override bases from config/env (comma-separated).
- `--format <plain|json>`: Output format for all commands.

Note: The `graph` subcommand uses its own `--format` allowing `mermaid`, `dot`, or `json`.

### Environment Variables
- `ADR_RAG_CONFIG`: Override config path (lower precedence than `--config`).
- `ADR_RAG_BASES`: Comma-separated bases override (lower precedence than `--base`).

Precedence: CLI flags > env vars > nearest `.adr-rag.toml` > tool defaults.

### Shell Completions
- Generate completions: `adr-rag completions bash|zsh|fish`.
- Example (bash): `adr-rag completions bash > ~/.local/share/bash-completion/adr-rag` then `source` it, or add to your shell init.

## Notes
- Multi-base merging is supported; results are de-duplicated by `id` (conflicts will be flagged in future `validate`).
- Front matter groups (e.g., `groups: ["Tools & Execution"]`) drive `topics`/`group`.
- See `docs/masterplan-v2/IMP-004-adr-rag-cli-and-config.md` for design details.

Index and Groups behavior
- Commands load from an index at `<base>/<index_relative>` if present; otherwise they scan markdown.
- `validate` scans markdown and, if there are no blocking errors, writes/upserts per-base JSON indexes by default.
- `validate --dry-run` prints errors/warnings but does not write.
 - Index entries include minimal metadata (`mtime`, `size`) to accelerate incremental runs.
- `validate --write-groups` also writes the semantic groups JSON to `<base>/<groups_relative>`.
- `topics` reads `groups_relative` if present; otherwise derives topics from ADR front matter.

Isolated ADRs
- ADRs with no `depends_on` and no inbound dependents are valid. `validate` will not fail but will emit warnings listing isolated IDs.

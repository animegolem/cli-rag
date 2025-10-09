# Contracts Change Log

## 2025-10-08: Help/README/completions alignment for AI namespace + alias removal

### Reason for change
- Reflect the unified `cli-rag ai` hierarchy across help output and docs as we finalize the AI-first surface.
- Remove the legacy `ai-index-plan` / `ai-index-apply` aliases now that downstream usage has migrated.
- Provide an explicit migration note and quickstart showcasing the draft workflow.

### Overview of change
- src/cli.rs / src/bin/cli-rag.rs: removed legacy alias commands entirely; help output now lists only the unified `ai` hierarchy.
- README.md: reorganized command overview, added an AI authoring quickstart, refreshed the migration note, and documented `ai new cancel` auto-selection behavior when only one draft exists.
- ai new cancel: now omits `--draft` when a single draft is active and surfaces a structured error listing options when multiple drafts exist.
- This work satisfies AI-IMP-026 and AI-IMP-027 prior to closing the tickets; shell completions continue to cover the refreshed hierarchy.

## 2025-09-19: Validate JSON envelope parity and info schema cleanup

### Reason for change
- Align `validate --format json` with global conventions by emitting `protocolVersion`.
- Ensure diagnostics tolerate `null` paths when a file cannot be located.
- Remove duplicate property definitions in `info` schema to avoid drift.

### Overview of change
- cli/validate_result.schema.json: require top-level `protocolVersion` (integer ≥1) and allow diagnostic `path` to be `string|null`.
- src/commands/validate_cmd.rs: emit `protocolVersion` alongside doc counts and diagnostics.
- tests/integration_validate_json.rs: assert the presence of `protocolVersion`.
- cli/info.schema.json: remove the duplicate `cache` property block.

## 2025-09-19: Authoring destination keys

### Reason for change
- Decouple note placement from filename templates and make destinations declarative.

### Overview of change
- config schema: new `[config.authoring.destinations]` mapping and per-schema `output_path` (string or array) override.
- AI authoring (`ai new start/submit`): resolves destinations with precedence schema `output_path` > authoring destinations > first base; rejects paths outside configured bases (exit 4).
- Templates: repository schemas now set `output_path` and use basename-only filename templates.
- README/config samples updated to document the new keys and precedence.

## 2025-09-19: Remove legacy `new` command

### Reason for change
- Eliminate the duplicated authoring surface now that AI-first flows cover note creation.

### Overview of change
- CLI: `cli-rag new` subcommand removed; authors must use `cli-rag ai new start|submit|cancel|list`.
- Docs/tests: README and integration coverage updated to reference AI-first authoring; legacy env toggles removed.

## 2025-09-18: Unified `ai index` namespace and deprecated aliases

### Reason for change
- Align CLI surfaces with ADR-003d and reduce confusion between top-level `ai-index-*` commands and the `ai` namespace.

### Overview of change
- CLI: expose `cli-rag ai index plan|apply` as the preferred entrypoints.
- CLI: retain `ai-index-plan` / `ai-index-apply` as aliases that emit a deprecation warning for one release window.
- Docs: update README and examples to reference the unified namespace.
- Completions/help refreshed to match the new hierarchy.

## 2025-09-12: Add protocolVersion to graph/path

### Reason for change
- Align with global convention: all top-level JSON responses include `protocolVersion`.

### Overview of change
- cli/graph.schema.json: require `protocolVersion` (integer, min 1) at top-level.
- cli/path.schema.json: require `protocolVersion` (integer, min 1) at top-level.
- Implementations updated to emit `protocolVersion` in outputs.

## 2025-09-12: GTD schema polish and casing normalization

### Reason for change
- Reduce ambiguity around kanban field casing across surfaces.
- Enrich TODO items to support Agenda-like views without over-specifying UI semantics.
- Expose minimal GTD context in ai_get and advertise capabilities for UI adapters.
- Formalize CLI flag casing to avoid drift.

### Overview of change
- search_result.schema.json
  - note.kind=note: add optional `kanbanStatusLine`.
  - note.kind=kanban: standardize `kanbanStatusLine` casing (was sometimes `kanbanStatusline` in docs).
  - todo.kind: add optional `dueDate` (date), `source` ("body"|"frontmatter"), `span` ([start,end]), `priorityScore` (1–10).
- ai_get.schema.json
  - neighbors[]: add optional `kanbanStatus` and `dueDate`.
  - root: add optional `kanbanStatus` and `kanbanStatusLine` (high-level descriptor context) and optional `dueDate`.
- info.schema.json
  - capabilities: allow optional `gtdTasks` and `kanban` booleans.
- global-conventions.md
  - Document that CLI long flags use kebab-case (e.g., `--graph-format`).

Notes: These additions are optional fields; the only casing normalization is aligning on `kanbanStatusLine`. Contracts remain the source of truth.

## 2025-09-18: Ability to define write paths for "ai new" and functions via the gui

### Reason For Change 

- Allow granular control of the write path for new notes. This should be allowed on both the .cli-rag.toml for project wide settings (eg dump all notes in an unsorted bucket, let the application handle it) or given schema specific overrides (~/RAG/ADR)

### Overview of Change 

**Additions made in ADR.toml** 

```
[schema.new]
#: Define the name template for the `new` command. unset = filename
id_generator = { strategy = "increment", prefix = "ADR-", padding = 3 }
#: Options are ["increment", "datetime", "uuid"]
#: Prefix is not mandatory if using the later two options
filename_template = "{{id}}-{{title|kebab-case}}.md"
#: Define the default write path for notes in this schema. Overides settings in
#: .cli-rag.toml.
output_path = [ "docs/RAG/ADR" ]
```

**Additions made in .cli-rag.toml**

```
#: =============================================================================
#:                            # --- AUTHORING --- #
#: =============================================================================
#: Settings related to creating and editing notes
[config.authoring]
#: The editor to launch for new or existing notes.
#: Uses $EDITOR or $VISUAL if not set.
editor = "nvim"
#: runs the `watch` command as a background process for live updates when
#: visual mode is open and active. default = true
background_watch = true
#: Define the default write path for all new notes. Can be overidden per
#: schema.
output_path = "docs/RAG"
```

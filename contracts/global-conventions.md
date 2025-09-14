# Locked conventions v1

## Project-wide naming and casing
- TOML: snake_case keys. No hyphens in keys.
- Lua: snake_case for keys and hook names. Module returns table with fields mirroring TOML (snake_case).
- JSON outputs: camelCase keys. Exception: edge kind names and schema names are emitted as-is (e.g., "depends_on", "ADR").
- Frontmatter (YAML): snake_case keys.

### CLI flags
- Long flags use kebab-case (e.g., `--graph-format`, `--full-rescan`).
- Short flags remain single letters where applicable (e.g., `-q`).

### IDs, dates, times
- Note IDs: schema-specific; recommended regex default ^[A-Z]{2,5}-\d{3,}$ unless overridden.
- Date-only: YYYY-MM-DD.
- Timestamps: RFC 3339/ISO 8601 UTC, e.g., 2025-09-08T12:14:00Z.

### Text positions and locations
- Line numbers are 1-based.
- Byte/char spans in diagnostics are half-open [start, end). If only line/col are available, include a minimal span or the line number.

### Deterministic ordering
- search: score desc → lastModified desc → id asc
- neighbors (ai get): distance asc → score desc → lastModified desc → id asc
- graph/path: stable id asc for nodes at same depth; edges sorted by from,id,to,id,kind

### Exit codes
- 0 success
- 1 unexpected error
- 2 validation/contract failure (bad plan, failed checks, policy violations)
- 3 draft not found/expired (ai new only)
- 4 schema/config error
- 5 IO/index lock error

### Version signaling
- All top-level JSON responses include `protocolVersion`.
- `ai get` includes `retrievalVersion`.
- `info` shape:
  - `capabilities.luaApiVersion` (integer, Lua hooks API major version)
  - `config.version` (string; version from TOML)
- ResolvedConfig shape:
  - top-level `configVersion` (string; from TOML `config_version`)
  - top-level `luaApiVersion` (integer)
- TOML `config_version` is a string (e.g., "0.1").
  - Note: Lua API may have point releases; `luaApiVersion` signals the major. Minor, non-breaking changes do not bump the major. A semver string may be added later as `luaApiSemver`.

### Template system
- Variables: {{id}}, {{title}}, {{schema.name}}, {{now | date:"%Y-%m-%d"}}
- Directives: ((frontmatter)) injects generated/system/user frontmatter. {{LOC|N}} caps lines per heading.
- filename_template uses same variable filters (| kebab-case, | snake_case, etc.).
- Headings policy labels: exact | missing_only | ignore.

### Validation semantics
- Severity: "error" | "warning" | "ignore".
- Frontmatter field rules:
  - regex: single string
  - globs: array of strings
  - enum: array of strings
  - integer/float: { min?, max? }
- Edges:
  - required: "error" | "warning" | "ignore".
  - cycle_detection: "error" | "warning" | "ignore".
  - wikilinks: min_outgoing, min_incoming (ints).
  - cross_schema.allowed_targets: array of schema names.

### Graph/AI defaults
- config.graph: { depth: 1, include_bidirectional: true }
- config.graph.ai: { depth: 1, default_fanout: 5, include_bidirectional: true, neighbor_style: "metadata", outline_lines: 2 }
- ai get policy:
  - If neighbor_style=full and depth>1 → exit 2 (NEIGHBORS_FULL_DEPTH_GT1), or require explicit override if you later add one.

### Watch/streaming
- NDJSON watch: first event must be {"event":"watch_start","protocolVersion":1}.
- Subsequent events include event type and minimal payload.

### Path/edges
- edges.locations.line is 1-based.
- Edge kind names are case-sensitive strings; prefer snake_case.

### Casing map (TOML → JSON)
- index_path → indexPath
- include_bidirectional → includeBidirectional
- default_fanout → defaultFanout
- neighbor_style → neighborStyle
- outline_lines → outlineLines
- last_modified (frontmatter) → lastModified (computed/index)
- kanban_statusline (frontmatter) → kanbanStatusLine (JSON outputs)

## Lua hook API v1 (stable signatures)
- All hooks optional. Use snake_case names.

### Signatures
- id_generator(schema, ctx) → { id, filename? }
- render_frontmatter(schema, title?, ctx) → table
- template_prompt(ctx) → string|nil (optional; may be ignored by CLI)
- template_note(ctx) → string|nil (optional; may be ignored by CLI)
- validate(note, ctx) → { diagnostics: Diagnostic[] }

### Context (read-only)
- ctx.schema: resolved schema definition (table)
- ctx.config: resolved config (table)
- ctx.index: index handle with helpers (e.g., next_numeric_id(prefix): int)
- ctx.request: { title?: string, id?: string }
- ctx.util: { kebab_case(s), snake_case(s), pascal_case(s) }
- ctx.clock: { today_iso(), now_iso() }
- ctx.fs (optional, sandboxed): { exists(path): bool, read_file(path): string }

### Diagnostic
- { severity: "error"|"warning"|"info", code: string, msg: string, path?: string, span?: [number, number], field?: string, nodeId?: string }

## TOML → JSON/ResolvedConfig mapping notes
- Load order: defaults → TOML → repo Lua overlay (.cli-rag.lua) → user-local Lua (~/.config/cli-rag/config.lua), with a --no-lua switch to disable overlays.
- Normalize TOML snake_case to JSON camelCase when emitting ResolvedConfig or CLI outputs.
- Persist a resolved snapshot at .cli-rag/resolved.json after validate for editor/adapters.

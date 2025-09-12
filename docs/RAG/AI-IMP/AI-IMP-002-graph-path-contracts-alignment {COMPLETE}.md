---
node_id: AI-IMP-002
tags:
  - IMP-LIST
  - Implementation
  - cli
  - graph
  - path
kanban_status: in_progress
confidence_score: 0.78
created_date: 2025-09-11
---

# AI-IMP-002-graph-path-contracts-alignment

## Sub-Issue #1: Graph JSON alignment
Align `graph --format json` to `contracts/v1/cli/graph.schema.json` with `{root, nodes[], edges[]}` and edge `kind`.

### Files to Touch
- `src/commands/graph.rs`: emit contract-shaped JSON (`root{id}`, `nodes[{id,title?,schema}]`, `edges[{from,to,kind}]`).
- `src/discovery/unified.rs`: ensure loader continues to work; no change expected.
- `contracts/v1/cli/graph.schema.json`: reference only (no code changes).
- `tests/integration_graph_json.rs`: new tests for schema shape and deterministic ordering.

### Implementation Checklist
- [x] Update `graph.rs` JSON branch to emit `{protocolVersion, root:{id}, nodes:[{id,title?,schema}], edges:[{from,to,kind}]}`.
- [x] Populate `schema` on nodes (infer from filename via existing `build_schema_sets`).
- [x] Emit `kind: "depends_on"` for dependency edges (bidirectional setting does not duplicate kinds; only forward edges carry kind).
- [x] Ensure deterministic ordering: nodes sorted by `id` asc; edges sorted by `(from,to,kind)` asc.
- [x] Add `tests/integration_graph_json.rs` with a small fixture asserting shape and ordering.
- [x] Document in the help text that `--graph-format json` is the machine/AI surface.

### Acceptance Criteria
Given a repo with ADR-001 depending on ADR-002
When I run `cli-rag graph --id ADR-001 --format json`
Then the JSON validates against `contracts/v1/cli/graph.schema.json` and includes `edges:[{"from":"ADR-001","to":"ADR-002","kind":"depends_on"}]`.

## Sub-Issue #2: Path JSON alignment
Align `path --format json` to `contracts/v1/cli/path.schema.json` with `{ok, path[nodes], edges[{from,to,kind,locations}]}` including `locations` for mention-derived segments when available.

### Files to Touch
- `src/commands/path.rs`: compute shortest path and emit contract-shaped JSON; include `locations` for any mention-derived edges if line info is available.
- `src/discovery/unified.rs`: expose mention edges/locations for path resolution (already emitted in unified index writer).
- `contracts/v1/cli/path.schema.json`: reference only.
- `tests/integration_path_json.rs`: new tests for shape and locations.

- [x] Update/implement `src/commands/path.rs` to emit `{protocolVersion, ok, path:[{id,title?,schema}], edges:[{from,to,kind,locations}]}`.
- [x] Resolve nodes from unified index; include `schema` and optional `title` in path nodes.
- [x] Use edges from unified index: prefer `depends_on` edges; allow `mentions` when necessary; copy any `locations` from index for `mentions`.
- [x] Deterministic ordering for tie-breaking paths (stable BFS by `id`).
- [x] Add `tests/integration_path_json.rs` covering:
  - straight dependency path (no locations on depends_on)
  - path including a `mentions` edge with `locations[{path,line}]`.

### Acceptance Criteria
Given ADR-024 → IMP-006 (depends_on) and IMP-006 mentions ADR-029 at line 42
When I run `cli-rag path --from ADR-024 --to ADR-029 --format json`
Then output validates against `contracts/v1/cli/path.schema.json` and includes `edges:[{"from":"IMP-006","to":"ADR-029","kind":"mentions","locations":[{"path":"IMP/IMP-006.md","line":42}]}]`.

## Sub-Issue #3: CI contracts validators for Graph/Path
Add CI gates to validate `graph` and `path` outputs against their schemas on a minimal fixture.

### Files to Touch
- `.github/workflows/ci.yml`: extend “Contracts Compliance” job to validate `graph` and `path` JSON outputs.
- `contracts/v1/cli/graph.schema.json`, `contracts/v1/cli/path.schema.json`: reference only.

### Implementation Checklist
- [x] In CI, create a temp fixture with two notes (ADR-001 depends on ADR-002) and run:
  - [x] `graph --id ADR-001 --graph-format json` → validate against `contracts/v0.1/cli/graph.schema.json`.
  - [x] `path --from ADR-001 --to ADR-002 --format json` → validate against `contracts/v0.1/cli/path.schema.json`.
- [x] Add a second fixture case exercising `mentions` edge on `path` with `locations`.

### Acceptance Criteria
CI “Contracts Compliance” job passes on PRs with validations for `graph` and `path` outputs against their respective schemas using the temp fixtures.

## Issues Encountered
See [[LOG-AI-2025-09-11b]]

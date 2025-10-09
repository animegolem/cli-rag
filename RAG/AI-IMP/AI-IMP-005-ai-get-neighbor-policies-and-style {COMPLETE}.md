---
node_id: AI-IMP-005
tags:
  - IMP-LIST
  - Implementation
  - ai_get
  - neighbors
  - policies
kanban_status: completed
depends_on: [AI-IMP-004]
confidence_score: 0.9
created_date: 2025-09-12
close_date: 2025-09-12
---

# AI-IMP-005-ai-get-neighbor-policies-and-style

## Summary of Issue #1
ai get (JSON surface) should fully enforce neighbor retrieval policies and expose neighborStyle variants per contracts and global conventions. We must: implement `neighborStyle` handling (metadata|outline|full), enforce limits (depth, maxFanout) including the policy that `neighborStyle=full` with `depth>1` is a contract violation, and add optional `contentOutline` for neighbors when requested. Outcome: `get --format json` accepts flags and emits outputs that validate against `contracts/v1/cli/ai_get.schema.json` with deterministic ordering and policy enforcement errors exiting with code 2.

### Out of Scope 
- AI index plan/apply surfaces.
- Embedding or model-powered scoring.
- Multi-hop content blocks beyond metadata/outline.

### Design/Approach  
- Add CLI flags: `--neighbor-style (metadata|outline|full)`, `--depth`, `--max-fanout` with defaults from global conventions.
- Enforce policy: if `neighbor-style=full` and `depth>1` → exit 2 with coded error per conventions.
- Compute neighbors deterministically (distance→score→mtime→id). Keep `score` null for now.
- When `neighbor-style=outline`, build `contentOutline` using first N lines per heading (N from config `graph.ai.outlineLines`, default 2).
- When `neighbor-style=metadata`, omit content fields on neighbors.
- Root object remains with full body in `content` (type=text), as already implemented.

### Files Touched
- `src/cli.rs`: added ai get flags for neighbor style, depth, max_fanout.
- `src/bin/cli-rag.rs`: wired new flags to handler.
- `src/commands/get.rs`: added BFS neighbors, policy checks, outline builder, deterministic ordering, fanout limit, style‑specific emission.
- `tests/integration_get_ai_neighbors.rs`: added tests for metadata vs outline and policy violation for full+depth>1.
- `.github/workflows/ci.yml`: ai_get JSON validation step added (schema check).

### Implementation Checklist
<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [x] CLI: add `--neighbor-style`, `--depth`, `--max-fanout` for `get`.
- [x] get: enforce `neighborStyle=full` with `depth>1` → exit 2 (NEIGHBORS_FULL_DEPTH_GT1).
- [x] get: implement outline extraction (first N lines per heading) and emit `contentOutline` only when style=outline.
- [x] get: keep neighbor `content` empty (metadata only) unless style=full.
- [x] get: deterministic ordering distance→score→lastModified→id.
- [x] Tests: metadata vs outline content emission.
- [x] Tests: policy violation returns exit code 2.
- [x] CI: validate ai_get JSON against `contracts/v1/cli/ai_get.schema.json`.

### Acceptance Criteria
**GIVEN** a repo with a few linked notes, **WHEN** running `get --id ADR-001 --format json --neighbor-style outline --depth 1`, **THEN** output includes neighbors with `contentOutline` entries and no `content` on neighbors, and validates against `contracts/v1/cli/ai_get.schema.json`.
**GIVEN** the same repo, **WHEN** running `get --neighbor-style full --depth 2`, **THEN** the command exits with code 2 and an explanatory policy error code `NEIGHBORS_FULL_DEPTH_GT1`.

### Issues Encountered 
- Tests require a unified index; integration tests call `validate --format json` before exercising `get` to ensure consistent discovery.

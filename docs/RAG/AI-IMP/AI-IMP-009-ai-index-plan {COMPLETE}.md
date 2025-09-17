---
node_id: AI-IMP-009
tags:
  - IMP-LIST
  - Implementation
  - ai-index
  - graph
  - clustering
kanban_status: done
depends_on:
  - ADR-003d
  - docs/RAG/BRIDGE_PLAN_V2.md
confidence_score: 0.74
created_date: 2025-09-14
close_date:
---

# AI-IMP-009-ai-index-plan

## Summary of Issue #1
Add `ai index plan` to compute clusters over the unified graph and emit a deterministic work order matching `contracts/v1/cli/ai_index_plan.schema.json`. Include `generatedAt`, `sourceIndexHash` (sha256 of unified index JSON), `params` (edges, minClusterSize, schema), and a stable list of clusters with metrics and placeholders for label/summary. Outcome: a reproducible plan file suitable for human/LLM labeling.

### Out of Scope 
- Advanced community detection (e.g., Louvain). Use connected components or degree-threshold subgraphs for v1.
- Embeddings/summarization; label/summary remain empty.
- Apply/cache write (handled by separate ticket).

### Design/Approach  
- Input: read unified index from `cfg.index_relative` at project root; error if missing (prompt to run `validate`).
- Hash: compute sha256 over the exact file bytes; prefix `sha256:`.
- Graph: build adjacency for allowed `edges` kinds (filter param); compute connected components; metrics `{ size, density }` where density = edges/(n*(n-1)) for undirected projection.
- Determinism: sort clusters by `size desc, representative id asc`; generate `clusterId` as `c_` + zero-padded index; choose first two IDs as `representatives`.
- Output: write JSON to `--output`; include `params` echo.

### Files to Touch
- `src/cli.rs`: add `ai index plan` subcommand and flags.
- `src/commands/ai_index_plan.rs`: new command implementing plan generation.
- `contracts/v1/cli/ai_index_plan.schema.json`: already present (no changes expected).
- Tests: integration test generating a small plan and validating with jsonschema (CI also validates).

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [ ] CLI: Add `ai index plan` with flags `--edges`, `--min-cluster-size`, `--output`, `--schema?`.
- [ ] Loader: Read unified index from project root; validate existence; return helpful error otherwise.
- [ ] Planner: Build graph filtered by edge kinds; compute connected components; compute metrics; representatives.
- [ ] Hash: Compute sha256 over unified index; embed as `sourceIndexHash`.
- [ ] Output: Serialize JSON matching the contract; write to `--output` path.
- [ ] Tests: Integration test on a small fixture repo; validate JSON with schema; verify deterministic cluster IDs/order.

### Acceptance Criteria
**GIVEN** a repo with a unified index present, **WHEN** running `ai index plan --edges depends_on --min-cluster-size 2 --output plan.json`, **THEN** `plan.json` exists, validates against the schema, and contains stable `clusterId`s and metrics.

**GIVEN** no unified index, **WHEN** running the command, **THEN** it errors clearly (non-zero) instructing to run `validate` first.

### Issues Encountered 
(to be completed during implementation)


---
node_id: AI-IMP-010
tags:
  - IMP-LIST
  - Implementation
  - ai-index
  - cache
  - tags
kanban_status: done
depends_on:
  - AI-IMP-009
  - ADR-003d
confidence_score: 0.73
created_date: 2025-09-14
close_date: 2025-09-14
---

# AI-IMP-010-ai-index-apply-cache-and-tags

## Summary of Issue #1
Add `ai index apply` to persist labeled clusters to an authoritative cache and optionally add tags to note frontmatter. Validate that `plan.sourceIndexHash` matches the current unified index unless overridden. Emit an apply report matching `contracts/v1/cli/ai_index_apply_report.schema.json`. Outcome: cache at `.cli-rag/cache/ai-index.json` reflects applied labels; optional tag writes occur when allowed.

### Out of Scope 
- Non-additive frontmatter mutation (no removals); complex conflict resolution.
- UI for manual label editing (external to CLI).

### Design/Approach  
- Inputs: `--from <plan.json>`, `--write-cache true|false` (default true), `--write-frontmatter true|false` (default false), `--dry-run`.
- Validation: if `plan.sourceIndexHash` present and mismatched, exit 2 by default.
- Cache write: write minimal authoritative cache `{version, clusters[]}` under `.cli-rag/cache/ai-index.json` (create directories as needed).
- Tag writes: if enabled, for each cluster with `label` non-empty, add that tag to members where schema has a `tags` field; skip with a warning if schema doesn’t support tags.
- Report: emit `{ ok, written: { cache, frontmatter }, clustersApplied, membersTagged, warnings[] }` and appropriate exit code (0 success; 2 validation mismatch; 4 schema error; 5 IO error).

### Files to Touch
- `src/cli.rs`: add `ai index apply` subcommand and flags.
- `src/commands/ai_index_apply.rs`: new command.
- `src/config/schema.rs`: utility to query if a schema defines `tags` (if needed for tag write policy).
- `contracts/v1/cli/ai_index_apply_report.schema.json`: already present (no change expected).
- Tests: integration test with a sample plan and dry-run; verify cache write and tag write behavior.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [x] CLI: Add `ai index apply` flags per design.
- [x] Parser: Read and validate `plan.json`; verify `sourceIndexHash` against current unified index when present (exit 2 on mismatch).
- [x] Cache: Write `.cli-rag/cache/ai-index.json` when enabled; ensure directories are created.
- [x] Tags: When enabled, add label-derived tag (kebab-case) or plan.tags; require existing `tags` field; additive, de-dup; exit 4 on schema/format error.
- [x] Report: Print contract-shaped JSON; set exit codes (2/4) per conventions.
- [x] Tests: Integration for dry-run; for write-cache+frontmatter; for mismatched hash (exit 2).

### Acceptance Criteria
**GIVEN** a valid plan with matching `sourceIndexHash`, **WHEN** running `ai index apply --from plan.json --write-cache true --write-frontmatter false`, **THEN** `.cli-rag/cache/ai-index.json` exists, JSON validates, and report indicates `written.cache=true` and `frontmatter=false`.

**GIVEN** `--write-frontmatter true`, **WHEN** the schema supports tags, **THEN** members receive the label tag in frontmatter; **AND** report shows `membersTagged>0`.

**GIVEN** hash mismatch, **WHEN** running apply, **THEN** process exits with code 2 and a clear message.

### Issues Encountered 
- None blocking; added strict guards for frontmatter tags format (sequence) to avoid schema ambiguity.

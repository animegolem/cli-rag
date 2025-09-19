---
id: ADR-015
tags: []
status: draft
depends_on: []
created_date: 2025-09-17
last_modified: 2025-09-17
related_files:
  - .cli-rag.toml
  - .cli-rag/templates/ADR.toml
---

# ADR-015: Dogfooding Config Practices

## Context
The repository previously relied on `.adr-rag.toml`, which no longer reflects the contracts-first loader. Without an updated config we could not safely dogfood new surfaces or rely on CI to enforce schema rules.

## Decision
Adopt `.cli-rag.toml` with the nested `[config.scan|authoring|graph|templates]` structure, import dedicated ADR/IMP/EPIC schemas, and require that future notes are authored through `cli-rag ai new` so that contract-aligned templates stay authoritative.

## Consequences
- Dogfooding exercises the same config that CI validates, reducing drift between the repo and the contracts job.
- Schema-managed templates remove bespoke boilerplate, so new notes start with consistent frontmatter.
- Contributors must use the new CLI workflow (or keep templates in sync) when introducing documentation work.

## Updates
{{LOC|20}}

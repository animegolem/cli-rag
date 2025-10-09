---
id: ADR-006
tags:
  - toml
  - config
  - merge_conflict
status: accepted
depends_on:
  - ADR-001
  - ADR-004
created_date: 2025-08-27
last_modified: 2025-08-27
related_files:
  - .cli-rag.toml
---

# config-loader-error-codes

## Objective
<!-- A concise statement explaining the goal of this decision. -->

We do not wish to allow complex merge conflicts and inheritance across the .toml config files. 

We accept the following invariants: 

1. **Exactly one** top-level `.cli-rag.toml` is permitted. All files below this in the tree inherit this config. 
2.  All `[[schema]]` names are **unique**.
3. Imported files are allowed to define **schemas only** (no `[scan]`, `[graph]`, `[retrieval]`, `[authoring]`, etc.). If they do, it’s an error.

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

**Process**
1. Read the single project .cli-rag.toml.
2. Read any import = [...] files.
3. Validate each import contains only `[[schema]]` blocks (else: error).
4. Concatenate all schemas (project + imports).
5. Enforce unique schema names (else: error).
6. Freeze the effective config.

**Example Errors**
- E100: Multiple project configs detected. Only one .cli-rag.toml is allowed.
- E110: Illegal top-level key [scan] in import templates/ADR.toml. Imports may define schemas only.
- E120: Duplicate schema name "ADR" (defined in templates/ADR.toml and templates/ADR-pack.toml).
  
## Decision
<!-- What is the change that we're proposing and/or doing? -->

adopt a strict 1 top level config + all scheme tables must be named uniquely rule. if else, error. 

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

Eliminate a class of bugs and creeping config complexity wholesale. 

## Updates
<!-- Changes that happened when the rubber met the road -->

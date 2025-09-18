---
node_id: AI-EPIC-003
tags:
  - EPIC
  - AI
  - contracts
  - adr
  - alignment
date_created: 2025-09-18
date_completed:
kanban-status: in-progress
AI_IMP_spawned:
  - AI-IMP-022
  - AI-IMP-023
  - AI-IMP-024
  - AI-IMP-025
  - AI-IMP-026
---

# AI-EPIC-003-adr-contract-alignment

## Problem Statement/Feature Scope 
The current CLI and authoring flow only partially reflect the ADR contract. Top-level AI commands are inconsistent (`ai-index-plan` vs `ai new`), template linkage is split between TOML and Markdown with gaps in prompting guidance, and file placement relies on filename templates instead of explicit destinations. This creates friction for contributors and CI drift when adding new docs. {LOC|10}

## Proposed Solution(s) 
Unify AI surfaces under a single `ai` namespace with `ai index plan|apply` and `ai new …` subcommands, deprecating the legacy top-level variants. Align schema templates with the contract by moving prompt-rich guidance from the contract examples into our Markdown templates and documenting how TOML links to those bodies. Introduce explicit output path keys (global and per-schema override) so note placement no longer relies on filename tricks. Update help/README to set expectations and smooth migration. {LOC|25}

## Path(s) Not Taken 
- Fully removing legacy commands immediately (we will deprecate first).
- Building a new templating DSL; we leverage existing placeholders and contract patterns. {LOC|10}

## Success Metrics 
- By end of sprint N: `cli-rag ai index plan|apply` documented and usable; top-level aliases emit a deprecation notice. 
- By end of sprint N+1: Templates incorporate contract-style guidance; new notes from `ai new` pass validate without manual edits. 
- 100% of new ADR/IMP/EPIC notes created via CLI with correct destinations. {LOC|15}

## Requirements

### Functional Requirements
- [ ] FR-1: Provide `cli-rag ai index plan|apply` under `ai` namespace with identical outputs to current commands.
- [ ] FR-2: Emit deprecation warnings for `ai-index-plan` and `ai-index-apply` for one release window (no behavior change).
- [ ] FR-3: Link schema TOML to Markdown templates and copy contract guidance (hidden comments, sections) into templates.
- [ ] FR-4: Support explicit output destinations: global `[config.authoring.destinations]` and per-schema `[schema.new] output_path` override.
- [ ] FR-5: Update help/README/completions to reflect the unified AI command layout and template behavior. {LOC|40}

### Non-Functional Requirements 
- Preserve protocolVersion and schema outputs; zero breaking changes in JSON.
- Maintain deterministic behavior; keep CI green across OS/rust matrix. {LOC|20}

## Implementation Breakdown 
- AI-IMP-022: AI subcommand alignment (`ai index plan|apply` + deprecations)
- AI-IMP-023: Template parity with contract examples (prompts, hidden guidance) and docs
- AI-IMP-024: Output destination keys (global + per-schema) and docs
- AI-IMP-025: Deprecate legacy `new` path and steer to AI-first authoring
- AI-IMP-026: Help/README/completions refresh and migration notes {LOC|25}


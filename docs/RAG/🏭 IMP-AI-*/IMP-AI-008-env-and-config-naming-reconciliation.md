---
node_id: IMP-AI-008
tags:
  - DX
  - docs
  - config
kanban_status:
  - planned
kanban_statusline: Reconcile env vars/flags and .adr-rag.toml vs .cli-rag.toml naming for consistency.
depends_on:
  - ADR-001
blocked_by: []
created_date: 2025-08-30
related_files:
  - src/config.rs
  - src/cli.rs
---

# IMP-AI-008-env-and-config-naming-reconciliation

### **Goal**
Establish a single canonical config filename and consistent env var/flag names; document aliases and deprecations with clear warnings.

### **Context**
The docs reference both `.cli-rag.toml` and `.adr-rag.toml`; env vars must be coherent for scripting and NVIM/MCP integration. Cleaning this up avoids confusion and stabilizes automation.

### **Implementation Plan**
- [ ] Decide and document canonical filename: adopt `.adr-rag.toml` (with optional read-only alias `.cli-rag.toml` for migration, warning on use).
- [ ] Env audit: standardize on `ADR_RAG_*` names; keep any legacy aliases with deprecation warnings printed once per run.
- [ ] CLI flags: ensure `--config` and `--base` semantics match env precedence; document in `doctor` output.
- [ ] Update templates and ADRs to consistently reference `.adr-rag.toml`.
- [ ] Tests: env precedence over file; legacy alias triggers warning; `doctor --json` surfaces resolved config path and envs used.

### **Acceptance Criteria**
- Running with `.cli-rag.toml` present prints a deprecation warning and still loads; `.adr-rag.toml` preferred when both exist.
- `doctor` clearly shows which config is loaded and which env vars influenced resolution.
- Templates/ADRs consistently reference the chosen filename.

### **Takeaway**
To be completed upon ticket closure (record final alias policy and sunset timeline).

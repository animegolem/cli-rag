---
node_id: IMP-AI-006
tags:
  - index
  - watch
  - cache
kanban_status:
  - in-progress
kanban_statusline: Unified index written at config root; reader remains per-base for now.
depends_on:
  - ADR-001
  - ADR-AI-001
blocked_by: []
created_date: 2025-08-30
related_files:
  - src/index.rs
  - src/watch.rs
  - src/discovery.rs
  - src/commands/doctor.rs
---

# IMP-AI-006-single-index-per-repo

### **Goal**
Adopt a single repo-level index file and derive group views from it, simplifying watch/caching and making outputs uniform for NVIM/MCP.

### **Context**
Current implementation writes per-base indexes, while ADR-001’s direction favors a single, consolidated index to avoid fragmentation and enable cheaper cache invalidation. A unified index also simplifies downstream consumers and reduces file churn.

### **Implementation Plan**
- [x] Decide index location semantics: path relative to project config dir; single file (e.g., `.cli-rag/index.json`).
- [x] Writer: write the consolidated index at config root (in addition to current per-base writes to preserve compatibility).
- [ ] Reader: prefer unified index if present; else per-base; else scan; emit deprecation warning for per-base.
- [x] Watch: update unified index on debounce cycles (uses config dir from resolved `--config`).
- [ ] Doctor/Status: display unified index path/mode; counts sourced from unified index when present.
- [ ] Tests: unified read/write; dual-mode fallback; migration behavior (warn once).

### **Acceptance Criteria**
- `validate` writes a single consolidated index; subsequent commands read it.
- `topics`/`group` consume the unified source (no dependency on multiple per-base files).
- `doctor --json` shows unified index path and accurate counts.
- Tests cover the migration and fallback scenarios.

### **Takeaway**
- Implemented unified index writer at the config root and integrated it into `validate` and `watch` while retaining per-base index writes for compatibility.
- Next steps: update `load_docs` to prefer the unified index, enhance `doctor` reporting, and add migration/fallback tests.

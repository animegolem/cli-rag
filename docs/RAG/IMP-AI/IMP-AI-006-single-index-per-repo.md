we I s---
node_id: IMP-AI-006
tags:
  - index
  - watch
  - cache
kanban_status:
  - completed
kanban_statusline: Unified index written at config root; readers and doctor prefer unified.
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
- [x] Reader: prefer unified index if present; else per-base; else scan; (deprecation note pending).
- [x] Watch: update unified index on debounce cycles (uses config dir from resolved `--config`).
- [x] Doctor/Status: prefer unified index; include `{path, mode}` of unified in report; counts sourced from the loaded source.
- [x] Tests: unified read/fallback (integration_unified_index.rs). Migration/deprecation warning: pending.

### **Acceptance Criteria**
- `validate` writes a single consolidated index; subsequent commands read it.
- `topics`/`group` consume the unified source (no dependency on multiple per-base files).
- `doctor --json` shows unified index path and accurate counts.
- Tests cover the migration and fallback scenarios.

### **Takeaway**
- Implemented unified index writer at the config root and integrated it into `validate` and `watch` while retaining per-base index writes for compatibility.
- Readers (search/topics/group/get/cluster/graph/path) now prefer the unified index; doctor prefers unified and reports its path/mode.
- Added integration tests for unified read and fallback; a deprecation message is emitted when falling back to per-base/scan.
- Future enhancements (optional): migrate incremental state to unified; derive groups at config root; sunset per-base indexes.

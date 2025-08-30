---
node_id: IMP-AI-006
tags:
  - index
  - watch
  - cache
kanban_status:
  - planned
kanban_statusline: Adopt a single index per repo and derive groups from it.
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
- [ ] Decide index location semantics: path relative to project config dir; single file (e.g., `.adr-rag/index.json`).
- [ ] Writer: merge all bases into a single in-memory list and write the consolidated index; move groups generation to be derived (or embed groups in index).
- [ ] Reader: prefer unified index if present; else scan; ensure backward compatibility with legacy per-base indexes (deprecation warning).
- [ ] Watch: on changes, update unified index once per debounce window; emit NDJSON events (`index_written`).
- [ ] Doctor/Status: display unified index path and mode; counts sourced from the unified index.
- [ ] Tests: unified read/write; dual-mode fallback; migration behavior (warn once).

### **Acceptance Criteria**
- `validate` writes a single consolidated index; subsequent commands read it.
- `topics`/`group` consume the unified source (no dependency on multiple per-base files).
- `doctor --json` shows unified index path and accurate counts.
- Tests cover the migration and fallback scenarios.

### **Takeaway**
To be completed upon ticket closure (migration notes and perf observations).

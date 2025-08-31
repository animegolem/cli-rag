---
node_id: IMP-AI-010
tags:
  - refactor
  - discovery
  - index
kanban_status:
  - completed
kanban_statusline: Completed — discovery split into scan/per_base/unified with facade.
depends_on:
  - IMP-AI-006
blocked_by: []
created_date: 2025-08-31
related_files:
  - src/discovery/
  - src/index.rs
---

# IMP-AI-010-split-discovery-module

### **Goal**
Extract `src/discovery.rs` into smaller modules: scanning, per-base index reader, and unified index reader (plus a facade), to make the retrieval paths explicit and easy to evolve.

### **Context**
With unified index in place, discovery now supports multiple sources (unified, per-base, scan). `src/discovery.rs` is over 350 lines; splitting clarifies responsibilities and helps as we add incremental improvements and deprecation steps.

### **Implementation Plan**
- [x] Create `src/discovery/` and introduce `mod.rs` facade with the stable functions used by commands.
- [x] `scan.rs`: file system scan + parse front matter.
- [x] `per_base.rs`: read legacy per-base indexes (current logic).
- [x] `unified.rs`: read unified index logic (current `load_docs_unified`).
- [x] `facade`: `load_docs_unified`, `load_docs`, and `docs_with_source` reexported; no behavior changes.
- [x] Keep public function signatures intact so callers don’t change.

### **Acceptance Criteria**
- Code compiles and all integration tests pass unchanged. ✅
- Discovery surface is clearer; each source has its own module. ✅
- File size in each new module stays well under the 350-line guideline. ✅

### **Takeaway**
Modular discovery reduces coupling and makes subsequent feature work (e.g., unified incremental, deprecation warnings) simpler and safer.

---
node_id: IMP-AI-012
tags:
  - protocol
  - acp
  - ndjson
  - outputs
kanban_status:
  - completed
kanban_statusline: Completed — watch NDJSON; get --format ai; doctor capabilities; path/validate locations.
depends_on:
  - ADR-AI-004
blocked_by: []
created_date: 2025-08-31
related_files:
  - src/protocol.rs
  - src/watch.rs
  - src/commands/watch_cmd.rs
  - src/commands/get.rs
  - src/commands/doctor.rs
---

# IMP-AI-012-acp-alignment-surfaces

### Goal
Implement the minimal ACP-aligned protocol surfaces described in ADR-AI-004: NDJSON watch stream, AI retrieval format, and capabilities exposure. This will align us for [[ADR-002-Visual-Mode-planning]]

### Context
Editors and agents can more easily consume cli-rag if our outputs follow the ACP shapes for content and updates. This work is additive and keeps existing JSON outputs unchanged.

### Implementation Plan
- [x] `src/protocol.rs`: Add structs/enums
  - [x] `PROTOCOL_VERSION: u32 = 1`.
  - [x] `ContentBlock` (text|image|audio|resource_link|resource).
  - [ ] `ToolCallContent::Diff { path, oldText?, newText }` (deferred; not yet needed).
  - [x] `ToolCallLocation { path, line? }`.
  - [x] `SessionUpdate` tagged variants for `validated`, `index_written`, `groups_written`.
- [x] Watch NDJSON
  - [x] Add `watch --json` flag to emit NDJSON events using `SessionUpdate` shapes.
  - [x] Integrate with existing validation/index write flow.
  - [ ] Tests for events (planned follow-up; manual verification OK).
- [x] `get --format ai`
  - [x] AI format returns `{ id, title, file, neighbors?[], content: ContentBlock[] }`.
  - [x] Includes `resource_link` and `text` blocks.
  - [x] Integration test added: `tests/integration_get_ai.rs`.
- [x] Doctor capabilities
  - [x] `doctor --json` includes `{ protocolVersion, capabilities }`.
  - [x] `pathLocations` set true after adding locations.
- [x] Path/validate locations (follow-up bundled here)
  - [x] `path --format json|ai` includes `locations: [{path,line?}, ...]` for each hop (best-effort scan).
  - [x] `validate --json|--ndjson` attaches optional `{path,line?}` per issue when derivable.
- [x] README surfaces overview.

### Acceptance Criteria
- `watch --json` emits NDJSON events with the specified shapes. ✅ (manual)
- `get --format ai` returns ACP-like content blocks. ✅ (integration test)
- `doctor --json` advertises protocolVersion and capabilities. ✅
- `path` JSON includes `locations` best-effort. ✅
- `validate` JSON/NDJSON issues include optional `{path,line?}` when derivable. ✅
- All tests green. ✅

### Takeaway
These surfaces lay the groundwork for NVIM/TUI consumption and future ACP/MCP adapters without breaking current consumers.

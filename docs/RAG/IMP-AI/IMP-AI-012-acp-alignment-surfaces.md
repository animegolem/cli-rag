---
node_id: IMP-AI-012
tags:
  - protocol
  - acp
  - ndjson
  - outputs
kanban_status:
  - planned
kanban_statusline: Add ACP-aligned surfaces: watch NDJSON, get --format ai, doctor capabilities.
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
Implement the minimal ACP-aligned protocol surfaces described in ADR-AI-004: NDJSON watch stream, AI retrieval format, and capabilities exposure.

### Context
Editors and agents can more easily consume cli-rag if our outputs follow the ACP shapes for content and updates. This work is additive and keeps existing JSON outputs unchanged.

### Implementation Plan
- [ ] `src/protocol.rs`: Add structs/enums
  - `PROTOCOL_VERSION: u32 = 1`.
  - `ContentBlock` (text|image|audio|resource_link|resource).
  - `ToolCallContent::Diff { path, oldText?, newText }`.
  - `ToolCallLocation { path, line? }`.
  - `SessionUpdate` tagged variants for `validated`, `index_written`, `groups_written` (reserve `tool_call*`).
- [ ] Watch NDJSON
  - Add `watch --json` flag to emit NDJSON events using `SessionUpdate` shapes.
  - Integrate with existing validation/index write flow.
  - Tests: header event on startup; index_written/groups_written events produced.
- [ ] `get --format ai`
  - Add AI format returning `{ id, title, file, neighbors?[], content: ContentBlock[] }`.
  - `content` is at least one `text` block of the file content; add `resource_link` to file URI/path.
  - Tests: shape presence; backward compatibility of existing formats.
- [ ] Doctor capabilities
  - Extend `doctor --json` to include `{ protocolVersion, capabilities: { watchNdjson, getAi, pathLocations } }`.
  - Tests: fields present when enabled.

### Acceptance Criteria
- `watch --json` emits NDJSON events with the specified shapes.
- `get --format ai` returns ACP-like content blocks.
- `doctor --json` advertises protocolVersion and capabilities.
- All existing tests pass; new tests cover the added surfaces.

### Takeaway
These surfaces lay the groundwork for NVIM/TUI consumption and future ACP/MCP adapters without breaking current consumers.


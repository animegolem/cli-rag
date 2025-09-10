---
id: ADR-AI-004
tags:
  - protocol
  - ACP
  - integration
  - json
  - ndjson
status: complete
depends_on:
  - ADR-002
  - ADR-008
created_date: 2025-08-31
last_modified: 2025-08-31
related_files:
  - src/protocol.rs
  - src/watch.rs
  - src/commands/watch_cmd.rs
  - src/commands/get.rs
  - src/commands/path.rs
  - src/commands/doctor.rs
---

# ADR-AI-004-acp-aligned-protocol-surfaces

## Objective
Align cli-rag's machine-readable surfaces (JSON/NDJSON) with the emerging Agent Client Protocol (ACP) patterns to maximize editor/agent interoperability while keeping our current outputs backward compatible.

## Context
Zed and Google announced the Agent Client Protocol (ACP), defining a session-oriented, capability-negotiated protocol with structured content blocks, tool-call progress, and an event stream (`session/update`).

cli-rag already exposes JSON/NDJSON for `validate`, `search`, `group`, and plans NDJSON for `watch`. Our agent does not directly drive an editor, but ACP's shapes (content blocks, diffs, locations, and status updates) map well to our use cases: rendering note content for LLMs, streaming watch events to NVIM/TUI, and eventual edit/new flows.

## Decision
Adopt ACP-inspired primitives and surfaces with additive, backward-compatible changes:

1) Content and locations
- Introduce content primitives in `src/protocol.rs` mirroring ACP:
  - ContentBlock: `text`, `image`, `audio`, `resource_link { uri }`, `resource { mimeType, data }`.
  - ToolCallContent: include a `diff { path, oldText?, newText }` variant for edits/new files.
  - ToolCallLocation: `{ path, line? }` for file + optional line reference.

2) NDJSON event stream (watch)
- Add `watch --json` (NDJSON) that emits ACP-like updates:
  - `sessionUpdate: "validated"` with `{ ok, docCount }` (header-like summary).
  - `sessionUpdate: "index_written" | "groups_written"` with `{ path, count }`.
- Reserve `sessionUpdate: "tool_call"|"tool_call_update"` for future edit flows (e.g., `new`).

3) Retrieval for LLMs (`get --format ai`)
- Add AI format that returns a compact object with ACP-style content blocks:
  - `{ id, title, file, neighbors?[], content: ContentBlock[] }`.
  - Keep existing `--format json` intact; `ai` is additive.

4) Capabilities and versioning
- Add `PROTOCOL_VERSION = 1` constant and advertise capabilities in `doctor --json`:
  - `{ protocolVersion, capabilities: { watchNdjson, getAi, pathLocations } }`.
  - All fields are additive and optional to remain backward compatible.

5) Schemas and stability
- Define Rust structs for the above in `src/protocol.rs` and keep them additive.
- Optionally publish JSON Schema as a future improvement (codegen not required now).

## Consequences
Benefits:
- Clearer, editor-friendly outputs for NVIM/TUI integrations and future MCP/ACP adapters.
- A standard shape for content and diffs unlocks richer UI and tool-follow features.

Tradeoffs:
- Slightly more complex protocol layer and documentation burden.
- Requires careful adherence to additive changes to preserve backward compatibility.

## Updates
see [[IMP-AI-012-acp-alignment-surfaces]]


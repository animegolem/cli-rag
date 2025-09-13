---
node_id: AI-IMP-006
tags:
  - IMP-LIST
  - Implementation
  - watch
  - ndjson
kanban_status: completed
depends_on: []
confidence_score: 0.9
created_date: 2025-09-12
close_date: 2025-09-12
---

# AI-IMP-006-watch-ndjson-handshake-and-events

## Summary of Issue #1
Standardize the watch NDJSON stream per global conventions: emit a first handshake line `{"event":"watch_start","protocolVersion":1}` followed by event envelopes with minimal, typed payloads. Outcome: `watch --json` produces a valid handshake first line, subsequent event objects with `event` and payload fields, and a short CI check asserts the handshake.

### Out of Scope 
- Long-running stability tests or OS-specific FS edge cases.
- Rich diff payloads; keep event payloads minimal for now.

### Design/Approach  
- On command start with `--json`, immediately print the handshake line and flush stdout.
- Define a small set of events: `scan_start`, `scan_complete`, `file_changed`, `index_written`, `groups_written`. Each includes minimal fields (paths, counts, durations as available).
- Ensure all events include `protocolVersion` and `event` fields.
- Keep determinism and avoid excessive chatter (debounce already exists).

### Files Touched
- `src/watch.rs`: added handshake emission and structured event envelopes with `protocolVersion`.
- `tests/integration_watch_ndjson.rs`: asserts first line handshake.
- `.github/workflows/ci.yml`: added a short watch run to validate handshake.

### Implementation Checklist
<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [x] Emit `{"event":"watch_start","protocolVersion":1}` as first line on `watch --json`.
- [x] Event envelope with `event` and minimal payload; include `protocolVersion` on each event.
- [x] Tests: spawn `watch --json`, read first line handshake, terminate process; parse line as JSON.
- [x] CI: add a minimal handshake assertion step.

### Acceptance Criteria
**GIVEN** a repo, **WHEN** starting `watch --json`, **THEN** the first line equals the handshake event and subsequent lines are valid JSON objects with `event` fields.

### Issues Encountered 
{None yet}

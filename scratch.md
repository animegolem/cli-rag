# scratch.md

  Observed Tensions

  - Naming drift: kanbanStatusline vs kanbanStatusLine appears across examples and between ai_get
  neighbors and search results.
  - TODO shape vs examples: AgendaView examples use priorities like TODO@10 (numeric) and sometimes
  dates; search.todo currently has priority as a string and no dueDate/source.
  - Consistency across surfaces: ai_get neighbors expose optional kanbanStatusLine; search.note/kanban
  use kanbanStatus/kanbanStatusline; fields and casing aren’t aligned.
  - Extensibility: We want Lua-driven extraction policies without baking UI‑specific choices into
  core schemas; outputs should carry enough signals (priority, due date, source, spans) for multiple
  UI treatments.

  Proposed Contract Changes (v1 patch)

  - Standardize kanban field casing
      - Use kanbanStatus for the state, kanbanStatusLine for the short description/line across all
  surfaces.
      - Changes:
          - v1/cli/search_result.schema.json: rename kanbanStatusline → kanbanStatusLine
  (note.kind=kanban item), and allow kanbanStatusLine? on note kind.
          - v1/cli/ai_get.schema.json: keep neighbors.kanbanStatusLine (already matches), no change.
      - Code: emit the standardized field; if any legacy uses exist, stop emitting them (we’re alpha).
      
>  Treat the contracts as authoritative in cases of minor/non-logic conflicts overall the idea of the contracts is to protect from inevitable document drift and prevent endless correction churn. If the contracts aren't consistent we'll need to fix them. 
>  The other north star is the global conventions document. When there is casing doubt we should align there. e.g 

```
## Project-wide naming and casing
- TOML: snake_case keys. No hyphens in keys.
- Lua: snake_case for keys and hook names. Module returns table with fields mirroring TOML (snake_case).
- JSON outputs: camelCase keys. Exception: edge kind names and schema names are emitted as-is (e.g., "depends_on", "ADR").
- Frontmatter (YAML): snake_case keys.
```
      
> But agreed there are two functions here so if they've blended we should make a clear separation. You've got it correct, status is the declarative state and statusline is just a descriptor. 
> The above list does however show we've not set any case for _in the cli itself_ we have an actual gap in our standards. It seems likely we should align to snake or camel. 

  - Enrich TODO items for AgendaView without locking in policy
      - v1/cli/search_result.schema.json (todo kind):
          - Add dueDate? (date) — optional, supports body or FM derived due dates.
          - Add source? ("body" | "frontmatter") — indicates where the task came from.
          - Add span? [start, end] — optional byte offsets in the note’s body (consistent with
  diagnostics) for precise UI highlights.
          - Keep priority as string but add priorityScore? (integer 1–10) — lets Lua map TODO@10 to a
  numeric scale while preserving human labels like "HIGH".
      - Rationale: preserves current clients but enables richer sorting/highlighting and consistent
  extraction provenance.
  
> This seems like the right stance that covers the needs for the current tooling but lets me keep thinking about the final shape. 
  
  - Align ai_get neighbor metadata (optional GTD hints)
      - v1/cli/ai_get.schema.json neighbors items:
          - Add kanbanStatus? (string) — parallel to search.note.
          - Keep kanbanStatusLine? (string|null) — already present.
          - Optionally add dueDate? (date) — when neighbor is a kanban candidate.
      - Rationale: makes ai_get usable for local “what’s near me that’s in-progress/blocked” views
  without another round trip.
  
> I agree, these are sensible additions to expose in the search. I agree due is worth adding.
  
  - Info capabilities advertisement (non‑breaking)
      - v1/cli/info.schema.json capabilities (additionalProperties already true):
          - Encourage adding gtdTasks: true, kanban: true to signal that the CLI emits those shapes
  (optional).
      - Rationale: UI plugins can feature‑detect.

> Not something I'd actively thought through but this seems sane and reasonable to me yeah. 

```
  Out of Scope Now (but worth noting)

  - New “agenda” endpoint: current ADR uses search --todo/--kanban to power AgendaView, which is
  sufficient; no need for a separate schema.
  - Enumerating kanbanStatus values: leave open (project/workflows differ). Lua can normalize.
  - Task completion toggles: operationalize later; search.todo already carries completedAt for status.
```

> Reasonable to punt till we hit the ui phase i do agree. 

```
  Impact Overview

  - Backward compatibility: We’re alpha; prefer correctness over shims. The only breaking change
  proposed is renaming kanbanStatusline → kanbanStatusLine in search results. Everything else is
  additive/optional.
  - Implementation:
      - Update extraction to populate new todo fields (dueDate/source/span/priorityScore) via Lua
  policies, with safe fallbacks.
      - Search emitters to use kanbanStatusLine consistently.
      - ai_get neighbor builder to optionally include kanbanStatus/kanbanStatusLine/dueDate when
  available from index/FM.
  - CI/contracts:
      - Update v1/cli/search_result.schema.json and v1/cli/ai_get.schema.json accordingly.
      - [ ] Add/adjust CI validators once code paths emit the new fields (optional fields should not break
  existing tests).
```

* [ ] If you agree, I’ll 
  - Patch the two schemas (search_result, ai_get) per above.
  - Add a brief note in Bridge Plan to track the naming normalization and todo field enrichments.
  - Open a small IMP ticket for “GTD schema polish (naming + todo fields)” so we can implement
  emitters + tests right after Graph/Path CI validators.

Perfect, I agree. A small addition. Lets file an extremely concise change log in the root of contracts and then make the schema changes required. I've moved the contracts to v0.1 to reflect the 'in alpha' reality of the project. 

> Prompt below this line 

# Contracts Change Log 

## {Date}: High Level Description 

### Reason for change 

### Overview of change 

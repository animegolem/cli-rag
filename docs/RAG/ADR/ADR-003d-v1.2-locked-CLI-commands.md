---
id: ADR-003d
tags:
  - cli
  - refactor
status: accepted
depends_on:
  - ADR-001
  - ADR-004
created_date: 2025-08-25
last_modified: 2025-08-26
related_files:
  - ~/cli-rag/src/commands
---

# v1.2-locked-CLI-commands


## Objective
<!-- A concise statement explaining the goal of this decision. -->

Present a concise "final" list of commands before full implementation. These tools are designed for either a human or an AI consumer.

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

### Commands

---

#### Cross-cutting flags (all verbs)

- `--json` machine outputs. `watch` streams ndjson
- `--color auto|always|never`
- `--cwd <path>` override working dir (great for NVIM/project roots).
- `--editor <cmd>` and `--no-editor`.

---

#### Human Focused Commands

##### `init` 

Create `.cli-rag.toml` (and optional `templates/`), and open the commented file in the default editor.

**Flags**

- `--schema <NAME>` scaffold a commented `[[schema]]` block (writes to `.cli-rag.toml` or `templates/<NAME>.toml` if `--separate`).
- `--separate` put schema into `.cli-rag/templates/<NAME>.toml` instead of inline.
- `--silent` write without opening editor.
- `--force` overwrite if exists.
- `--print-template` print to stdout (no files).
- `--json` machine summary of what was created.

###### Implementation Notes 

The first run of init should maybe be very very lightly interactive e.g.

Init opens

```bash
Welcome to cli-rag! Do you want to: 

    1. Set the Project_Manager preset? **[Recommended]**
    2. Modify the Personal_Notes templates?  
    3. Dive into a custom config? (toml or lua)
```

The new user burden is very high, currently we ship an application that does nothing on install. The alternative is to just say "no this is a project manager tool first and foremost" which is arguably the most honest. 

the preset screen would need basic information eg 

```bash
Project Manager:
  - ADR schema for architectural decisions
  - IMP schema for implementation tickets  
  - Kanban workflow with due dates
  - Git-friendly incremental IDs
  
[Press ENTER to use this preset, or ESC to go back]
```

presets should actually live in the binary directly as a const string. 


##### `new`

Create a new note (runs Lua hooks for id/frontmatter/template fill). Optionally open in editor. **Human-oriented.**

**Flags**

* `--schema <NAME>` define schema under which the note will be created
* `--title <T>` and/or `--id <I>` (if both, engine verifies consistency with schema rules)
* `--edit` open the created note in the configured editor
* `--dry-run` print would-be filename/frontmatter/body (no write)
* `--print-body` when used with `--dry-run`, include the body text
* `--json` return `{path, id, schema, frontmatter}` (body omitted unless `--print-body`)

##### `validate` (Simplified flags as the `groups` command has been removed.)

Creates/Updates the unified index. Does not write to a cache. Run validators (TOML + Lua) and exit non-zero on any error.

Validation enforces:
- Per-edge policies (`required`, `cycle_detection`) with schema-level severity fallbacks.
- Wikilink thresholds via `schema.validate.edges.wikilinks` (unique outgoing targets per note, unique incoming referrers across notes).
- Cross-schema allowlists: leaving `allowed_targets` empty allows linking to any schema; populating the list restricts to those schemas.

**Flags**
- `--dry-run` do not write/update index. print errors. 
- `--full-rescan` ignore incremental cache/index and rescan everything
- `--json` emits `{ ok, docCount, diagnostics:[{severity,code,msg,path,span?,field?}] }`

eg
```json
{  
"ok": true,  
"docCount": 126,  
"diagnostics": [  
{  
"severity": "warning",  
"code": "LINK_MIN_OUT",  
"msg": "notes/ADR-002-Visual-Mode-planning.md: wikilinks outgoing unique targets 0 below minimum 1",  
"path": "notes/ADR-002-Visual-Mode-planning.md",  
"span": [123,145],  
"field": null  
}  
]  
}
```


##### `info` (RENAMED FROM DOCTOR DUE TO THE COMMAND BEING PASSIVE AND `DOCTOR` OVERLAPPING WITH `VALIDATE`)

Show resolved config, discovery mode, counts, and unknown-frontmatter stats.

**Flags**
- `--json` machine summary
- `--verbose` include per-schema file lists
- **Human mode**: use table output; color diagnostics

eg 

```json
{
  "protocolVersion": 1,
  "config": { "path": ".cli-rag.toml", "version": "0.3", "deprecated": false },
  "index": { "path": ".cli-rag/index.json", "exists": true },
  "cache": { "aiIndexPath": ".cli-rag/cache/ai-index.json", "exists": false },
  "capabilities": {
    "watchNdjson": true,
    "aiGet": { "retrievalVersion": 1 },
    "pathLocations": true,
    "aiIndex": false
  }
}
```

##### `watch`

Incrementally indexes & validates while files change. Runs when visual mode is open for live updating file changes. 

**Flags**
- `--debounce-ms <n>` default 400
- `--dry-run`
- `--full-rescan`
- `--json` stream structured events per change: `{"event":"validated","path":...,"diagnostics":[...]}`

eg
```json
{"event":"validated","ok":true,"docCount":126}  
{"event":"index_written","path":".cli-rag/index.json","count":126}
```

##### `search` (Expanded, groups removed)

Purpose: list nodes by fuzzy match, schema/status filters, or recency; also surface TODO/kanban views.
    
Possibly this should use [fuzzy matcher](https://github.com/skim-rs/fuzzy-matcher) but a repl might complicate agentic use. 
	
**Flags:**
- `--query <substr>` optional; if omitted, acts as a “browse”
- `--schema <S>`
- `--status <csv>` optional filter on status frontmatter
- `--field <frontmatter_key>=<regex>` repeatable
- `--topic <regex>` filters on index.computed.topics[].label (computed)
- `--tags <regex>` filters on frontmatter.tags (human)
- `--recent <N>` returns last N changed (by index recency; see note)
- `--todo` include inline TODOs parsed from body and FM-derived tasks
- `--kanban` list items with kanban_status set (respect --schema, --status)
- `--json`

eg 
Return a consistent envelope with results (not a bare array), plus a “kind” discriminator per item. This makes adding metadata easy later.

```json
{  
"results": [  
{  
"kind": "note",  
"id": "ADR-002",  
"title": "Visual-Mode-planning",  
"schema": "ADR",  
"path": "ADRs/ADR-002-Visual-Mode-planning.md",  
"tags": ["TUI","NeoVIM","rataTUI"],  
"status": "draft",  
"kanbanStatus": null,  
"score": 0.87,  
"lastModified": "2025-08-28T12:14:00Z",  
"lastAccessed": "2025-09-07T21:43:00Z"  
},  
{  
"kind": "todo",  
"id": "ADR-002#L83",  
"noteId": "ADR-002",  
"schema": "ADR",  
"path": "ADRs/ADR-002-Visual-Mode-planning.md",  
"line": 83,  
"priority": "HIGH",  
"text": "Critical parser bug",  
"createdAt": "2025-09-01T10:22:00Z",  
"completedAt": null  
},  
{  
"kind": "kanban",  
"id": "IMP-004",  
"noteId": "IMP-004",  
"schema": "IMP",  
"path": "IMP/IMP-004-something.md",  
"kanbanStatus": "in_progress",  
"kanbanStatusLine": "Refactor validation",  
"dueDate": "2025-09-03"  
}  
]  
}
```

###### Implementation Notes
- Output
	- note: `{ id, title, schema, path, tags?, status?, kanbanStatus?, score?, lastModified?, lastAccessed? }`
	- todo: `{ id, noteId, schema, path, line, priority, text, createdAt?, completedAt? }`
	- kanban: `{ id, noteId, schema, path, kanbanStatus, kanbanStatusLine?, dueDate? }`
- Fuzzy Filters
    - Use an in-proc fuzzy scorer (skim/skim-like) internally; no REPL. Deterministic sort: score desc, then `last_modified` desc, then `id`.
    - `--recent` uses index fields: `file_mtime` (required), `git_last_commit_ts` (optional), or your own `last_accessed` when commands touch a note. Do not write this into files.
- Deterministic ordering rules
	- search: score desc → lastModified desc → id asc
	- neighbors: score desc → lastModified desc → id asc
	- Optional topic boost for neighbors in primary cluster: +0.05 (documented, small)

##### `graph`

Emit a local dependency graph for an ID (or whole repo with `--root none`).

**Flags**
- `--id <ID>|--root none`
- `--depth <N>` defaults to 1 and shows immediate connections. 
- `--include-bidirectional true|false`  default true 
- `--graph-format ascii|mermaid|dot` (default `ascii`)
- `--json` machine summary (AI consistent alias)

eg
```json
{  
"root": { "id": "ADR-002" },  
"nodes": [  
{ "id": "ADR-002", "title": "Visual-Mode-planning", "schema": "ADR" },  
{ "id": "ADR-001", "title": "cli-rag.toml", "schema": "ADR" }  
],  
"edges": [  
{ "from": "ADR-002", "to": "ADR-001", "kind": "depends_on" }  
 ]  
}
```

##### `path` 

Shortest path between two IDs with edge kinds.

**Flags**
- `--from <ID>` `--to <ID>`
- `--json includes edges: [{from,to,kind,locations}]`

**Example**
ADR-024 → IMP-006 (depends_on)
IMP-006 → ADR-029 (mentions: `[[ADR-029]]` L42)

eg

```json
{
  "ok": true,
  "path": [
    { "id": "ADR-024", "title": "X", "schema": "ADR" },
    { "id": "IMP-006", "title": "Y", "schema": "IMP" },
    { "id": "ADR-029", "title": "Z", "schema": "ADR" }
  ],
  "edges": [
    { "from": "ADR-024", "to": "IMP-006", "kind": "depends_on", "locations": [] },
    { "from": "IMP-006", "to": "ADR-029", "kind": "mentions", "locations": [{ "path": "IMP/IMP-006.md", "line": 42 }] }
  ]
}
```

---

#### ai Focused Sub-Commands

Agent workflow for two-phase creation with on-creation validation. Not intended for direct human use. All outputs return structured json. 

##### `ai new`

###### `ai new start`

Begin an agent draft; reserves ID/filename and returns the section/LOC contract.

**Flags**
- `--schema <NAME>`
- `--title <T>` (optional if `--id` provided)
- `--id <I>` (optional; engine may still assign)

###### `ai new submit`

Finalize a draft by submitting filled content **before** any file is written. Enforces `scan_policy = "on_creation"` (LOC per heading, heading rules, frontmatter enums/regex, Lua `validate`).

**Flags**
- `--draft <DRAFT_ID>` (required)
- one of:
    - `--stdin` (reads structured JSON `{frontmatter, sections}` from stdin)
    - `--sections @path.json` (same shape as stdin)
    - `--from-file <path.md>` (engine parses headings/sections)
- `--allow-oversize` write even if LOC caps fail; library marks `needs_attention=true` in index causing validate to print a one time warning. 

###### `ai new cancel`

Abort and remove a pending draft; releases reserved ID/filename.

**Flags**
- `--draft <DRAFT_ID>`
- 
###### `ai new list`

List active/stale drafts in the draft store.
**Flags**

- `--stale-days <N>` filter to drafts older than N days (default: show all)

###### Implementation Notes

###### `ai new start`

- Reserves ID/filename (no file written).
- Runs Lua `id_generator` + `render_frontmatter`.
- Returns a **contract** the agent must satisfy:

```json
{  
"draftId": "dft_7fb0c2",  
"schema": "ADR",  
"filename": "ADR-003-circuit-breaker.md",  
"constraints": {  
"headings": [  
{"name":"Objective","maxLines":50},  
{"name":"Context","maxLines":300},  
{"name":"Decision","maxLines":50},  
{"name":"Consequences","maxLines":50},  
{"name":"Updates","maxLines":100}  
],  
"headingStrictness": "missing_only",  
"frontmatter": {  
"allowed": ["id","status","tags","created_date"],  
"readonly": ["id","created_date"],  
"enums": {"status":["draft","proposed","accepted","superseded","cancelled"]}  
}  
},  
"seedFrontmatter": {"id":"ADR-003","status":"draft","created_date":"2025-08-30","tags":[]},  
"instructions": "# Instructions\nGenerate an Architectural Decision Record…\n- Use status from [draft, proposed, accepted, superseded, cancelled]\n- Fill every heading.\n- Keep it concise.",  
"noteTemplate": "---\n((frontmatter))\n---\n\n# {{filename}}\n\n## Objective\n{{LOC|50}}\n\n## Context\n{{LOC|300}}\n\n## Decision\n{{LOC|50}}\n\n## Consequences\n{{LOC|50}}\n\n## Updates\n{{LOC|100}}\n",  
"ttlSeconds": 86400,  
"contentHash": "sha256:…"  
}
```

Persist a small draft file: `.cli-rag/drafts/dft_7fb0c2.json`.

###### `ai new submit`

- Accepts either:
    
    - **structured JSON** on stdin:
        ```json
        {"frontmatter":{"tags":["resilience"]},
         "sections":{"Objective":"…","Context":"…","Decision":"…","Consequences":"…","Updates":""}}
        ```
    - `--sections @file.json` with the same shape
    - `--from-file note.md` (engine parses headings/sections)
        
- Engine builds the note **in memory**, then:
    
    1. Parse frontmatter/body
    2. Enforce `headingStrictness` & **per-heading LOC**
    3. Validate frontmatter (regex/enums)
    4. Run Lua `validate`
        
- If `scan_policy="on_creation"` and any **error** → **no write**:
    
    ```json
    {"ok":false,"draftId":"dft_7fb0c2",
     "diagnostics":[
       {"severity":"error","code":"LOC_Objective","heading":"Objective","max":50,"actual":73},
       {"severity":"warning","code":"LINK_MIN","msg":"Add at least 1 wikilink"}
     ]}
    ```
    
- If OK → write file, clear draft, return:
    
    ```json
    {"ok":true,"path":"notes/ADR-003-circuit-breaker.md","id":"ADR-003","schema":"ADR"}
    ```
    

###### `ai new submit` Notes

- `--allow-oversize` lets you create despite LOC errors; engine marks `needs_attention=true` in index so `validate` still fails later.
- [ ] Include `contentHash` in `submit` to make retries idempotent:
    - Engine rejects if same draft already finalized with same hash.

###### `ai new cancel`

- Deletes the draft file; releases reserved ID/filename.

###### `ai new list`

- Emits JSON array of stale/orphaned drafts with `created_at`, `schema`, `filename`. Use `--stale-days` to filter.

##### `ai get`

Resolve a note and its neighborhood; print for ai optimized for stepwise traversal of the graph presenting a section of context.

**Flags**
- `--id <ID> | --path <P>` (required)
- `--edges <csv>` (optional; default e.g., depends_on,related_to)
- `--include-bidirectional <true|false>` (default false)
- `--max-fanout <n>` (default 5)
- `--depth <N>` returns error (eg, `NEIGHBORS_FULL_DEPTH_GT1`) if used >1 with neighborstyle full. 
- `--json` (default; single JSON object)
- `--neighbor-style full|outline|metadata` default = metadata 
	- metadata: { id, title, schema, path, distance, discoveredFrom, edge, status?, tags?, kanbanStatusLine?, lastModified?, score? }
	- full: same as metadata + content: [{ type:"text", text, tokenEstimate? }]
	- outline: same as metadata + contentOutline: [{ heading, firstLines: [string] }]
- `--outline-lines <N>` (only used when neighbor-style=outline. Default=2) 
  
**All defaults can be adjusted/set in [[ADR-001-cli-rag.toml]]. They can also be overridden with the following optional Flags**  
  
###### Implementation Notes

Example: neighbor-style=metadata, depth=1  

```json
{  
"protocolVersion": 1,  
"retrievalVersion": 1,  
"id": "ADR-002",  
"schema": "ADR",  
"title": "Visual-Mode-planning",  
"file": "ADRs/ADR-002-Visual-Mode-planning.md",  
"frontmatter": { "status": "draft", "tags": ["TUI","NeoVIM","rataTUI"] },  
"content": [  
{ "type": "text", "text": "…entire markdown…", "tokenEstimate": 3120 }  
],  
"neighbors": [  
{  
"id": "ADR-001",  
"title": "cli-rag.toml",  
"schema": "ADR",  
"path": "ADRs/ADR-001-cli-rag.md",  
"distance": 1,  
"discoveredFrom": "ADR-002",  
"edge": "depends_on",  
"status": "accepted",  
"tags": ["config"],  
"kanbanStatusLine": null,  
"lastModified": "2025-08-28T12:14:00Z",  
"score": 0.82  
}  
],  
"limits": { "depth": 1, "maxFanout": 5 }  
}
```

Example: neighbor-style=outline, outline-lines=2, depth=1

- Same as above, but each neighbor has contentOutline instead of only metadata:
    - `"contentOutline": [{ "heading": "Context", "firstLines": ["…", "…"] }, …]`

###### **Exit Codes**

- 0 success
- 1 generic/unexpected
- 2 validation failed (bad plan, failed checks)
- 3 draft not found/expired (ai new only)
- 4 schema/config error
- 5 IO/index lock error

##### `ai index`

###### `ai index plan`
- Purpose: compute communities over graph_edges and emit a work order for an LLM/human to label and summarize.
- Flags:
    - `--edges <csv> filter which edge kinds to use`
    - `--min-cluster-size <n>` default 3
    - `--output <path.json>` required
    - `--schema <S> optional`
- Output JSON (contracts/v1/cli/ai_index_plan.schema.json):

```json
{
  "version": 1,
  "generatedAt": "2025-09-08T12:00:00Z",
  "sourceIndexHash": "sha256:…",
  "params": { "edges": ["depends_on","implements"], "minClusterSize": 3, "schema": null },
  "clusters": [
    {
      "clusterId": "c_0001",
      "members": ["ADR-001","ADR-003b","ADR-006"],
      "representatives": ["ADR-001","ADR-003b"],
      "metrics": { "size": 3, "density": 0.66 },
      "label": "",          
      "summary": "",       
      "tags": []        
    }
  ]
}
```

if sourceIndexHash doesn't match exit 2 by default

###### `ai index apply`
persist cluster labels/summaries; optionally add relevant tags per cluster to the frontmatter.

**Flags:**
- `--from <plan.json>`
- `--write-cache true|false` default true
- `--write-frontmatter true|false` default false 
  	- Only writes tags if schema defines a tags field. errors with exit 4 if not present.
  	- Tag writes are additive; no removals.
- `--dry-run`
  
  
- Behavior:
	- If plan.sourceIndexHash is present and doesn’t match current index: exit 2 by default
    - Cache write: `.cli-rag/cache/ai-index.json` (authoritative), shape:
        

```json
{
"version": 1,
"clusters": [
  {"clusterId":"c_0001","label":"retrieval","summary":"…","members":["ADR-001","ADR-003b","ADR-006"]}
 ]
}
```
        
- Frontmatter (optional): assign relevant tags to notes in cluster creating appropriate links. 
- Apply report JSON (contracts/v1/cli/ai_index_apply_report.schema.json):

```json
{
  "ok": true,
  "written": { "cache": true, "frontmatter": false },
  "clustersApplied": 3,
  "membersTagged": 9,
  "warnings": []
}
```
 
 
Cache file (authoritative, non-rebuildable) 

```json
{  
"version": 1,  
"clusters": [  
{  
"clusterId": "c_0001",  
"label": "retrieval",  
"summary": "…",  
"members": ["ADR-001","ADR-003b","ADR-006"],  
"tags": ["retrieval"], // optional label-to-tag mapping  
"confidence": 0.9, // optional  
"updatedBy": "ai|human", // optional  
"updatedAt": "2025-09-08T12:10:00Z" // optional  
}  
]  
}
```

###### Exit Codes 

- 0 success
- 1 generic/unexpected
- 2 validation failed (bad plan, failed checks)
- 3 draft not found/expired (ai new only)
- 4 schema/config error
- 5 IO/index lock error

---

## Notes 

### Groups

The entire concept of a dedicated groups frontmatter that is tracked and used in commands has been deprecated in favor of tags and the `ai index` sub-command. 

## Decision
<!-- What is the change that we're proposing and/or doing? -->

Major Changes: 
- Doctor --Renamed--> Info 
- replaced get --human--> get ai sub commands  
- groups removed in full 
- etc. it's a dramatic shift. 

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

Increased CLI ergonomics and clarity. no further changes are allowed until the dog fooding phases. 

---
id: ADR-003c
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

# v1.1-CLI-commands


## Objective
<!-- A concise statement explaining the goal of this decision. -->

Present a concise "final" list of commands before full implementation. These tools are designed for either a human or an AI consumer.

## Ideation 


- There should probably/maybe be a top level GTD command? 
	- update kanban 
	- update due date 
	- etc 

There is a chance this is a more generic "update note" tool but it's not ultra obvious how you'd actually structure that feedback loop for an agent on some items. 

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

---


- **Every command supports `--json`** for machine use; human pretty by default.  
- **Idempotent reads, explicit writes.** Anything that mutates disk says so.
- **Same nouns across CLI + NVIM** (note, schema, graph, index).
- **Lua is policy, not parsing** Lua hooks run only in `new`/`validate`.

---

### Commands

## `init`

Create `.cli-rag.toml` (and optional `templates/`), and open the commented file in the default editor.

**Flags**

- `--schema <NAME>` scaffold a commented `[[schema]]` block (writes to `.cli-rag.toml` or `templates/<NAME>.toml` if `--separate`).
- `--separate` put schema into `.cli-rag/templates/<NAME>.toml` instead of inline.
- `--silent` write without opening editor.
- `--force` overwrite if exists.
- `--print-template` print to stdout (no files).
- `--json` machine summary of what was created.

## `new`

Create a new note (runs Lua hooks for id/frontmatter/template fill). Optionally open in editor. **Human-oriented.**

**Flags**

* `--schema <NAME>` define schema under which the note will be created
* `--title <T>` and/or `--id <I>` (if both, engine verifies consistency with schema rules)
* `--edit` open the created note in the configured editor
* `--dry-run` print would-be filename/frontmatter/body (no write)
* `--print-body` when used with `--dry-run`, include the body text
* `--json` return `{path, id, schema, frontmatter}` (body omitted unless `--print-body`)

## `get` (alias show)

Resolve a note and (optionally) its neighborhood; print for humans or ai.

**Flags**

- `--id <ID>` (or `--path <P>`)
- `--depth N` (default 2)
- `--include-bidirectional`
- `--format md|ai|json`
    - `md`: stitched doc with frontmatter + appended neighbors
    - `ai`: compact JSON with `{frontmatter, text, neighbors: [...]}` optimized for LLM
    - `json`: full parse summary `{frontmatter, headings, wikilinks, md_links, code_fences, text}`

## `validate`

Run validators (TOML + Lua) and exit non-zero on any error.

**Flags**
- `--dry-run` don’t touch indexes
- `--full-rescan`
- `--write-index` write index only
- `--write-groups` write groups only
- `--write-all` both
- `--json` emits `{path, diagnostics:[{severity,code,msg,span,field}]}`
  
## `watch`

Incrementally indexes & validates while files change. Runs when the NVIM extension is open for live updating file changes. 

**Flags**
- `--debounce-ms <n>` default 400
- `--dry-run`
- `--full-rescan`
- `--json` stream structured events per change: `{"event":"validated","path":...,"diagnostics":[...]}`

## `graph`

Emit a local dependency graph for an ID (or whole repo with `--root none`).

**Flags**
- `--id <ID>|--root none`
- `--depth N|max`, `--include-bidirectional`  
- `--graph-format ascii|mermaid|dot` (default `ascii`)
- `--json` machine summary

## `status`

Show resolved config, discovery mode, counts, and unknown-frontmatter stats.

**Flags**
- `--json` machine summary
- `--verbose` include per-schema file lists
- **Human mode**: use table output; color diagnostics

## `search`

Fuzzy search ID/title; filter by schema or status.

**Flags**

- `--query <substr>`
- `--schema <S>` Search within a schema 
- `--kanban <user-defined-strings>` Search Kanban. List all tracked items with no scoping input. 
- `--TODO` Search TODO's. Lists by priority without input. 
- `--recent <X>` last X changes
- `--field` filter search by frontmatter content 
- `--json`    

## `path`

Shortest path between two IDs with edge kinds.

**Flags**
- `--from <ID>` `--to <ID>`
- `--max-depth N`
- `--json` includes `edges: [{from,to,kind,meta}]`


**Human example**

ADR-024 → IMP-006 (depends_on)
IMP-006 → ADR-029 (mentions: `[[ADR-029]]` L42)

## `groups`

List notes by `groups` frontmatter (if enabled in schema).

**Flags**
- `--list` list group names
- `--name <G>` show members of a group
- `--json`

## `ai` (JSON/NDJSON outputs)

Agent workflow for two-phase creation with on-creation validation. Not intended for direct human use.

### `ai draft start`

Begin an agent draft; reserves ID/filename and returns the section/LOC contract.

**Flags**
- `--schema <NAME>`
- `--title <T>` (optional if `--id` provided)
- `--id <I>` (optional; engine may still assign)
- `--json` (default)

---

### `ai draft submit`

Finalize a draft by submitting filled content **before** any file is written. Enforces `scan_policy = "on_creation"` (LOC per heading, heading rules, frontmatter enums/regex, Lua `validate`).

**Flags**
- `--draft <DRAFT_ID>` (required)
- one of:
    - `--stdin` (reads structured JSON `{frontmatter, sections}` from stdin)
    - `--sections @path.json` (same shape as stdin)
    - `--from-file <path.md>` (engine parses headings/sections)
- `--allow-oversize` write even if LOC caps fail; marks `needs_attention=true`
- `--json` (default)
    

---

### `ai draft cancel`

Abort and remove a pending draft; releases reserved ID/filename.

**Flags**

- `--draft <DRAFT_ID>`
- `--json` (default)

---

### `ai draft list`

List active/stale drafts in the draft store.

**Flags**

- `--stale-days <N>` filter to drafts older than N days (default: show all)
- `--json` (default)

---

**Exit codes (ai subcommands)**

- `0` success
- `2` validation failed (diagnostics returned)
- `3` draft not found/expired
- `4` schema/config error
- `5` IO/index lock error

---

### Cross-cutting flags (all verbs)

- `--json` machine output everywhere.
- `--color auto|always|never`
- `--cwd <path>` override working dir (great for NVIM/project roots).
- `--editor <cmd>` and `--no-editor`.

## Notes 

### `init`

#### Notes 

The first run of init should maybe be very very lightly interactive e.g.

Init opens

```bash
Welcome to cli-rag! Do you want to: 
    1. Dive into a custom config [**RECOMMENDED]**
    2. Set the Project_Manager preset? 
    3. Set the RPG_Manager preset?
    4. Set the Personal_Notes preset?  
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

---

### AI 

#### Behaviors & contracts

##### `ai draft start`

- Reserves ID/filename (no file written).
- Runs Lua `id_generator` + `render_frontmatter`.
- Returns a **contract** the agent must satisfy:
    

```json
{
  "draft_id": "dft_7fb0c2",
  "schema": "ADR",
  "filename": "ADR-003-circuit-breaker.md",
  "constraints": {
    "headings": [
      {"name":"Objective","max_lines":50},
      {"name":"Context","max_lines":200},
      {"name":"Decision","max_lines":50},
      {"name":"Consequences","max_lines":50},
      {"name":"Updates","max_lines":100}
    ],
    "heading_strictness": "missing_only",
    "frontmatter": {
      "allowed": ["id","status","tags","created_date"],
      "readonly": ["id","created_date"],
      "enums": {"status":["draft","proposed","accepted","superseded","cancelled"]}
    }
  },
  "seed_frontmatter": {"id":"ADR-003","status":"draft","created_date":"2025-08-30","tags":[]},
  "ttl_seconds": 86400,
  "content_hash": "sha256:…"
}
```

Persist a small draft file: `.cli-rag/drafts/dft_7fb0c2.json`.

##### `ai draft submit`

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
    2. Enforce `heading_strictness` & **per-heading LOC**
    3. Validate frontmatter (regex/enums)
    4. Run Lua `validate`
        
- If `scan_policy="on_creation"` and any **error** → **no write**:
    
    ```json
    {"ok":false,"draft_id":"dft_7fb0c2",
     "diagnostics":[
       {"severity":"error","code":"LOC_Objective","heading":"Objective","max":50,"actual":73},
       {"severity":"warning","code":"LINK_MIN","msg":"Add at least 1 wikilink"}
     ]}
    ```
    
- If OK → write file, clear draft, return:
    
    ```json
    {"ok":true,"path":"notes/ADR-003-circuit-breaker.md","id":"ADR-003","schema":"ADR"}
    ```
    

###### Notes

- `--allow-oversize` lets you create despite LOC errors; engine marks `needs_attention=true` in index so `validate` still fails later.
- Include `content_hash` in `submit` to make retries idempotent:
    - Engine rejects if same draft already finalized with same hash.

##### `ai draft cancel`

- Deletes the draft file; releases reserved ID/filename.

##### `ai draft list`

- Emits JSON array of stale/orphaned drafts with `created_at`, `schema`, `filename`. Use `--stale-days` to filter.

---

### Stable JSON schemas (so NVIM is trivial)

**`get --format ai`**

```json
{
  "id": "ADR-002",
  "schema": "ADR",
  "frontmatter": { "status": "draft", "tags": ["TUI","NeoVIM","rataTUI"] },
  "text": "…",
  "neighbors": [
    {"id": "ADR-001", "edge": "depends_on"},
    {"id": "ADR-003", "edge": "depends_on"}
  ]
}
```

**`validate --json` event (also from `watch`)**

```json
{"event":"validated","path":"notes/ADR-002-Visual-Mode-planning.md","ok":false,
 "diagnostics":[{"severity":"warning","code":"LINK_MIN","msg":"Add a wikilink","span":[123,145]}]}
```

---


## The Missing Tools 

**The "Refactor" Problem:** A common pain point in these systems is renaming a note. If NPC-Grak.md is renamed to NPC-Grak-The-Loud.md, how do all the `[[NPC-Grak]]` links get updated? A `cli-rag refactor --id <ID> --new-title <T>` command would be a massive power feature.

In the same vein we could have an add dependency or supersede tool. the issue is keeping the abstractions universal. 


## Decision
<!-- What is the change that we're proposing and/or doing? -->

Accept the above and refactor impacted files. 

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

Increased CLI ergonomics and clarity. 
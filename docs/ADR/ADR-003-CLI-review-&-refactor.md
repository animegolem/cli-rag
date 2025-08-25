---
id: ADR-003
tags:
  - cli
  - refactor
  - TUI
  - neovim
status: draft
depends_on:
  - ADR-001
  - ADR-004
created_date: 2025-08-24
last_modified: 2025-08-25
related_files:
  - ~/cli-rag/src/commands
---

# ADR-004-CLI-review-&-refactor

## Objective
<!-- A concise statement explaining the goal of this decision. -->

Until this point CLI-RAG has grown adhoc out of a a dog-fooded .js script. As the workflow is becoming more clear the existing commands should be reviewed and adjusted to ensure everything is coherent. 

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

The current command list 


- `init` — Create `.adr-rag.toml` in the current repo and open it in an editor by default. Flags:
    - `--force` overwrite if exists
    - `--print-template` print template to stdout
    - `--silent` do not open the config after creating/detecting it
- `doctor` — Show resolved config, bases, discovery mode (index vs scan), and quick stats.
    - Reports per-type counts when schemas are defined, and unknown-key stats.
    - JSON: use global `--format json`.
- `search --query <substr>` — Fuzzy search by ID/title across discovered ADR files.
- `topics` — List semantic groups derived from front matter (`groups` in ADRs).
- `group --topic "<name>" [--include-content]` — Show ADRs in a group; optionally include full content.
- `get --id ADR-021 [--include-dependents]` — Print an ADR with dependencies (and dependents if requested).
- `cluster --id ADR-021 [--depth N] [--include-bidirectional]` — Traverse dependencies (and dependents) to depth.
- `graph --id ADR-021 [--depth N] [--include-bidirectional] [--format mermaid|dot|json]` — Export a dependency graph around an ADR.
- `path --from ADR-011 --to ADR-038 [--max-depth N]` — Find a dependency path if any.
- `validate [--format json] [--dry-run] [--full-rescan] [--write-groups]` — Validate front matter/refs; on success writes indexes (unless `--dry-run`).
- Incremental by default: only reparses changed files using mtime/size. Use `--full-rescan` to force scanning all.
- Exits non-zero if validation fails.
- `watch [--debounce-ms 400] [--dry-run] [--full-rescan]` — Watch bases for changes and incrementally validate + update indexes.
- Debounces rapid events; writes on success (unless `--dry-run`).

## Decision
<!-- What is the change that we're proposing and/or doing? -->

### The Universal Flag

#### `--format Plain|Json|Ndjson`

All command EXCEPT for `graph` will accept a `--format` flag. This is designed for integrating the library with tooling and the eventual Model Context Protocol Server. 

### The Essential Functions 

#### Init 
Create `.adr-rag.toml` in the current repo and open it in an editor by default.

**Flags** 
- `--silent`: Write the default template directly to disk without opening the config in the editor 
- `--force`: Overwrite `.cli-rag.toml` even if it exists
- `--print-template`: Print template to stdout

#### validate 

Lint's notes based on the various [[schema]] configured in `.cli-rag.toml`. By default all index scans are incremental reparse's using `mtime` and `size`. Exits with an error code in the event validation fails. 
  
**Flags** 
- `--dry-run`: Validate the index and print errors without writing or updating the index on disk. 
- `--full-rescan`: Scans all files regardless of tracked `mtime` or `size`
- `--write-groups`: 

#### new (added from above)
`[--schema <S>] [--title <T>] [--id ...] [--edit]` 

- The primary tool for creating a new note. Can open the defined editor with the `--edit` flag 

#### watch 
`[--debounce-ms 400] [--dry-run] [--full-rescan]`

- Watch bases for changes and incrementally validate + update indexes.
- Debounces rapid events; writes on success (unless `--dry-run`).


### The Syntactic Sugar

#### doctor 
Show resolved config, bases, discovery mode (index vs scan), and quick stats.
- Reports per-type counts when schemas are defined, and unknown-key stats.

#### search 
`[--query <substr>]`
Fuzzy search by ID/title across discovered ADR files.

#### path --from ABC --to XYZ 
`[--max-depth N]` 
- Find a dependency path if any.

### Problematic/Underdeveloped User Story

#### group

```markdown 
#### topics
- List semantic groups derived from frontmatter (`groups` in ADRs).

#### group 
`[--topic "<name>"] [--include-content]` 
- Show ADRs in a group; defaults to full content.
```

These tools are currently not logically implemented and are are holdover from the original `masterplan-validate.js` this library was built out of. The issue is currently we do not have a clear user story around how these groups would be created or defined. 

The purpose of this tool is primarily to give an AI agent a map to understand the code-base. 

One option would be to define `groups` as `validated_frontmatter` with hard-coded logic in the [[ADR-001-cli-rag-toml]];

```toml
# --- `validated_frontmatter` are built tools that will run validation logic beyond simply checking if it exist and has content --- 
# |-----------------|--------------------------------------------------------------------------------|
# |  `depends_on`   | validates the id exists and is valid or throws and error                       |
# |-----------------|--------------------------------------------------------------------------------|
# | `created_date`  | Updates the note with system time when created using the `new` command         |
# |-----------------|--------------------------------------------------------------------------------|
# | `last_modified` | if `watch` is active the note will be updated with a new modified time on edit |
# |-----------------|--------------------------------------------------------------------------------|
# |     `groups`     | Defines subgroups for note types. These can generate a human/AI readable Index |
# |-----------------|--------------------------------------------------------------------------------|
validated_frontmatter = ["depends_on", "created_date", "last_modified", "group"]
```

This would allow the user to dynamically define groups as required. 

The second issue is the split name for `topics` and `groups` is not especially intuitive or ergonomic. Maybe it could be arranged in a single command like so; 

```markdown 
#### groups
`[--list] [--] []`

- `--list` print all semantic groups tracked by `validated_frontmatter` on the current graph.
```

---

#### graph

```markdown
#### graph 
`[--id ABCD] [--depth N] [--include-bidirectional] [--format mermaid|dot|json]`
- Export a dependency graph around an ADR.
```

There are two places the graph tool currently introduces uncertainty. 

Firstly the `--format` command collides with the global `--format plain|json|ndjson` formatting command. This should be changed eg `--graph-output`. This is a relatively minor concern. 

The deeper concern is just "Is this actually useful?" It's pretty commonly agreed that the graphview in discord is more visual fun and games than something truly productive and useful. There is a chance this is just not a feature that's a large value add (even if it's like, neat, and i want to see it.)

The thing that jump out is that it makes seeing "orphan" notes trivial. That said, we can just surface this information in doctor as well. 

An idea that keeps coming to mind is using the local `--include-bidirectional` graph as a navigation system in the TUI/NVIM. 

[Notably, graphiz directly supports ascii output.](https://graphviz.org/docs/outputs/ascii/) An imaginable workflow is we pull a local graph and append the ID's so it's 

1. ADR-001 
2. ADR-002 
   
You are then shown a screen like this where you can with a single key press to fly around notes e.g. 

```ascii
     ┌−−−−−−−−−−−−−−−−−−−−−−−−−−−−−┐
     ╎             adr             ╎
     ╎                             ╎
     ╎ ┌─────────┐     ┌─────────┐ ╎       ┌─────────┐
  ┌─ ╎ │ ADR-001 │ ◀── │ ADR-000 │ ╎ ◀──   │  start  │
  │  ╎ └─────────┘     └─────────┘ ╎       └─────────┘
  │  ╎   │               ▲         ╎         │
  │  ╎   │               │         ╎         │
  │  ╎   │               │         ╎         ▼
  │  ╎   │               │         ╎     ┌−−−−−−−−−−−−−┐
  │  ╎   │               │         ╎     ╎     imp     ╎
  │  ╎   ▼               │         ╎     ╎             ╎
  │  ╎ ┌─────────┐       │         ╎     ╎ ┌─────────┐ ╎
  │  ╎ │ ADR-002 │       │         ╎     ╎ │ IMP-001 │ ╎
  │  ╎ └─────────┘       │         ╎     ╎ └─────────┘ ╎
  │  ╎   │               │         ╎     ╎   │         ╎
  │  ╎   │               │         ╎     ╎   │         ╎
  │  ╎   │               │         ╎     ╎   ▼         ╎
  │  ╎   │               │         ╎     ╎ ┌─────────┐ ╎
  │  ╎   │               │         ╎     ╎ │ IMP-002 │ ╎
  │  ╎   │               │         ╎     ╎ └─────────┘ ╎
  │  ╎   │               │         ╎     ╎   │         ╎
  │  ╎   │               │         ╎     ╎   │         ╎
  │  ╎   │               │         ╎     ╎   ▼         ╎
  │  ╎   │             ┌─────────┐ ╎     ╎ ┌─────────┐ ╎
  │  ╎   └───────────▶ │ ADR-003 │ ╎ ◀── ╎ │ IMP-003 │ ╎
  │  ╎                 └─────────┘ ╎     ╎ └─────────┘ ╎
  │  ╎                             ╎     ╎   │         ╎
  │  └−−−−−−−−−−−−−−−−−−−−−−−−−−−−−┘     ╎   │         ╎
  │                      │               ╎   │         ╎
  │                      │               ╎   │         ╎
  │                      │               ╎   ▼         ╎
  │                      │               ╎ ┌─────────┐ ╎
  └──────────────────────┼─────────────▶ ╎ │ IMP-004 │ ╎
                         │               ╎ └─────────┘ ╎
                         │               ╎             ╎
                         │               └−−−−−−−−−−−−−┘
                         │                   │
                         │                   │
                         │                   ▼
                         │                 ┌─────────┐
                         └─────────────▶   │   end   │
                                           └─────────┘
```

It is to be completely honest still not entirely clear what the relative value here actually is. Worth thinking about further. We need the ability to filter this down. so 

- Only [[wikistyle links]]
- Only hard `superscedes` or `depends`. In order to not write custom logic it could be any frontmatter that matches a note title or ID based on the validation type defined in the schema. 

I think this is "in-theory" useful but it needs to be a very fast and intuitive UI to beat just using telescope or directly opening the notes in obsidian. 

---

```markdown 
#### get 
`--id ADR-021 [--include-dependents]` 
- Traverse a note (and it's dependencies) and print the contents. This tool is primarily designed for an AI consumer and is the primary 'RAG' in CLI-RAG. 

#### cluster 
`[--id ABCD] [--depth N] [--include-bidirectional]` 
```

Currently the user story here is a bit unclear. Cluster should probably just be a modifier e.g. 

- `get --id <ID>` (Gets the single node's content)
- `get --id <ID> --cluster [--depth N --include-bidirectional]` (Gets the content for the entire cluster/neighborhood)
- `graph --id <ID>` (Graphs the cluster, as graph implicitly operates on a cluster)

the point being the top level commands should be 'verbs' so all in;  

```markdown 
#### get 
`--id <ID> [--cluster] [--depth N] [--include-bidirectional]` 
Traverse a note (and it's dependencies) and print the contents. This tool is primarily designed for an AI consumer and is the primary 'RAG' in CLI-RAG. 
```

## LLM Sourced Feature Ideas/Changes 

- Deep Refactor Support (very useful for agents) 
	- `rename --id ADR-012 --new-id ADR-012R` updates filename and all `depends_on` references in repo (front matter and wiki-links in bodies).
	- `supersede --old ADR-008 --new ADR-031` writes both sides of the link (`supersedes`/`superseded_by`) 

- **Improved Output and Formatting**:
    - **Why?** Plain output is functional but can be noisy (e.g., long paths in `doctor`). JSON is great for scripting, but humans need summaries.
    - **How**: For plain mode, use tables (e.g., via `comfy_table` crate: add to Cargo.toml). In `doctor`, output like:
       
        ```
        +---------+-------+
        | Base    | Mode  |
        +---------+-------+
        | docs/   | index |
        +---------+-------+
        ```
        
	- Colorize errors/warnings in `validate` (use `anstream` for cross-platform coloring—already in your deps).

- **Autocompletion and ID Suggestions**:
    
    - **Why?** Commands like `get --id ADR-022` are error-prone if IDs are mistyped. Your `completions` command is a start—enhance with dynamic suggestions.
    - **How**: In `get`/`cluster`/`path`, if `--id` is invalid, fuzzy-search IDs (like your `search`) and suggest: "Did you mean ADR-022? (y/n)". Use `clap` 's built-in validation for runtime checks.
- **Config Ergonomics and Defaults**:
    
    - **Why?** The `.adr-rag.toml` is comprehensive but overwhelming (e.g., long schema comments). `init --print-template` is good—make it interactive.
    - **How**: In `init`, prompt for key values (e.g., "Enter bases (comma-separated):") using `dialoguer` crate (add to Cargo.toml: `dialoguer = "0.11"`). Add a `config edit` command that opens the file (like your `try_open_editor`).
- **Error Handling and UX**:
    
    - **Why?** Rust's `anyhow` is used well, but user-facing errors could be friendlier (e.g., "Config not found—run init?").
    - **How**: In `load_config`, if no config, prompt to run `init`. Add verbose logging with `--debug` (use `env_logger` crate). For `validate`, group errors by file/schema for easier debugging.
    - 
- **Advanced Validation Rules**:
    
    - **Why?** Your schemas are already powerful (e.g., required fields, allowed statuses, regex for `depends_on`). Extend to cyclic dependency detection or status consistency (e.g., warn if a "draft" ADR depends on a "superseded" one).
    - **How**: In `validate_docs`, after building `id_to_docs` and `id_set`, add a graph traversal to detect cycles (using your existing `compute_cluster`). For status checks, walk dependencies and compare against `allowed_statuses` or schema rules.
    - **Ergonomic Twist**: Add a `--fix` flag to `validate` that auto-updates statuses (e.g., propagate "superseded") or suggests resolutions in JSON output.

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

A more ergonomic and intuitive developer experience. 
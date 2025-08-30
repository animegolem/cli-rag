---
id: ADR-003a
tags:
  - cli
  - refactor
status: accepted
depends_on:
  - ADR-001
  - ADR-004
created_date: 2025-08-24
last_modified: 2025-08-25
related_files:
  - ~/cli-rag/src/commands
---

# ADR-004-CLI-refactor-planning

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
Lint's notes based on the various `[[schema]]` configured in `.cli-rag.toml`. By default all index scans are incremental reparse's using `mtime` and `size`. Exits with an error code in the event validation fails. Writes out `index_relative` unless `--dry-run` is used.
  
**Flags** 
- `--dry-run`: Validate the index and print errors without writing or updating the index on disk. Prints errors or success. 
- `--full-rescan`: Scans all files regardless of tracked `mtime` or `size`
- `--write-groups`: Write out only `groups_relative` and updates `semantic-groups.json`. This requires an active schema with `validated_frontmatter` and `groups` active.  
- `--write-all`: Write out both `groups_relative` and `index_relative`. 

#### new 
The primary tool for creating a new note. Can open the defined editor with the `--edit` flag 
  
**Flags**
- `--schema <S>`: Select a .toml defined schema to use as a template. 
- `--title <T>`: Create a new note defined by Title. 
- `--id <I>`: Create a new note defined by ID. 
- `--edit`: Open the note in the defined editor. 

---

#### watch 
Watches filepaths for changes and incrementally validates & update the indexes. Debounces rapid events; writes on success (unless `--dry-run`).

**Flags**
- `[--debounce-ms 400]`: Set custom debounce time. 
- `[--dry-run]`: Watches file changes but does not write to the index. 
- `[--full-rescan]`: Fully scans all tracked paths and updates the index. 
  
  ```markdown 
  - when watching, ignore temp/lockfiles (`**/*~`, `**/.#*`, `**/*.tmp`) and editor swap dirs.
  ```

### The Syntactic Sugar

#### doctor 
Show resolved config, filepaths, discovery mode (index vs scan), and quick stats. Reports per-type counts when schemas are defined, and unknown-frontmatter stats.

#### search --query `<substr>`

Fuzzy search by ID/title across tracked notes.

**Flags**
- `--schema <S>`: Search only one defined schema 

#### path --from ABC --to XYZ 
Returns the shortest dependency path between two notes or throws and error if none exist.
  
**Flags**
- `[--max-depth N]`: Set maximum number of recursive notes the path will traverse. 
  
  
### Problematic/Underdeveloped User Story

#### group

```markdown 
##### topics
- List semantic groups derived from frontmatter (`groups` in ADRs).

##### group 
`[--topic "<name>"] [--include-content]` 
- Show ADRs in a group; defaults to full content.
```

These tools are currently not logically implemented and are are holdover from the original `masterplan-validate.js` this library was built out of. The issue is currently we do not have a clear user story around how these groups would be created or defined. 

The purpose of this tool is primarily to give an AI agent a map to understand the code-base. 

One option would be to define `groups` as `validated_frontmatter` with hard-coded logic in the [[ADR-001-cli-rag.toml]];

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

##### groups
lists tracked notes organized by `groups` frontmatter. If the frontmatter is not active this will throw errors. 

**Flags**
- `--list`: print all semantic groups tracked by `validated_frontmatter` on the current graph.

---

#### graph

```markdown
##### graph 
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

I'm not totally sure how useful this would be but it's roughly equal to the obsidian local graph. it's worth a shot. However, it doesn't seem like it requires new graph primitives other than eventually adding an ascii output. 

##### graph 
Export a dependency graph around an ID.

- `[--id ABCD]: Define the ID or TITLE for the root node of the local graph to display. 
- `[--depth N] `: Set maximum number of recursive notes the graph will include. 
- `[--include-bidirectional] `: Include both dependencies as well as notes depending on the selected ID.  
- `[--format mermaid|dot|json|ascii]`: Select the output format. If undefined it prints graphviz ascii in the terminal. 

---

#### Get

```markdown 

##### get 
`--id ADR-021 [--include-dependents]` 
- Traverse a note (and it's dependencies) and print the contents. This tool is primarily designed for an AI consumer and is the primary 'RAG' in CLI-RAG. 

##### cluster 
`[--id ABCD] [--depth N] [--include-bidirectional]` 
```

Currently the user story here is a bit unclear. Cluster should probably just be a modifier e.g. 

- `get --id <ID>` (Gets the single node's content)
- `get --id <ID> --cluster [--depth N --include-bidirectional]` (Gets the content for the entire cluster/neighborhood)
- `graph --id <ID>` (Graphs the cluster, as graph implicitly operates on a cluster)

the point being the top level commands should be 'verbs' so all in;  

##### get 
Traverse a note (and it's dependencies) and print the contents. This tool is primarily designed for an AI consumer and is the primary 'RAG' in CLI-RAG. 

**Flags**
- `--id <ID>`: Define any tracked note ID 
- `[--cluster] `: Pull an ID and it's connections as defined by the following flags. 
- `[--depth N] `: Set maximum number of recursive notes the path will traverse. 
- `[--include-bidirectional]`: Include dependencies and notes depending on the selected ID.  

## Better Error Handling and Output 

```markdown 
- **Autocompletion and ID Suggestions**:
    
    - **Why?** Commands like `get --id ADR-022` are error-prone if IDs are mistyped. Your `completions` command is a start—enhance with dynamic suggestions.
    - **How**: In `get`/`cluster`/`path`, if `--id` is invalid, fuzzy-search IDs (like your `search`) and suggest: "Did you mean ADR-022? (y/n)". Use `clap` 's built-in validation for runtime checks.
- **Config Ergonomics and Defaults**:
```

```markdown 
- **Error Handling and UX**:
    
    - **Why?** Rust's `anyhow` is used well, but user-facing errors could be friendlier (e.g., "Config not found—run init?").
    - **How**: In `load_config`, if no config, prompt to run `init`. Add verbose logging with `--debug` (use `env_logger` crate). For `validate`, group errors by file/schema for easier debugging.
```

```markdown
- **Improved Output and Formatting**:
    - **Why?** Plain output is functional but can be noisy (e.g., long paths in `doctor`). JSON is great for scripting, but humans need summaries.
    - **How**: For plain mode, use tables (e.g., via `comfy_table` crate: add to Cargo.toml). In `doctor`, output like:
        
        +---------+-------+
        | Base    | Mode  |
        +---------+-------+
        | docs/   | index |
        +---------+-------+
       
	- Colorize errors/warnings in `validate` (use `anstream` for cross-platform coloring—already in your deps).
```


---

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

A more ergonomic and intuitive developer experience. see [[ADR-003b-v1-CLI-commands]]
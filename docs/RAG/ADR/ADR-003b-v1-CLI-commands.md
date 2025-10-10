---
id: ADR-003b
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

# v1-CLI-commands


## Objective
<!-- A concise statement explaining the goal of this decision. -->

Present a concise "final" list of commands before full implementation. These tools are designed for either a human or an AI consumer.


## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

---

#### Init 
Create `.cli-rag.toml` in the current repo and open it in an editor by default.

**Flags** 
- `--schema <S>`: Opens a commented but blank scheme template in the primary editor. Optionally define a name| Name is required with `silent`. 
- `--silent`: Write the default template directly to disk without opening the config in the editor 
- `--force`: Overwrite `.cli-rag.toml` even if it exists
- `--print-template`: Print template to stdout

---

#### new 
The primary tool for creating a new note. Can open the defined editor with the `--edit` flag. If  
  
**Flags**
- `--schema <S>`: Select a .toml defined schema to use as a template. 
- `--title <T>`: Create a new note defined by Title. 
- `--id <I>`: Create a new note defined by ID. 
- `--edit`: Open the note in the defined editor. 
- `--agent`: Generates an agent friendly text prompt.

---

#### get 
Traverse a note (and it's dependencies) and print the contents. This tool is primarily designed for an AI consumer and is the primary 'RAG' in CLI-RAG. 

**Flags**
- `--id <ID>`: Get any tracked note by unique ID 
- `[--depth N] `: Set maximum number of recursive notes the path will traverse. 0 returns the node only. `Defaults = 2`
- `[--include-bidirectional]`: Include both dependencies as well as notes depending on the selected ID.  

---

#### watch 
Watches filepaths for changes and incrementally validates & update the indexes. Debounces rapid events; writes on success (unless `--dry-run`).

**Flags**
- `[--debounce-ms 400]`: Set custom debounce time. 
- `[--dry-run]`: Watches file changes but does not write to the index. 
- `[--full-rescan]`: Fully scans all tracked paths and updates the index. 

---

#### graph 
Export a dependency graph around an ID. Prints the local graph in the terminal. 

**Flags**
- `[--id ABCD]: Define the ID or TITLE for the root node of the local graph to display. 
- `[--depth N] `: Set maximum number of recursive notes the graph will include. 0 returns the node only. `defaults = 2`
- `[--include-bidirectional] `: Include both dependencies as well as notes depending on the selected ID.  
- `[--format mermaid|dot|json|ascii]`: Select the output format. If undefined it prints graphviz ascii in the terminal. 

An idea that keeps coming to mind is using the local graph as a navigation system navigation system for the tui/cli. The key is that [graphiz directly supports ascii output.](https://graphviz.org/docs/outputs/ascii/) This creates a possible workflow where we pull a local graph and append the ID's so it's 

1. ADR-001 
2. ADR-002 
   
You are then shown a screen like this where you can with a single key press to fly around notes. we could open them in a pager or an editor (and that could be a flag in the .toml) e.g. 
```
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

---

#### doctor 
Show resolved config, filepaths, discovery mode (index vs scan), and quick stats. Reports per-type counts when schemas are defined, and unknown-frontmatter stats.

```markdown
- **Improved Output and Formatting**:
    - **Why?** Plain output is functional but can be noisy (e.g., long paths in `doctor`). JSON is great for scripting, but humans need summaries.
    - **How**: For plain mode, use tables (e.g., via `comfy_table` crate: add to Cargo.toml). In `doctor`, output like:
        
        +---------+-------+
        | Base    | Mode  |
        +---------+-------+
        | docs/   | index |
        +---------+-------+
        
	- Colorize errors/warnings in `validate` useing `anstream` for cross-platform coloring
```

---

#### search --query `<substr>`
Fuzzy search by ID/title across tracked notes.

**Flags**
- `--schema <S>`: Search in only a single defined schema 
- `--recent <X>`: Lists the last `X` changes tracked on the index

---

#### path --from ABC --to XYZ 
Returns the shortest dependency path between two notes or throws and error if none exist.
  
**Flags**
- `[--max-depth N]`: Set maximum number of recursive notes the path will traverse. 

output should show graph edges 
```bash 
ADR-024 → IMP-006 (depends_on)                  # explicit frontmatter
IMP-006 → ADR-029 (mentions: [[ADR-029]] in body, L42)
```

---

#### groups
lists tracked notes organized by `groups` frontmatter. If the frontmatter is not active this will throw errors. 

**Flags**
- `--list`: print all semantic groups tracked by `validated_frontmatter` on the current graph.

---

#### validate 
Lint's notes based on the various `[[schema]]` configured in `.cli-rag.toml`. By default all index scans are incremental reparse's using `mtime` and `size`. Exits with an error code in the event validation fails. Writes out `index_relative` unless `--dry-run` is used.
  
**Flags** 
- `--dry-run`: Validate the index and print errors without writing or updating the index on disk. Prints errors or success. 
- `--full-rescan`: Scans all files regardless of tracked `mtime` or `size`
- `--write-groups`: Write out only `groups_relative` and updates `semantic-groups.json`. This requires an active schema with `validated_frontmatter` and `groups` active.  
- `--write-all`: Write out both `groups_relative` and `index_relative`. 

---

### Ideas...

#### Help 
Prints on stdout 

**Flags**

---

#### list

we need some kind of command to list out "all in progress, all closed, all draft." this implies this logic needs to be baked in deeper for basic agile states. this need probably should be part of search or a graph action etc it takes a deeper rethink of the verb space now that i have a clearer idea what actions are needed. 


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
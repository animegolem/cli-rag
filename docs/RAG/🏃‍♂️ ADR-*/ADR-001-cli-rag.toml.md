---
id: ADR-001
tags:
  - config
  - toml
  - cli
  - north-star
status: accepted
depends_on:
created_date: 2025-08-24
last_modified: 2025-08-30
related_files:
  - .cli-rag.toml
---

# cli-rag.toml

## Objective
<!-- A concise statement explaining the goal of this decision. -->

At the time of writing this document is **aspirational** and is not in line with the actual codebase. The intention of this effort is to create a north star that explains the UX vision for the application. 

The .toml is designed to be very extensible but we will create a [[ADR-010-the-LUA-escape-hatch]] escape hatch for highly custom use-cases. 
  
## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

### Example `cli-rag.toml` Block 

if we see issues we can add an option to shard the index but i don't instantly think it's likely; 

```toml
[scan.index]
strategy = "sharded"
shard_by = "schema"  # Separate AIM-*, ADR-*, etc.
max_shard_size_mb = 10
```

famous last words. 


```TOML
[[config]]
#: =============================================================================
#:                            # --- Version --- #
#: =============================================================================
#: Sets the config version for all top level rules. All imported notes inherit
#: the same version.
#: An individual `[[schema]`] can call this to remain on an older version
config_version = 0.1

#: =============================================================================
#:                            # --- SCAN --- #
#: =============================================================================
#: Settings related to discovery, filepaths, and scan process
[config.scan]
#: The root directories that cli-rag will scan for notes.
#: All paths must be relative to the location of this `.cli-rag.toml` file.
#: Defaults to the current directory ["."] if left empty.
filepaths = ["file/path", "file/other/path"]
#: By default, an index will be created at '.cli-rag/rag-index.json'.
#: This can be given an arbitrary override.
#: File paths are relative to the location of this `.cli-rag.toml` file.
index_path = "alternate/filepath/rag-index.json"
#: `hash_mode` controls HOW the tool detects if a file has changed.
#: Defaults to `mtime`.
#: +-----------+-----------------------------------------------------------+
#: |  `mtime`  |   Fast but less accurate.                                 |
#: |           |   Compares the file's "last modified" timestamp.          |
#: +-----------+-----------------------------------------------------------+
#: | `content` |  100% Accurate but slower.                                |
#: |           |  Computes a hash of the entire file's content.            |
#: +-----------+-----------------------------------------------------------+
hash_mode = "mtime"
#: `index_strategy` controls what is stored in the index for search.
#: Defaults to content.
#: +------------+----------------------------------------------------------+
#: | `metadata` | Fastest option. Stores only front matter and links.      |
#: +------------+----------------------------------------------------------+
#: | `content`  | (Full-Text Search) Stores all metadata AND the           |
#: |            | full text of every note.                                 |
#: +------------+----------------------------------------------------------+
index_strategy = "content"
#tuning for large repos 
# parallel_workers = 4
# cache_strategy = "aggressive"
#: Remove directories or patterns from scanning to improve speed.
#: Patterns are relative to the location of this `.cli-rag.toml` file.
ignore_globs = ["**/node_modules/**", "**/dist/**"]

#: =============================================================================
#:                            # --- AUTHORING --- #
#: =============================================================================
#: Settings related to creating and editing notes
[config.authoring]
#: The editor to launch for new or existing notes.
#: Uses $EDITOR or $VISUAL if not set.
editor = "neovim"
#: runs the `watch` command as a background process for live updates when
#: the neovim plugin is open and active.
background_watch = true

#: =============================================================================
#:                             # --- GRAPH --- #
#: =============================================================================
#: Default settings for graph traversal commands (get, graph, path)
[config.graph]
#: Default depth for traversing dependencies.
depth = 2
#: Whether to include dependents (backlinks) in traversals.
include_bidirectional = true

#: =============================================================================
#:                            # --- RETRIEVAL --- #
#: =============================================================================
#: Default settings for content retrieval commands (get, groups)
[config.retrieval]
#: Whether to include the full markdown content in the output.
include_content = true

#: =============================================================================
#:                        # --- TEMPLATE MANAGEMENT --- #
#: =============================================================================
#: For a cleaner .`cli-rag.toml` it's advised to import an external schema.
#: Alternatively one or more `[[schema]]` may be defined inline below
[config.templates]
import = [".cli-rag/templates/ADR.toml", ".cli-rag/templates/RPG.toml"]

#: =============================================================================
#:                            # --- SCHEMA --- #
#: =============================================================================
#: 1. Define `[[schema]]` blocks to create a note type. Assign a `name`.
#: 2. (Optional) Pin a `config_version` to prevent breaking changes.
#: 2. Set one or more `file_patterns` for discovery.
#: 3. Define the rules for filename generation.
#: 5. Use `template` to define the full structure of the final note.
#: 6. Define the custom and system frontmatter if present.
```

### Templates 

#### ID Defined Schema(s)

```toml
#: =============================================================================
#:                                # --- SCHEMA --- #
#: =============================================================================
#: 1. Define `[[schema]]` blocks to create a note type. Assign a `name`.
#: 2. (Optional) Pin a `config_version` to prevent breaking changes.
#: 3. Set one or more `file_patterns` for discovery.
#: 4. Define the rules for filename generation.
#: 5. Use `template` to define the full structure of the final note.
#: 6. Define the custom and system frontmatter if present.
#: 7. Define custom validation rules using `schema.validate`.

[[schema]]
name = "ADR"
#: =============================================================================
#:                            # --- Version --- #
#: =============================================================================
#: Sets the config version for all top level rules.
#: If not set this is inherited from `.cli-rag.toml`
config_version = 0.1
#: =============================================================================
#:                           # --- DISCOVERY --- #
#: =============================================================================
#: Set discovery rules for notes types. Accepts globs/regex.
#: Notes must be under your defined filepaths in `.cli-rag.toml` to be discovered.
file_patterns = ["ADR-*.md", "ADR-DB-*.md"]

#: =============================================================================
#:                     # --- TEMPLATES & GENERATION --- #
#: =============================================================================
#: The `[schema.new]` block configures the `new` command and defines how
#: notes are generated.
#: This block configures both how the note is tracked and what template is used.
#: If no `id_generator`is defined the note will be tracked by filename.
#: The `filename_template` defines the structure for the output filename.
#:
#:                    #--- FILENAME TEMPLATE BUILDER --- #
#:   Advanced rules for creating custom titles when using the 'new' command
#: +-------------------+-------------------------------------------------------+
#: |        Filter     |                   Description                         |
#: +-------------------+-------------------------------------------------------+
#: | `{{title}}`       |  Injects the string from the --title flag.            |
#: +-------------------+-------------------------------------------------------+
#: | `{{id}}`          |  Injects the stable ID from the id_generator.         |
#: |                   |  Use only if the `id_generator` is defined.           |
#: +-------------------+-------------------------------------------------------+
#: | `{{schema.name}}` |  Injects the name of the schema. Used as-is.          |
#: +-------------------+-------------------------------------------------------+
#: | `{{now}}`         |  Injects system time. Default = ISO 8601              |
#: +-------------------+-------------------------------------------------------+
#:
#:                       # --- Modifier Rules --- #
#: +-----------------------+--------------------------------------------------+
#: |        Filter         |                    Example                       |
#: +-----------------------+--------------------------------------------------+
#: |` kebab-case `         | {{title|kebab-case}} --> "my-new-feature"        |
#: +-----------------------+--------------------------------------------------+
#: | `snake_case`          | {{title|snake_case}} --> "my_new_feature"        |
#: +-----------------------+--------------------------------------------------+
#: | `SCREAMING_SNAKE_CASE`| {{title|SCREAMING_SNAKE_CASE}} -->               |
#: |                       | "MY_NEW_FEATURE"                                 |
#: +-----------------------+--------------------------------------------------+
#: | `camelCase`           | {{title|camelCase}} --> "myNewFeature"           |
#: +-----------------------+--------------------------------------------------+
#: | `PascalCase`          | {{title|PascalCase}} --> "MyNewFeature"          |
#: +-----------------------+--------------------------------------------------+
#: | `date:"<strftime>"`   | {{now \| date:"%Y-%m-%d"}} -> "2025-08-26"       |
#: +-----------------------+--------------------------------------------------+
[schema.new]
#: Define the name template for the `new` command. unset = filename
id_generator = { strategy = "increment", prefix = "ADR-" padding = "3" }
#: Options are ["increment", "datetime", "uuid"]
#: Prefix is not mandatory if using the later two options
filename_template = "{{id}}-{{title|kebab-case}}.md"

#: =============================================================================
#:                           # --- TEMPLATES --- #
#: =============================================================================
#: Manually defined frontmatter via the template are not tracked by `validate`.
#: The variables listed below are injected by the `new` command.
#: 
#:                       # --- TEMPLATE VARIABLES --- #
#: +-----------------+---------------------------------------------------------+
#: | {{id}}          | Inject the `ID` field as defined by `id_generator`.|
#: |                 | Fallsback to filename.                                  |
#: +-----------------+---------------------------------------------------------+
#: | {{title}}       | Inject the title provided via the `--title <T>` flag    |
#: |                 | on the `new` command.                                   |
#: +-----------------+---------------------------------------------------------+
#: | ((frontmatter)) | Inject items within the `schema.frontmatter` table into |
#: |                 | the template                                            |
#: +-----------------+---------------------------------------------------------+
#: | {{LOC|100}}     | Set the maximum number of lines per heading.            |
#: +-----------------+---------------------------------------------------------+
#: | {{date}}        | Today's date. **Default format:** `YYYY-MM-DD`.         |
#: +-----------------+---------------------------------------------------------+
#: | {{time}}        | Current time. **Default format:** `HH:mm`.              |
#: +-----------------+---------------------------------------------------------+
[schema.new.template]
[schema.new.template.prompt]
template = """
# Instructions
**Generate an Architectural Decision Record based on the user's request.**
- Fill out the template below exactly as provided.
- For the 'status' field, you MUST use one of the following values:
- [draft, proposed, accepted, superseded, cancelled].
- For the 'tags' field, provide a comma-separated list of relevant technical tags.
- Fill in the content for each heading (Objective, Context, etc.) based on the user's goal.
"""
[schema.new.template.note]
template = """
((frontmatter))

# {{filename}}

## Objective
<!-- A concise statement explaining the goal of this decision. -->
{{LOC|50}}

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->
{{LOC|300}}

## Decision
<!-- What is the change that we're proposing and/or doing? -->
{{LOC|50}}

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->
{{LOC|50}}

## Updates
<!-- Changes that happened when the rubber met the road -->
{{LOC|100}}
"""

#: =============================================================================
#:            # --- ADVANCED: CUSTOM TEMPLATE GENERATOR --- #
#: =============================================================================
#: Use the OpenAPI contract to write a custom lua template manager. Buyer Beware. 
# lua_generator = "path/to/script.lua"


#: =============================================================================
#:                        # --- ~.FRONTMATTER  --- #
#: =============================================================================
#: Frontmatter can be "System" or "User".
#: Named Graph edges can be defined as well. 
#: Only frontmatter defined below will be tracked by the `validate` command.
#: 
#:                      # --- FRONT MATTER SYLES --- #
#: +-----------------------+---------------------------------------------------+
#: | `system_frontmatter`  | Runs pre-configured logic against YAML            |
#: |                       | frontmatter. See table below.                     |
#: +-----------------------+---------------------------------------------------+
#: | `user_frontmatter`    | Define arbitrary YAML frontmatter. Validation     |
#: |                       | logic can be created via String, Glob or regex.   |
#: +-----------------------+---------------------------------------------------+
#: | `graph_edges`         | Define all tracked/labeled graph edges for the    |
#: |                       | DAG. Will error if ID does not exist in index.    |
#: +-----------------------+---------------------------------------------------+
#: | `~.frontmatter.GTD`   | System frontmatter that triggers inclusion on     |
#: |                       | the main screen on neovim and the GTD command     |
#: +-----------------------+---------------------------------------------------+
#:
#:                      # --- SYSTEM_FRONTMATTER --- #
#: +-----------------+---------------------------------------------------------+
#: | `node_id`       | Defines the note on the DAG using `id_generator`.       |
#: +-----------------+---------------------------------------------------------+
#: | `created_date`   | Updates the note with system time when using `new`      |
#: +-----------------+---------------------------------------------------------+
#: | `last_modified` | if `watch` is active the note will be updated with a    |
#: |                 | new modified time on edit                               |
#: +-----------------+---------------------------------------------------------+
#: | `groups`        | Defines subgroups for note types. These can generate a  |
#: |                 | human/AI readable Index                                 |
#: +-----------------+---------------------------------------------------------+
#:
#:                      # --- `~.frontmatter.GTD` --- #                         
#: +-----------------+---------------------------------------------------------+
#: | `kanban_status` | Define Kanban status. Deplayed on Agenda screen. Provide|
#: |                 | a table of legal statuses.                              |
#: +-----------------+---------------------------------------------------------+
#: | `kanban_        | Define a status line that shows under the kanban staus  | 
#: | statusline`     | on the agenda screen. true or false. `default = "false"`|
#: +-----------------+---------------------------------------------------------+
#: | `due_date`      | Set a due date. Appears on neovim agenda x days prior.  | 
#: |                 | true or false to enable.                                |
#: +-----------------+---------------------------------------------------------+
  
[schema.frontmatter]
 # Explicitly defined system fields with special behavior
system_frontmatter = [
 "node_id",
 "created_date",
 "last_modified",
 "groups",
 ]

# Regular user fields
user_frontmatter = [
  "tags",
  "priority",
  "confidence_score"
  ]

# Graph edge fields - automatically validated as node references
graph_edges = [
  "depends_on",
  "blocked_by",
  "supersedes",
  # Additional edges can be defined freely. 
  ]

[schema.frontmatter.GTD] 
kanban_status = [ 
  "backlog", 
  "planned", 
  "in-progress", 
  "completed",
  "cancelled"
  ]
kanban_statusline = true
due_date = true

#: =============================================================================
#:                           # --- VALIDATION --- #
#: =============================================================================
#: This block configures the `validate` command largely using globs and regex.
#: Construct your configuration using the options below.
#: 
#:                         # --- VALIDATORS KEYS --- #
#: +---------------------------------+--------------------+---------------------+
#: | Validator Table                 | Configuration Key  | Description/Options |
#: +---------------------------------+--------------------+---------------------+
#: | [schema.validate]               | Top level config inherited to all tables |
#: |                                 |--------------------+---------------------|
#: |                                 | `severity`         | accepts "error"     |
#: |                                 |  (all tables)      | "warning", "ignore" |
#: +---------------------------------+--------------------+---------------------+
#: | [schema.validate.frontmatter]   | Top most Table for frontmatter config    | 
#: +---------------------------------+--------------------+---------------------+
#: | ↳ [~.frontmatter.allow_unknown] | Determine behavior for undefined         |
#: |                                 | frontmatter                              |
#: |                                 |--------------------+---------------------|
#: |                                 | `allow_unknown`    | Policy for extra    |
#: |                                 |                    | fields.             |
#: |                                 |                    | Options: "true",    |
#: |                                 |                    | "false",            |
#: +---------------------------------+--------------------+---------------------+
#: | ↳ [~.frontmatter.fields]        | Custom rules against `user_frontmatter`  |
#: |                                 |------------------------------------------|
#: |                                 | `regex`            | A single regex      |
#: |                                 |                    | string.             |
#: |                                 +--------------------+---------------------+
#: |                                 | `enum`             | An array of exact   |
#: |                                 |                    | string matches.     |
#: |                                 +--------------------+---------------------+
#: |                                 | `globs`            | An array of glob    |
#: |                                 |                    | patterns.           |
#: |                                 +--------------------+---------------------+
#: |                                 | `float`            | A table with        |
#: |                                 |                    | optional `min` and  |
#: |                                 |                    | `max` float values. |
#: |                                 +--------------------+---------------------+
#: |                                 | `integer`          | A table with        |
#: |                                 |                    | optional `min` and  |
#: |                                 |                    | `max` float values. |
#: +---------------------------------+--------------------+---------------------+
#: | [schema.validate.body]          | Top most Table for body validation       |
#: +---------------------------------+--------------------+---------------------+
#: | ↳ [~.body.headings]             | Various body Validation settings         |
#: |                                 |------------------------------------------|
#: |                                 | `heading_policy`   | How to check heads. |
#: |                                 |                    | Options: "strict",  |
#: |                                 |                    | "missing_only",     |
#: |                                 |                    | "ignore".           |
#: |                                 |--------------------+---------------------+
#: |                                 | `max_count`        | INT.Maximum Nunmber |
#: |                                 |                    | of headings allowed |
#: |                                 |                    | in a document.      |
#: +---------------------------------+--------------------+---------------------+
#: | ↳ [~.body.line_count]           | Prevents the LLM from outputting over    |
#: |                                 | `x` LOC                                  |
#: |                                 |--------------------+---------------------|
#: |                                 | `scan_policy`      | When to run check.  |
#: |                                 |                    | Options:            |
#: |                                 |                    | "on_creation",      |
#: |                                 |                    | "on_validate".      |
#: +---------------------------------+--------------------+---------------------+
#: | [schema.validate.edges]         | Top level most for all graph config      | 
#: |                                 |------------------------------------------|
#: |                                 | `required_edges`   | Define if an edge   |
#: |                                 |                    | is MUST be set      |
#: +---------------------------------+--------------------+---------------------+
#: | ↳ [~.edges.wikilinks]           | Rules covering [[obsidian style]] links  |
#: |                                 |------------------------------------------|
#: |                                 | `min_outgoing`     | Int. Min required   |
#: |                                 |                    | [[wikilinks]] out.  |
#: |                                 |--------------------+---------------------+
#: |                                 | `min_incoming`     | Int. Min required   |
#: |                                 |                    | [[wikilinks]] in.   |
#: +---------------------------------+--------------------+---------------------+
#: | ↳ [~.edges.cross_schema]        | Rules for mutli schema DAGs.             |
#: |                                 |------------------------------------------|
#: |                                 | `allowed_targets`  | Define if graph     |
#: |                                 |                    |edges traverse schema|
#: +---------------------------------+--------------------+---------------------+
#: | [schema.validate.GTD]           | Top level table for all GTD config       |
#: |                                 |--------------------+---------------------|
#: |                                 | `enable_TODO`      | Determine if TODO   |
#: |                                 |                    | will be scanned on  |
#: |                                 |                    | notes. true/false   |
#: |                                 |--------------------+---------------------|
#: |                                 | `enable_kanban`    | Determine if Kanban |
#: |                                 |                    | will be scanned on  |
#: |                                 |                    | notes. true/false   |  
#: |                                 |--------------------+---------------------|
#: |                                 | `enable_kanban     | Determine if Kanban |
#: |                                 | statusline`        | status will be used |     
#: |                                 |                    | for the agenda. T/F |   
#: |                                 |--------------------+---------------------|
#: |                                 | `due_date_warning` | due dates will show |
#: |                                 |                    | on the agenda screen|     
#: |                                 |                    | when due in X days. |   
#: |                                 |--------------------+---------------------|
#: |                                 | `detect_cycles`    | enable or disable   |
#: |                                 |                    | cycle detection.    |     
#: |                                 |                    | true or false.      |   
#: +----------------------------------------------------------------------------+
[schema.validate]
severity = "error" # Default severity for all validate rules

#: =============================================================================
#:                   # --- ADVANCED: CUSTOM VALIDATOR --- #
#: =============================================================================
#: Use the OpenAPI contract to write a custom lua validator. Buyer Beware. 
# lua_validator = "path/to/script.lua"

#: =============================================================================
#:                     # --- VALIDATOR: FRONTMATTER --- #
#: =============================================================================
[schema.validate.frontmatter]
[schema.validate.frontmatter.allow_unknown]
#: Policy for fields not explicitly listed in `custom_frontmatter` 
#: or `validated_frontmatter`.
#: default = true
allow_unknown = true
severity = "warning" # Overrides default 
#:  --- Field-specific Rules ---  :#
[schema.validate.frontmatter.fields]
id     = { regex = "^ADR-\\d{3}$" }
tags   = { regex = "^[^\\n]*$", severity = "warning" } 
related_files = { regex = ["\\.exs?$", "\\.py$", "\\.js$", "\\.md$", "\\.toml$"] }
depends_on = { globs = ["ADR-*", "ADR-DB-*"], severity = "warning" } 
blocked_by = { globs = ["ADR-*", "ADR-DB-*"], severity = "warning" } 
priority = { integer = { min = 0, max = 100 } 
confidence_score = { float = { min = 0.0, max = 1.0 }, severity = "error" }

#: =============================================================================
#:                       # --- VALIDATOR: BODY --- #
#: =============================================================================
[schema.validate.body]
[schema.validate.body.headings]
#: Policy for matching headings against the note template.
#: Options: [
#: "exact" (exact match), "missing_only" (template headings requiured
#: + additional headings are allowed), "ignore"
#: ]
heading_check = "missing_only"
max_count = 10 
severity = "warning" # Override default 
[schema.validate.body.line_count]
#: When to perform this check.
#: define using {{LOC|`X`}} in the template manager.
#: This set the number of lines **per heading**
#: Options: "on_creation", "on_validate".
scan_policy = "on_creation"

#: =============================================================================
#:                      # --- VALIDATOR: EDGES --- #
#: =============================================================================
[schema.validate.edges]
#: All edges will be added to front matter but only the following will cause 
#: errors if unset. 
required_edges = ["depends_on"]
detect_cycles = true
[schema.validate.edges.wikilinks]
severity = "warning"
min_outgoing = 1
min_incoming = 0
[schema.validate.edges.cross_schema]
# Define schema's that may be be set as a DAG edge. 
allowed_targets = ["ADR", "IMP", "ADR-AI"]

#: =============================================================================
#:                       # --- VALIDATOR: GTD --- #
#: =============================================================================
[schema.validate.GTD]
# Determine if {TODO@high}: note content is tracked and added to the agenda  
enable_TODO = true
# Determine if the kanban yaml is tracked and added to the agenda  
enable_kanban = true
# Determine if statusline is added to agenda, if present. 
enable_kanban_statusline = true
# Items will be added to the agenda when due in X days. Accepts ints. 
due_date_warning = 5 
severity = "warning"
```

## Tech Details 

### In-Scope 
- We can force Titles to the correct case using either `heck` or `convert_case`. 
- We should ship with a folder with a few example schema's that users can modify or use directly before 1.0. This is the escape hatch so you can use the tool for a bit and decide if it's even worth setting up a config file.	  
- for speed connections, backlinks etc need to be tracked in the index. 
- the groups and file index have been collapsed to a single item vs the current codebase. We also no longer allow multiple indexes in a single repo.  

## Decision
<!-- What is the change that we're proposing and/or doing? -->

Tentatively adopt the above framework in full with WASM accepted but deferred as future scope. Exact details will be determined in implementation.  

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

A new implementation ticket will be created to capture the remedial work needed to align the codebase with the above "north star" in a form of configuration driven development.
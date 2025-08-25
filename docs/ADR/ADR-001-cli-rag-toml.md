---
id: ADR-001
tags: [config, toml, cli, tera]
status: proposed
depends_on: none
created_date: 2025-08-24
last_modified: 2025-08-24
related_files: [.cli-rag.toml]
---

# cli-rag-toml

## Example `.toml` Block 

```TOML
[file.paths]
# --- The root directories that cli-rag will scan for file changes. --- 
# --- Mutliple inputs are accepted and will be merged. ---
filepaths = [
"file/path",
"file/other/path",
]
  
# --- By default the index will be created at `bases-path(s)/index/*.json`. ---
# --- File paths used below must be present at the same directory level or below this .toml file. --- 
# index_relative = "alternate/filepath/adr-index.json"
# groups_relative = "alternate/filepath/semantic-groups.json"

# --- Remove directories or patterns from scanning to improve speed in large projects. ---     
ignore_globs  = ["**/node_modules/**", "**/src/**", "**/lib/**", "**/dist/**"]
 
# --- Control the default retrevial settings for adr-rag cluster etc --- 
[defaults]
default_editor = "micro"
depth = 2
include_bidirectional = true
include_content = true

# --- Define a title format. legal are `kebab-case`, `camelCase`, `snake_case`, `SCREAMING_SNAKE_CASE`, and `PascalCase`. Default is `kebab-case` ---
# --- This setting is applied globally but is overridden by if called again in a specific schema ---
# title.format = "kebab-case"

# --- SCHEMAS ---
# 1. Define `[[schema]]` blocks to create a note type and map how it's asvalidated. 
# 2. Set one or more `file_patterns`. These will determine what notes your rules are applied to. Notes must be under your defined filepaths to be discovered. 
# 3. Set `identity_source` to "frontmatter" for rich yaml notes or "title" for simple applications that only require wiki style linking. 
# 4. Define the plain and validated frontmatter if present plus any regex or wasm rules 
# 5. Use `template` to define the full structure of the final note. You can use placeholders like {{id}}, {{title}} and ((frontmatter)) for dynamic entry 

I should make [[table]] here to show all variable options once i know what that list ! ! ! 

# --- For a cleaner .`cli-rag.toml` you can provide a full schema in one or multiple dedicated documents ---  
# import = "templates/adr.toml"
# import = "templates/lore.toml"


# --- Front Matter Defined Notes --- 
[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md", "ADR-DB-*.md"]
identity_source = "frontmatter" 

# --- Required frontmatter will require the item be present and not empty in order to succeed without error --- 
# --- specific regex rules can be manually defined --- 
required_frontmatter = ["id", "tags", "status", "related_files"]

# --- Determines if undefined frontmatter should thrown an error if detected. Use if your notes should have ONLY the required frontmatter. Useful for limiting llm creativity.  ---
# -- Accepts `strict` and `loose`. Defaults to loose if undefined. ---
unknown_frontmatter_policy = "strict" 

# --- `validated_frontmatter` are built tools that will run validation logic beyond simply checking if it exist and has content --- 
# |-----------------|--------------------------------------------------------------------------------|
# |  `depends_on`   | validates the id exists and is valid or throws and error                       |
# |-----------------|--------------------------------------------------------------------------------|
# | `created_date`  | Updates the note with system time when created using the `new` command         |
# |-----------------|--------------------------------------------------------------------------------|
# | `last_modified` | if `watch` is active the note will be updated with a new modified time on edit |
# |-----------------|--------------------------------------------------------------------------------|
# |     `groups`    | Defines subgroups for note types. These can generate a human/AI readable Index |
# |-----------------|--------------------------------------------------------------------------------|
validated_frontmatter = ["depends_on", "created_date", "last_modified", "group"]

# --- Determine if missing headings from the template should throw an error. ---
# -- Accepts `strict` and `loose`. Defaults to loose if undefined. ---
heading_policy = "loose"

# --- Set specific rules for legal frontmatter ---  
# --- Severity can be set to Error | Warning | Ignore -- 
[schema.rules.status]
allowed_plain = ["draft", "proposed", "accepted", "superseded", "cancelled"]
severity = "error"

[schema.rules.related_files]
allowed_regex = ["*.ex*", "*.py", "*.js", "*.md", "*.toml"]
severity = "warning"

# --- ADVANCED USERS: Create a WASM module that handles all validation for your schema. Buyer beware. --- 
# validator_wasm = "validators/ADR.wasm"

# --- Complete template for new notes. This is used by the `new` command. Treat this as an LLM prompt. ---  
template = """
((required_frontmatter))

# {{id}}-{{title}}

## Objective
<!-- A concise statement explaining the goal of this decision. -->

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

## Decision
<!-- What is the change that we're proposing and/or doing? -->

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

## Updates
<!-- Changes that happened when the rubber met the road -->
"""

# --- Alterantively you can create simple file name defined notes with minimal linking --- 
# [[schema]]
# name = "Lore"
# file_patterns = ["lore-*.md"]
# identity_source = "filename"
# minimum.links = "2"
# minimum.backlinks = "0"

# template = """
# {{title}}

## Persona
# <!-- The core of the character -->

## Story so Far 
# <!-- Compressed recent events --> 

## Related
# <!-- Links to related content --> 
# """
```

## Tech Details 

- We can create the final notes from the .toml using `Tera` | https://github.com/Keats/tera
- Titles can be forced to the correct case using either `heck` or `convert_case`. 
- A **very large** number of names will need to be changed across the codebase to reflect the above. 
- The `validator_wasm` is more v2.0 north star in concept. The number of users likely to use it is vanishingly small but it's a compelling idea. 


## Scratchpad 

- The defaults section isn't good and should probably be broken up logically and also just expanded out sensibly. I'm not sold on these categories but the idea feels broadly fair.  

```toml
# --- Settings related to creating and editing notes ---
[authoring]
# The editor to launch for new or existing notes.
# Uses $EDITOR or $VISUAL if not set.
editor = "micro"

# The case format for titles when creating new notes from a template.
# legal are `kebab-case`, `camelCase`, `snake_case`, etc.
# This can be overridden per-schema.
title_format = "kebab-case"

# Default status to apply to a new note if not specified in the template.
# default_status = "draft"

# --- Default settings for graph traversal commands (cluster, graph, path) ---
[graph]
# Default depth for traversing dependencies.
depth = 2

# Whether to include dependents (backlinks) in traversals.
include_bidirectional = true

# --- Default settings for content retrieval commands (get, group) ---
[retrieval]
# Whether to include the full markdown content in the output.
include_content = true
```

## Update Log
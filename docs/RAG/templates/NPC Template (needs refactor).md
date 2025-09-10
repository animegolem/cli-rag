#### Filename Defined Schema(s)

# needs refactor 

```TOML
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
name = "NPC"
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
file_patterns = ["NPC-*.md"]

#: =============================================================================
#:                    # --- TEMPLATES & GENERATION --- #
#: =============================================================================
#: The `[schema.new]` block configures the `new` command and defines how
#: notes are generated.
#: This block configures both how the note is tracked and what template is used.
#: If no `id_generator`is defined the note will be tracked by filename.
#: The `filename_template` defines the structure for the output filename.
#:
#:                    #--- ADVANCED TITLE BUILDER --- #
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
#:                        # --- Modifier Rules --- #
#:  +-----------------------+--------------------------------------------------+
#:  |        Filter         |                    Example                       |
#:  +-----------------------+--------------------------------------------------+
#:  |` kebab-case `         | {{title|kebab-case}} --> "my-new-feature"        |
#:  +-----------------------+--------------------------------------------------+
#:  | `snake_case`          | {{title|snake_case}} --> "my_new_feature"        |
#:  +-----------------------+--------------------------------------------------+
#:  | `SCREAMING_SNAKE_CASE`| {{title|SCREAMING_SNAKE_CASE}} -->               |
#:  |                       | "MY_NEW_FEATURE"                                 |
#:  +-----------------------+--------------------------------------------------+
#:  | `camelCase`           | {{title|camelCase}} --> "myNewFeature"           |
#:  +-----------------------+--------------------------------------------------+
#:  | `PascalCase`          | {{title|PascalCase}} --> "MyNewFeature"          |
#:  +-----------------------+--------------------------------------------------+
#:  | `date:"<strftime>"`   | {{now \| date:"%Y-%m-%d"}} -> "2025-08-26"       |
#:  +-----------------------+--------------------------------------------------+
[schema.new]
filename_template = "{{schema.name}}-{{title|kebab-case}}.md"

#: =============================================================================
#:                           # --- TEMPLATES --- #
#: =============================================================================
#: Manually defined frontmatter via the template are not tracked by `validate`.
#: The variables listed below are injected by the `new` command.
#: 
#:                       # --- TEMPLATE VARIABLES --- #
#: +-----------------+---------------------------------------------------------+
#: | {{id}}          | Inject the `id` field as defined by `id_generator`.     |
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
- Review the recent chat history. Who is this NPC? What did the 
  player ask for specifically?
- How do they fit into the world you and the player have built? What 
  motivates them?
- Ensure you add at least 2 links to existing NPC's, Items, or Locations.
- Fill out the template below exactly as provided.
"""

[schema.new.template.note]
template = """
---
created_on: {{date}}
---

# {{title}}

## Persona
<!-- The core of the character -->
{{LOC|200}}

## Objective
<!-- What do they want and how does it relate to the user -->
{{LOC|50}}

## Story so Far
<!-- Compressed recent events -->
{{LOC|50}}

## Related
<!-- Links to related content -->
{{LOC|10}}
"""

#: =============================================================================
#:            # --- ADVANCED: CUSTOM TEMPLATE GENERATOR --- #
#: =============================================================================
#: Use the OpenAPI contract to write a custom lua template manager. Buyer Beware. 
# lua_generator = "path/to/script.lua"

#: =============================================================================
#:                         # --- FRONTMATTER  --- #
#: =============================================================================
#: Frontmatter can be "System" or "User".
#: Only frontmatter defined  below will be tracked by the `validate` command.

#:                   # --- FRONT MATTER SYLES --- #
#: +-----------------------+---------------------------------------------------+
#: | `system_frontmatter`  | Runs pre-configured logic against YAML            |
#: |                       | frontmatter. See table below.                     |
#: +-----------------------+---------------------------------------------------+
#: | `user_frontmatter`    | Define arbitrary YAML frontmatter. Validation     |
#: |                       | logic can be created via String, Glob or regex.   |
#: +-----------------------+---------------------------------------------------+

#:                      # --- SYSTEM_FRONTMATTER --- #
#: + --------------------------------------------------------------------------+
#: |                    Front matter with Predefined Logic                     |
#: +-----------------+---------------------------------------------------------+
#: | `depends_on`    | Validates the id is valid and exists.  false = error    |
#: +-----------------+---------------------------------------------------------+
#: | `created_date`  | Updates the note with system time when using `new`      |
#: +-----------------+---------------------------------------------------------+
#: | `last_modified` | if `watch` is active the note will be updated with a    |
#: |                 | new modified time on edit                               |
#: +-----------------+---------------------------------------------------------+
#: | `groups`        | Defines subgroups for note types. These can generate a  |
#: |                 | human/AI readable Index                                 |
#: +-----------------+---------------------------------------------------------+
#: Since this is a simple filename based template no frontmatter is defined

#: =============================================================================
#:                          # --- VALIDATION --- #
#: =============================================================================
#: This block configures the `validate` command using globs and regex.
#: Construct your configuration using the options below.

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
#: |                                 |------------------------------------------|
#: |                                 | `severity`         | accepts "error"     |
#: |                                 |                    | "warning", "ignore" |
#: +---------------------------------+--------------------+---------------------+
#: | [schema.validate.frontmatter]   | Top most Table for frontmatter config    | 
#: +---------------------------------+--------------------+---------------------+
#: | ↳ [~.frontmatter.allow_unknown] | Determine behavior for undefined         |
#: |                                 | frontmatter                              |
#: |                                 |------------------------------------------|
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
#: |                                 |------------------------------------------|
#: |                                 | `scan_policy`      | When to run check.  |
#: |                                 |                    | Options:            |
#: |                                 |                    | "on_creation",      |
#: |                                 |                    | "on_validate".      |
#: +---------------------------------+--------------------+---------------------+
#: | [schema.validate.edges]         | Top level most for all graph config      | 
#: +---------------------------------+--------------------+---------------------+
#: | ↳ [~.edges.wikilinks]           | Rules covering [[obsidian style]] links  |
#: |                                 |------------------------------------------|
#: |                                 | `min_outgoing`     | Int. Min required   |
#: |                                 |                    | [[wikilinks]] out.  |
#: |                                 |--------------------+---------------------+
#: |                                 | `min_incoming`     | Int. Min required   |
#: |                                 |                    | [[wikilinks]] in.   |
#: +---------------------------------+--------------------+---------------------+
[schema.validate]
severity = "error" # Default severity for all validate rules

#: =============================================================================
#:                   # --- ADVANCED: CUSTOM VALIDATOR --- #
#: =============================================================================
#: Use the OpenAPI contract to write a custom lua validator. Buyer Beware. 
# lua_validator = "path/to/script.lua"

#: =============================================================================
#:                      # --- VALIDATOR: FRONTMATTER --- #
#: =============================================================================
#: This note does not define any managed frontmatter rules.

#: =============================================================================
#:                         # --- VALIDATOR: BODY --- #
#: =============================================================================
[schema.validate.body]
[schema.validate.body.headings]
#: Policy for matching headings against the note template.
#: Options:#: "exact", "ignore", "missing_only" 
#: (template headings requiured + additional headings are allowed)
heading_policy = "missing_only"
max_count = 6
severity = "warning"
[schema.validate.body.line_count]
#: When to perform this check.
#: Options: "on_creation", "on_validate".
scan_policy = "on_creation"

#: =============================================================================
#:                        # --- VALIDATOR: EDGES --- #
#: =============================================================================
[schema.validate.edges]
[schema.validate.edges.wikilinks]
min_outgoing = 2
min_incoming = 0
```
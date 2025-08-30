---
id: ADR-001
tags:
  - config
  - toml
  - cli
  - teraallow_unknown
status: accepted
depends_on:
created_date: 2025-08-24
last_modified: 2025-08-26
related_files:
  - .cli-rag.toml
---

# cli-rag-toml

## Objective
<!-- A concise statement explaining the goal of this decision. -->

At the time of writing this document is **aspirational** and is not in line with the actual codebase. The intention of this effort is to create a north star that explains the UX vision for the application. 

*this idea was deferred/cancelled for simple text prompts. We'd probably want to take the json back from the LLM and mold the template eg tera but the value story is non-obvious*

```markdown
The `[[schema]]` can be *very* explicit because they are designed to be [map-able directly to an OpenAPI schema for forcing structured output as a prompt via the `new` command. 
](https://ai.google.dev/gemini-api/docs/structured-output)

This serves 2 functions: 

1. **Validation (Input)**: Defines the rules for parsing and validating handwritten Markdown files. 
2. **Generation (Output)**: Defines the structure that an AI must conform to when it generates a new note from a prompt.
```
 
 The `validate` command should in all cases enforce unique filenames across the graph. If the `id_generator` i used it will lead to notes being generated/enforced with an incrementing numeric ID. 
  
## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

### Example `cli-rag.toml` Block 

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
			filepaths = [
			"file/path",
			"file/other/path",
			]
				
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
				
			#: Remove directories or patterns from scanning to improve speed.
			#: Patterns are relative to the location of this `.cli-rag.toml` file.
			ignore_globs  = ["**/node_modules/**", "**/dist/**"]
				
		#: =============================================================================
		#:                            # --- AUTHORING --- #
		#: =============================================================================
		#: Settings related to creating and editing notes
		[config.authoring]
			#: The editor to launch for new or existing notes.
			#: Uses $EDITOR or $VISUAL if not set.
			# editor = "neovim"
				
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
		#: Alternatively one or more `[[schema]]` may be defined directly below  
		[config.templates]
			import = [
			".cli-rag/templates/ADR.toml", 
			".cli-rag/templates/RPG.toml",
			]
		
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
	#:                    # --- TEMPLATES & GENERATION --- # 
	#: =============================================================================
	#: The `[schema.new]` block configures the `new` command and defines how
	#: notes are generated.
	#: This block configures both how the note is tracked and what template is used. 
	#: If no `id_generator`is defined the note will be tracked by filename. 
	#: The `filename_template` defines the structure for the output filename.
	
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
		#: Define the name template for the `new` command. unset = filename 
		#: Options are ["increment", "datetime", "uuid"]
		#: Prefix is not mandatory if using the later two options
		id_generator = { strategy = "increment", prefix = "ADR-" padding = "3" } 
		filename_template = "{{id}}-{{title|kebab-case}}.md"
		
		#: =============================================================================
		#:                           # --- TEMPLATES --- #
		#: =============================================================================
		#: Manually defined frontmatter via the template are not tracked by `validate`. 
		#: The variables listed below are injected by the `new` command. 
		 
		#:                       # --- TEMPLATE VARIABLES --- #          
		#: +-----------------+---------------------------------------------------------+ 
		#: | {{id}}          | Inject the `id` field as defined by `id_generator`.     |
		#: |                 | Errors if not present.                                  | 
		#: +-----------------+---------------------------------------------------------+ 
		#: | {{title}}       | Inject the title provided via the `--title <T>` flag    |
		#: |                 | on the `new` command.                                   | 
		#: +-----------------+---------------------------------------------------------+ 
		#: | {{filename}}    | Inject the predefined filename string in full.          |
		#: +-----------------+---------------------------------------------------------+ 
		#: | ((frontmatter)) | Inject items within the `schema.frontmatter` table into |
		#: |                 | the template                                            | 
		#: +-----------------+---------------------------------------------------------+ 
		#: | {{LOC|100}}     | Set the maximum number of lines per heading.            |
		#: |                 | Only applies to LLM outputs.                            | 
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
				{{LOC|200}} 				
					
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
	#:                         # --- FRONTMATTER  --- # 
	#: =============================================================================
	#: Frontmatter can be "System" or "User". 
	#: Only frontmatter defined below will be tracked by the `validate` command. 
		
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
	#: | `depends_on`    | Validates the id is valid/note exists. False = error    |   
	#: +-----------------+---------------------------------------------------------+                            
	#: | `created_date`  | Updates the note with system time when using `new`      | 
	#: +-----------------+---------------------------------------------------------+                              
	#: | `last_modified` | if `watch` is active the note will be updated with a    | 
	#: |                 | new modified time on edit                               |
	#: +-----------------+---------------------------------------------------------+   
	#: | `groups`        | Defines subgroups for note types. These can generate a  |
	#: |                 | human/AI readable Index                                 |                            
	#: +-----------------+---------------------------------------------------------+                           
	[schema.frontmatter]
	   	system_frontmatter = ["depends_on", "created_date", "last_modified", "groups"]
		user_frontmatter = ["id", "tags", "status", "related_files"]
		
		
	#: =============================================================================    
	#:                           # --- VALIDATION --- #
	#: =============================================================================
	#: This block configures the `validate` command largely using globs and regex. 
	#: Construct your configuration using the options below. 
		
#:                           # --- VALIDATORS KEYS --- #
#: +---------------------------------+--------------------+---------------------+
#: | Validator Table                 | Configuration Key  | Description/Options |
#: +---------------------------------+--------------------+---------------------+
#: | [schema.validate]               | severity           | accepts "error"     |
#: |                                 |                    | "warning", "ignore" |
#: +---------------------------------+--------------------+---------------------+
#: | [schema.validate.frontmatter]   | severity           | Default severity    |
#: |                                 |                    | for all frontmatter |
#: |                                 |                    | rules.              |
#: +---------------------------------+--------------------+---------------------+
#: | [~.frontmatter.allow_unknown]   | Determine undefined frontmatter behavior |
#: |                                 |--------------------+---------------------+
#: |                                 | allow_unknown      | Policy for extra    |
#: |                                 |                    | fields.             |
#: |                                 |                    | Options: "true",    |
#: |                                 |                    | "false",            |
#: +---------------------------------+--------------------+---------------------+
#: | [~.frontmatter.FIELD_NAME]      | Custom rules against `user_frontmatter`  |
#: |                                 |--------------------+---------------------+
#: |                                 | legal_entry        | A regex string or   |
#: |                                 |                    | array of strings to |
#: |                                 |                    | validate a field.   |
#: +---------------------------------+--------------------+---------------------+
#: | [schema.validate.body]          | severity           | severity of check.  |
#: +---------------------------------+--------------------+---------------------+
#: | [~.body.headings]               | Various body validation settings         |
#: |                                 |--------------------+---------------------+
#: |                                 | heading_strictness | How to check heads. |
#: |                                 |                    | Options: "strict",  |
#: |                                 |                    | "missing_only",     |
#: |                                 |                    | "ignore".           |
#: |                                 |--------------------+---------------------+
#: |                                 | max_count          | INT.Maximum Nunmber |
#: |                                 |                    | of headings allowed |
#: |                                 |                    | in a document.      |
#: +---------------------------------+--------------------+---------------------+
#: | [~.body.line_count]             | Ristricts LLM output `x` LOC per heading |
#: |                                 |--------------------+---------------------+
#: |                                 | scan_policy        | When to run check.  |
#: |                                 |                    | Options:            |
#: |                                 |                    | "on_creation",      |
#: |                                 |                    | "on_validate".      |
#: +---------------------------------+--------------------+---------------------+
#: | [schema.validate.edges]         | severity           | severity of check.  |
#: +---------------------------------+--------------------+---------------------+
#: | [~.edges.wikilinks]             | Rules covering [[obsidian style]] links  |
#: |                                 |--------------------+---------------------+
#: |                                 | min_outgoing       | Int. Min required   |
#: |                                 |                    | [[wikilinks]] out.  |
#: |                                 |--------------------+---------------------+
#: |                                 | min_incoming       | Int. Min required   |
#: |                                 |                    | [[wikilinks]] in.   |
#: +---------------------------------+--------------------+---------------------+
	[schema.validate]
		severity = "error" # Default severity for all validate rules
		#: =============================================================================
		#:                     # --- VALIDATOR: FRONTMATTER --- #
		#: =============================================================================
	    [schema.validate.frontmatter]
		    [schema.validate.frontmatter.allow_unknown]
		        #: Policy for fields not explicitly listed in `custom_frontmatter` or `validated_frontmatter`.
		        #: default = true  
		        allow_unknown = "true"
				severity = "warning"		        
	        # --- Field-specific Rules --- #
		    [schema.validate.frontmatter.id]
	            legal_entry = '^ADR-\d{3}$' 
		            
	        [schema.validate.frontmatter.status]
	            legal_entry = ["draft", "proposed", "accepted", "superseded", "cancelled"]
		            
	        [schema.validate.frontmatter.related_files]
	            legal_entry = ["\\.exs?$","\\.py$","\\.js$","\\.md$","\\.toml$"]
	            severity = "warning" # Override default
		            
	        [schema.validate.frontmatter.depends_on]
	            legal_entry = ["ADR-*", "ADR-DB-*"]
	            severity = "warning" # Override default
	                
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
		        heading_strictness = "missing_only"
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
		    [schema.validate.edges.wikilinks]
		        severity = "warning"
		        min_outgoing = 1
		        min_incoming = 0
```

#### Filename Defined Schema(s)

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
		 
		#:                       # --- TEMPLATE VARIABLES --- #          
		#: +-----------------+---------------------------------------------------------+ 
		#: | {{id}}          | Inject the `id` field as defined by `id_generator`.     |
		#: |                 | Errors if not present.                                  | 
		#: +-----------------+---------------------------------------------------------+ 
		#: | {{title}}       | Inject the title provided via the `--title <T>` flag    |
		#: |                 | on the `new` command.                                   | 
		#: +-----------------+---------------------------------------------------------+ 
		#: | ((frontmatter)) | Inject items within the `schema.frontmatter` table into |
		#: |                 | the template                                            | 
		#: +-----------------+---------------------------------------------------------+ 
		#: | {{LOC|100}}     | Set the maximum number of lines per heading.            |
		#: |                 | Only applies to LLM outputs.                            | 
		#: +-----------------+---------------------------------------------------------+ 
		#: | {{date}}        | Today's date. **Default format:** `YYYY-MM-DD`.         |   
		#: +-----------------+---------------------------------------------------------+  
		#: | {{time}}        | Current time. **Default format:** `HH:mm`.              |                            
		#: +-----------------+---------------------------------------------------------+ 
		[schema.new.template]
			[schema.new.template.prompt]
			template = """
			# Instructions
			- Review the recent chat history. Who is this NPC? What did the player ask for specifically? 
			- How do they fit into the world you and the player have built? What motivates them? 
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
		
	#:                         # --- VALIDATORS KEYS --- # 	
	#: +---------------------------------+-------------------------+---------------------+
	#: | Validator Table                 | Configuration Key       | Description/Options |
	#: +---------------------------------+-------------------------+---------------------+
	#: | [schema.validate]               | severity                | accepts "error"     |
	#: |                                 |                         | "warning", "ignore" |
	#: +---------------------------------+-------------------------+---------------------+
	#: | [schema.validate.frontmatter]   | severity                | Default severity    |
	#: |                                 |                         | for all frontmatter |
	#: |                                 |                         | rules.              |
	#: +---------------------------------+-------------------------+---------------------+
	#: | [~.frontmatter.allow_unknown]   | Determine behavior for undefined frontmatter  |
	#: |                                 |-------------------------+---------------------+
	#: |                                 | allow_unknown           | Policy for extra    |
	#: |                                 |                         | fields.             |
	#: |                                 |                         | Options: "true",    |
	#: |                                 |                         | "false",            |
	#: +---------------------------------+-------------------------+---------------------+
	#: | [~.frontmatter.FIELD_NAME]      | Custom rules against `user_frontmatter`       |
	#: |                                 |-------------------------+---------------------+
	#: |                                 | legal_entry             | A regex string or   |
	#: |                                 |                         | array of strings to |
	#: |                                 |                         | validate a field.   |
	#: +---------------------------------+-------------------------+---------------------+
	#: | [schema.validate.body]          | severity                | severity of check.  |
	#: +---------------------------------+-------------------------+---------------------+
	#: | [~.body.headings]               | Various eading Validation settings            |
	#: |                                 |-------------------------+---------------------+
    #: |                                 | heading_strictness      | How to check heads. |
	#: |                                 |                         | Options: "strict",  |
	#: |                                 |                         | "missing_only",     |
	#: |                                 |                         | "ignore".           |
	#: |                                 |-------------------------+---------------------+
	#: |                                 | max_count               | INT.Maximum Nunmber |
	#: |                                 |                         | of headings allowed |
	#: |                                 |                         | in a document.      |
	#: +---------------------------------+-------------------------+---------------------+
	#: | [~.body.line_count]             | Prevents the LLM from outputting over `x` LOC |
	#: |                                 |-------------------------+---------------------+
	#: |                                 | scan_policy             | When to run check.  |
	#: |                                 |                         | Options:            |
	#: |                                 |                         | "on_creation",      |
	#: |                                 |                         | "on_validate".      |
	#: +---------------------------------+-------------------------+---------------------+
	#: | [schema.validate.edges]         | severity                | severity of check.  |
	#: +---------------------------------+-------------------------+---------------------+
	#: | [~.edges.wikilinks]             | Rules covering [[obsidian style]] links       |
	#: |                                 |-------------------------+---------------------+
	#: |                                 | min_outgoing            | Int. Min required   |
	#: |                                 |                         | [[wikilinks]] out.  |
	#: |                                 |-------------------------+---------------------+
	#: |                                 | min_incoming            | Int. Min required   |
	#: |                                 |                         | [[wikilinks]] in.   |
	#: +---------------------------------+-------------------------+---------------------+
	[schema.validate]
		severity = "error" # Default severity for all validate rules
		#: =============================================================================
		#:                     # --- VALIDATOR: FRONTMATTER --- #
		#: =============================================================================
	    #: This note does not define any managed frontmatter rules.  
	          
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
		        heading_strictness = "missing_only"
				severity = "warning"
	        [schema.validate.body.line_count]
		        #: When to perform this check.
		        #: Options: "on_creation", "on_validate".
		        scan_policy = "on_creation"
		#: =============================================================================
		#:                      # --- VALIDATOR: EDGES --- #
		#: =============================================================================
	    [schema.validate.edges]
		    [schema.validate.edges.wikilinks]
		        min_outgoing = 2
		        min_incoming = 0		
```

## Tech Details 

### In-Scope 
- We can force Titles to the correct case using either `heck` or `convert_case`. 
- We should ship with a folder with a few example schema's that users can modify or use directly before 1.0. This is the escape hatch so you can use the tool for a bit and decide if it's even worth setting up a config file. These need to be pretty well considered and usable.  
- We should limit filesystem writes as much as possible outside of our index and explicit user actions. For "last modified" we might be able to track updates in the index but only write out the file in a pre-commit hook. The downside is it requires user action. created_date may at some point just be removed if it's problematic. 
	- Thinking about this more if the file is getting written to how much load actually is a second write a fraction of a second later. not convinced it's a big deal on second thought. 
- for speed connections, backlinks etc need to be tracked in the index. 
- the groups and file index have been collapsed to a single item. 

### Deferred/Future-Scope 
- We could potentially create the final notes from the .toml using [Tera](https://github.com/Keats/tera). I think the practical choice here is to DEFER until such time we see an actual issue. 
- The `validator_wasm` is more v2.0 north star in concept. The number of users likely to use it is vanishingly small but it's a compelling idea. Sandboxing would be a hard requirement. 
```TOML
# --- ADVANCED USERS: Create a WASM module that handles all validation for your schema. Runs in a sandbox. Buyer beware. --- 
validator_wasm = "validators/ADR.wasm"
```
- Hybrid index is deferred. The ideal is that we create and store metadata + vector embeddings but the question of how we make the LLM call adds non-trivial complexity. 
```
#: Define what is persisted to the index
#: accepts ["metadata", "content", "hybrid"] 
#: metadata = structured data only | content =  | hybrid = 
index.strategy = "metadata|content|hybrid"
```  

## Decision
<!-- What is the change that we're proposing and/or doing? -->

Tentatively adopt the above framework in full with WASM accepted but deferred as future scope. Exact details will be determined in implementation.  

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

A new implementation ticket will be created to capture the remedial work needed to align the codebase with the above "north star" in a form of configuration driven development. 

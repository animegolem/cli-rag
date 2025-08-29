---
id: ADR-011
tags:
  - winnow
  - parsers
  - templates
  - markdown 
status: draft
depends_on: 
  - ADR-001
  - ADR-010
  - ADR-009
created_date: 2025-08-28
last_modified: 2025-08-28
related_files: [.cli-rag.toml]
---

# ADR-010-text-parsing

## Objective
<!-- A concise statement explaining the goal of this decision. -->

Tera has many many features we don't need. To keep the implementation simple we will lean on a few custom rules [using the winnow crate. 
](https://docs.rs/winnow/latest/winnow/index.html)

We do not plan to reinvent the wheel and will lean on a mature markdown parser for non-custom logic. The main options are markdown-rs and pulldown-cmark. 

At this time the most likely pattern is producing an AST with markdown-rs and then leaning on markdown-oxide for majority of edge functions. things like `depends on` still need custom logic. 

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

### Issue 
Our template manager presents various items to parse. 

```
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
```

### something like...

```LLM
1. **Adopt oxide for editor features.** Let it drive:
    - Completions for `[[...]]`, tags, headings/blocks; backlinks UI; rename-updates. [Markdown-Oxide Wiki](https://oxide.md/)
        
2. **Overlay your semantics via your plugin:**
    - On buffer read/change, call your CLI (`get --format json`) → receive parsed wikilinks + your computed edges.
    - **Highlight** custom edge kinds with **extmarks** (you don’t need a Tree-sitter fork).
    - **Completions**: write a tiny `cmp` source that merges oxide LSP items with your CLI index (IDs, tags, ADRs).
        
3. **Validation** remains yours: run `validate --json` and surface diagnostics through Neovim’s `vim.diagnostic` (spans map 1:1).
```

## Decision
<!-- What is the change that we're proposing and/or doing? -->

Accept the stack in principal knowing some specifics will be hammered out in implementation. 

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

Fast parsing minimizing code we have to directly maintain where sane. 

## Updates
<!-- Changes that happened when the rubber met the road -->

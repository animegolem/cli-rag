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

https://github.com/artempyanykh/marksman
https://github.com/Feel-ix-343/markdown-oxide

marksman seems like the more mature product  most likely. 

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


### future... 

```llm 
# Two paths for the Neovim side

## A) Zero-grammar fork (fast to ship)

Use the official **tree-sitter-markdown** (+ `markdown_inline`) for base Markdown. Add semantic highlights and completions via extmarks and `nvim-cmp`:

- For **graph edges** (`[[ADR-123]]`, `depends_on:` in YAML), get byte spans from your **markdown-rs** pass and set highlights via extmarks; you don’t need a new TS node type.
    
- For **tag completions**, use a cmp source that triggers:
    
    - inside YAML `tags:` values, or
        
    - after `[[` (wikilinks)  
        The cmp source can query your project index (IDs, tags) and insert candidates.
        

Pros: no grammar maintenance; works today.  
Cons: highlights are “semantic” (extmarks), not query-driven.

## B) Tiny grammar delta (cleaner long-term)

Fork **tree-sitter-markdown_inline** and add just your inline tokens:

- `wikilink` → children: `(page) (heading?) (alias?)`
    
- `loc_marker` → `{{LOC|N}}`
    
- `filter_pipe` → `{{title|kebab}}`
    
- (optional) `fm_tag` inside front-matter values
    

Then ship minimal queries:

**queries/markdown/highlights.scm**

```scm
; wikilinks
(wikilink (page) @clirag.id)
(wikilink (heading) @clirag.fragment)
(wikilink (alias) @clirag.alias)

; frontmatter tags
(fm_tag) @clirag.tag

; loc markers left in body
(loc_marker) @clirag.todo ; highlight loudly


**queries/markdown/injections.scm** (if you ever want injections)


; e.g., inject dot/mermaid code fences
(fenced_code_block
  (info_string) @injection.language
  (code_fence_content) @injection.content)


Now your **cmp** source can ask “what node is under cursor?” and suggest the right things (IDs for `wikilink/page`, tag values for `fm_tag`). Pros: all UI logic stays in TS queries; cleaner highlighting. Cons: maintain a tiny fork. 
```

## Decision
<!-- What is the change that we're proposing and/or doing? -->

Proceed with markdown-rs and accept the stack in principal knowing some specifics will be hammered out in implementation. 

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

Fast parsing minimizing code we have to directly maintain where sane. 

## Updates
<!-- Changes that happened when the rubber met the road -->

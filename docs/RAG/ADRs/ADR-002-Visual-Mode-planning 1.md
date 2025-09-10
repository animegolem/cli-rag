---
id: ADR-002
tags:
  - TUI
  - NeoVIM
  - rataTUI
  - Emacs
status: draft
depends_on:
  - ADR-01
  - ADR-003
created_date: 2025-08-28
last_modified: 2025-08-28
related_files: []
---

# Visual-Mode-planning

## Objective
<!-- A concise statement explaining the goal of this decision. -->

Determine a visual UI north star. This Document is not concerned with if this is implemented in TUI/Neovim/EMACS and is purely focused on defining the workflow.  

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

### AgendaView 

The main dashboard for the KB and application. 

This screen 
1. Presents current TODO's on tracked notes. These are created anywhere in a tracked note e.g. [{TODO}: Note that displays as a reminder]
2. Presents any upcoming or past due due-dates. 
3. Lists projects by status if the kanban_status and kanban_statusline front matter are active. 
4. Lists tracked notes and templates. 

The - [] todo items are live. If checked the application works like an LSP and updates the original instance in the linked note. ~~[{TODO}: Note that displays as a reminder]~~ 

The date it's marked done is tracked in the index and it falls off the agenda screen either 24 hours later or per user setting in the toml.  
   
  ```bash 
V ToDo ! 
	-[] TODO@HIGH: Critical parser bug | Note:[[IMP-005]] | Created: 2025-09-01 
	-[] TODO@MED:  Refactor validation  | Note:[[IMP-005]] | Created: 2025-08-27
	-[] TODO: Default priority task | Note:[[IMP-003]] | Created: 2025-09-03
V Due_Dates
    - 2025-8-16: IMP-001 **OVERDUE!** # past due color coded red and gets a flag 
    - 2025-9-03: IMP-004 
V **Kanban**
	> Backlog
	V In-Progress
	  L IMP-AI-004 | unified initial error codes; multi-schema E200; per-schema cycle policy; JSON/NDJSON codes + locations. 
	  L IMP-AI-002 | Introduce config_version with deprecation warnings and an upgrade helper.
	> Completed
	> Cancelled 
 
v **Templates** 
	1 - IMP-*  
	2 - ADR-*
	3 - ADR-AI-*
v **ADR** 
	L ADR-001-cli-rag.toml
	L ADR-002-Visual-Mode-planning
> **ADR-AI** 
v **IMP** 
	L IMP-001-config-loader-invariants
	L IMP-002-config-versioning-and-upgrade  

======================================================================================== 
TAB: Fold | RETURN: Select | SPACE+F: FuzzyFind | SPACE+G: GraphView | Space+E: EditView 
```

### FuzzyView

A limited fuzzy finder scoped down to only files tracked by the project schema's. Selecting an item opens it in edit view. 

We can lean on the Fuzzy Matcher crate https://github.com/skim-rs/fuzzy-matcher. 

```bash
docs/RAG/ADR-AI/ADR-AI-004-acp-aligned-protocol-surfaces.md
docs/RAG/ADRs/ADR-007-general-error-codes-ideation.md
docs/RAG/ADR-AI/ADR-AI-003-extensible-graph-edges.md
docs/RAG/ADR-AI/ADR-AI-002-gtd-kanban-integration.md
docs/RAG/ADRs/ADR-006-config-loader-error-codes.md
docs/RAG/ADRs/ADR-001b-cli-rag.toml-(patch_1).md
docs/RAG/ADRs/ADR-013-notebook-documentation.md
docs/RAG/ADRs/ADR-004-toml-config-versioning.md
docs/RAG/ADRs/ADR-003a-CLI-refactor-planning.md
docs/RAG/ADRs/ADR-002-Visual-Mode-planning 1.md
docs/RAG/ADR-AI/ADR-AI-001-three-layer-cache.md
docs/RAG/ADRs/ADR-010-the-LUA-escape-hatch.md
docs/RAG/ADRs/ADR-002-Visual-Mode-planning.md
docs/RAG/ADRs/ADR-011-text-parsing-stack.md
docs/RAG/ADRs/ADR-005-MCP-server-wrapper.md
docs/RAG/ADRs/ADR-003c-v1.2-CLI-commands.md
docs/RAG/ADRs/ADR-003c-v1.1-CLI-commands.md
docs/RAG/ADRs/ADR-012-create-man-pages.md
docs/RAG/ADRs/ADR-003b-v1-CLI-commands.md
docs/RAG/ADRs/ADR-003d-CLIO-appendix.md
docs/RAG/ADRs/ADR-009-GTD-ideation.md
docs/RAG/ADRs/ADR-001-cli-rag.toml.md
docs/RAG/ADRs/ADR-014-RAG-rethink.md
docs/RAG/ADRs/ADR-008-ai-rag.toml.md

26/126--------------------------------------------------------------------------------------------
> ADR-
===========================================================================  
RETURN: Select | SPACE+A: AgendaView | SPACE+G: GraphView | Space+E: Editor 
```

### GraphView

A core navigation view based on exporting a graphviz ascii view. This is based on the graph cli command. 

The individual notes should be tagged with some kind of leader so you can jump into notes via keyboard shortcut eg the alphabet. 


```bash
                                            ┌─────────────┐
                                            │ J:AI-IMP-03 │
                                            └─────────────┘
                                              ▲
                                              │
                                              │
                       ┌──────────────┐     ┌───────────────────────┐     ┌─────────────┐
                       │ H:AI-IMP-01  │ ◀── │       A:ADR-01        │ ──▶ │ I:AI-IMP-02 │
                       └──────────────┘     └───────────────────────┘     └─────────────┘
                         │                    │              │
                         │                    │              │
                         ▼                    ▼              │
     ┌───────────┐     ┌──────────────┐     ┌─────────────┐  │
     │ E:IMP-01  │ ◀┐  │ I: AI-IMP-02 │  ┌─ │  B:ADR-02   │ ─┼────┐
     └───────────┘  │  └──────────────┘  │  └─────────────┘  │    │
       │            │    │               │    │              │    │
       │            │    │               │    │              │    │
       ▼            │    ▼               │    ▼              │    │
     ┌───────────┐  │  ┌──────────────┐  │  ┌─────────────┐  │    │
  ┌▶ │ F:IMP-05  │  │  │ J: AI-IMP-03 │  │  │  C:ADR-04   │  │    │
  │  └───────────┘  │  └──────────────┘  │  └─────────────┘  │    │
  │    │            │                    │    │              │    │
  │    │            └────────────────────┘    │              │    │
  │    ▼                                      ▼              │    │
  │  ┌───────────┐                          ┌─────────────┐  │    │
  │  │ G: IMP-12 │                          │  D:ADR-07   │  │    │
  │  └───────────┘                          └─────────────┘  │    │
  │                                           │              │    │
  │                                           │              │    │
  │                                           ▼              │    │
  │                                         ┌─────────────┐  │    │
  │                                         │  G:IMP-12   │ ◀┘    │
  │                                         └─────────────┘       │
  │                                                               │
  └───────────────────────────────────────────────────────────────┘
===========================================================================
RETURN: Select | SPACE+A: AgendaView | SPACE+F: FuzzyView | Space+E: Editor 
```

### EditView 

Selecting a not via the graph, agenda or fuzzyview all result in opening a note in the Editor. This is just an editor by default but you can open the local graph overlay in a second buffer with a leader key. 

The editor has "simple" LSP functions. e.g.
1. Invalid wikilinks and node_ID's result in syntax highlights
2. tab completion for wikilinks/tags/ids etc 
3. re-naming notes across the base when a tag/link/title etc is updated.  
   
The leader key can open the local graph and this is updated live as you edit the note and create new connections. 
   
```
---							               |	                    ┌───────────┐     ┌──────────────────┐     ┌───────────┐
id: ADR-01								   |	                    │ AI-IMP-01 │ ◀── │                  │ ──▶ │ AI-IMP-07 │
tags:							           |	                    └───────────┘     │                  │     └───────────┘
  - Toml								   |	                      │               │                  │
status: in-progress				 	       |	                      │               │      ADR-01      │
depends_on: none						   |	                      ▼               │                  │
created_date: {{date}}					   |	                    ┌───────────┐     │                  │     ┌───────────┐
last_modified: {{date}}		  		       |	                    │ AI-IMP-02 │ ◀── │                  │ ──▶ │  IMP-06   │
related_files: []						   |	                    └───────────┘     └──────────────────┘     └───────────┘
										   |	                      │                 │         │
# cli-rag.toml	       			  	       |	                      │                 │         │
										   |	                      ▼                 ▼         │
## Objective							   |	     ┌────────┐     ┌───────────┐     ┌────────┐  │
At the time of writing...				   |	     │ IMP-01 │ ◀┐  │ AI-IMP-03 │  ┌─ │ ADR-02 │ ─┼────┐
								           | 	     └────────┘  │  └───────────┘  │  └────────┘  │    │
## Context								   |	       │         │                 │    │         │    │
if we see issues we can...				   |	       │         └─────────────────┘    │         │    │
										   |	       ▼                                ▼         │    │
## Decision					      	       |	     ┌────────┐                       ┌────────┐  │    │
Tentatively adopt the above...		       |	  ┌▶ │ IMP-05 │ ◀──────────────────── │ ADR-04 │  │    │
						        		   |	  │  └────────┘                       └────────┘  │    │
## Consequences			   		           |	  │    │                                │         │    │
A new implementation ticket will be...	   |	  │    │                                │         │    │
										   |	  │    │                                ▼         │    │
## Updates								   |	  │    │                              ┌────────┐  │    │
								    	   |	  │    │                              │ ADR-07 │  │    │
										   |	  │    │                              └────────┘  │    │
										   |	  │    │                                │         │    │
										   | 	  │    │                                │         │    │
										   |	  │    │                                ▼         │    │
										   |	  │    │                              ┌────────┐  │    │
										   | 	  │    └────────────────────────────▶ │ IMP-12 │ ◀┘    │
										   |	  │                                   └────────┘       │
										   |	  │                                                    │
										   |	  └────────────────────────────────────────────────────┘
===================================================================================================== 
RETURN: Select | SPACE+A: AgendaView | SPACE+F: FuzzyView | SPACE+G: GraphView | CTRL+G: GraphOverlay 
```

## Decision
<!-- What is the change that we're proposing and/or doing? -->


## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

## Updates
<!-- Changes that happened when the rubber met the road -->

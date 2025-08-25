---
id: ADR-002
tags:
  - TUI
  - NeoVIM
  - rataTUI
status: proposed
depends_on:
  - ADR-01
  - ADR-003
created_date: 2025-08-24
last_modified: 2025-08-24
related_files: []
---

# CLI-RAG Workflow Planning

## TUI (Initial Concept)

1. The user opens a new repo and types `cli-rag init` which checks for a .cli-rag.toml and if not present opens their editor with a marked up config file. They can either save this or exit the process. 
   
   See [[ADR-001-cli-rag-toml]] for a full example. 

2. Once the .cli-rag.toml is created it opens the TUI "master control" screen e.g. Magit/org style tab collapsible lists. This expands as tracked notes are added. 
   
```bash
v **Templates** 
	1 - IMP-*  
	2 - ADR-*
	3 - ADR-DB-*
		
v **Tracked Notes** 
	- ADR-001-my-first-plan
	- ADR-002-my-happy-place
	- ADR-003-oh-god-oh-fuck
	- ADR-004-panic-refactor
	* ADR-DB-001-event-store
	* ADR-DB-002-idempotency
	+ IMP-001-database-spike
	+ IMP-002-websocket-spike  
	...	        
TAB: Collapse/Expand | RETURN: Select | SPACE+F: FuzzyFind | G: GraphView
```

3.  The user can begin creating notes populated with expected (empty) frontmatter and template. Selecting a note or a template opens the editor for the user. When the close the editor the TUI catches the exit code and rebuilds the index and adds the new tracked notes to the master control screen. 

4. Ideally we would have a simple fuzzy finder that is accessible in all windows with the same keystrokes. It only indexes tracked notes. This and the master control screen let you fly around the knowledge base. 
   
5. GraphView the leans on graphviz dot view's ability to render out ascii. Could use the [[ADR-DRAFT-simple-query-dsl]] and let the user navigate the graph by pulling different clusters. I'm not sure in the real world if it would feel super useful but the lift should be low and it's fun and worth trying. 
   
## Implementation Details 

- RataTUI is the most likely back-end here. 
- The `watch` command is active while the TUI is running
   
## The NeoVim Advantage

All of the above can be much smoother in neovim. In theory we could 

- Define a consistent naviation UI using a leader key. 
- Lean on Existing fuzzy finding and implement less ourselves. 
- Offer a first class editing experience. We could in theory parse our index and have tree-sitter powered live linting of valid id's and note names
- make [[links]] and ID: ADR-005 directly navigable from the editor. 

This is ultimately inching much closer to a programmer friendly obsidian that lives in a repo without fuss. I'm not sure if that's a good or a bad thing. It would without a doubt be the most comfortable and fully featured version. 
  
However I do think asking people to configure a full neovim environment is a notably higher ask. 

---
id: ADR-002
tags:
  - TUI
  - NeoVIM
  - rataTUI
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


## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

### TUI/NeoVim Workflow Planning

1. The user opens a new repo and types `cli-rag init` which checks for a .cli-rag.toml and if not present opens their editor with a marked up config file. They can either save this or exit the process. 
   
   See [[ADR-001-cli-rag.toml]] for a full example. 

2. Once the .cli-rag.toml is created it opens the TUI "master control" screen e.g. Magit/org style tab collapsible lists. This expands as tracked notes are added. 
   
  ```bash 
# Agenda
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
	  L IMP-004
	  L IMP-002 
	> Completed
	> Cancelled 

# Vault 
v **Templates** 
	1 - IMP-*  
	2 - ADR-*
	3 - ADR-AI-*
v **ADR** 
	L ADR-001-my-first-plan
	L ADR-002-my-happy-place
> **ADR-AI** 
v **IMP** 
	L IMP-001-database-spike
	L IMP-002-websocket-spike  
       
TAB: Collapse/Expand | RETURN: Select | SPACE+F: FuzzyFind | G: GraphView
```

- This may not be possible in neovim vs emacs but the ideal is being able unfold the note directly into an editable minibuffer or hit enter to open the full note.  

3.  The user can begin creating notes populated with expected (empty) frontmatter and template. Selecting a note or a template opens the editor for the user. When the close the editor the TUI catches the exit code and rebuilds the index and adds the new tracked notes to the master control screen. 

4. Ideally we would have a simple fuzzy finder that is accessible in all windows with the same keystrokes. It only indexes tracked notes. This and the master control screen let you fly around the knowledge base. 
   
5. GraphView the leans on graphviz dot view's ability to render out ascii. Could use the search command and let the user navigate the graph by pulling different clusters 
   
```
   An idea that keeps coming to mind is using the local `--include-bidirectional` graph as a navigation system in the TUI/NVIM. 

[Notably, graphiz directly supports ascii output.](https://graphviz.org/docs/outputs/ascii/) An imaginable workflow is we pull a local graph and append the ID's so it's 

1. ADR-001 
2. ADR-002 
   
You are then shown a screen like this where you can with a single key press to fly around notes e.g. 


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


   
### Implementation Details 

- RataTUI or neovim is the most likely front-end here. 
- The `watch` command is active while the TUI is running unless toggled off via a .toml flag. 
- the TUI fuzzy finder is not a full telescope redo --it's just a magit style wrapper of `search --query `
   
### The NeoVim Advantage

All of the above can be much smoother in neovim. In theory we could 

- Define a consistent naviation UI using a leader key. 
- Lean on Existing fuzzy finding and implement less ourselves. 
- Offer a first class editing experience. We could in theory parse our index and have tree-sitter powered live linting of valid id's and note names
- make `[[links]]` and ID: ADR-005 directly navigable from the editor. 

This is ultimately inching much closer to a AI Co-Programmer friendly obsidian that lives in a repo without fuss. I'm not sure if that's a good or a bad thing. It would without a doubt be the most comfortable and fully featured version. 

## Decision
<!-- What is the change that we're proposing and/or doing? -->

Prioritize neovim. While the TUI is nice and more universal the neovim community is the space that makes sense for a lua configured programmers tool. its likely to be 

1. less work. 
2. more powerful. 

Also i want it to be fennel ))))))))))))))))))))))))))))))))))

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

plan to adopt `nvim-oxi` and ensure our patterns do not contradict this future goal. It may be v1 or v1.1 but priority is fairly high. 

## Updates
<!-- Changes that happened when the rubber met the road -->
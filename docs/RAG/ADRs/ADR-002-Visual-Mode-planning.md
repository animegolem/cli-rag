---
id: ADR-002
tags:
  - TUI
  - NeoVIM
  - rataTUI
  - Emacs
status: planning
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
1. Presents current TODO's on tracked notes. These are created anywhere in a tracked note e.g. [{TODO}: Note that displays as a reminder]. A date inside a TODO in UTC formatting will be tracked on the agenda screen. 
2. Presents any upcoming or past due due-dates. 
3. Lists projects by status if the kanban_status and kanban_statusline front matter are active. 
4. Lists tracked notes and templates. 

The - [] todo items are live. see the `Thoughts about GTD shapes` section for further clarity. 

The date it's marked done is tracked in the index and it falls off the agenda screen either 24 hours later or per user setting in the toml.  
   
  ```bash 
V ToDo ! 
	-[] TODO@10: Critical parser bug | Note:[[IMP-005]] 
	    -[] implement feature in code 
	    -[] create unit tests 
	-[] TODO@MED:  Refactor validation  | Note:[[IMP-005]] 
	-[] TODO: Default priority task | Note:[[IMP-003]] 
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
---							           |	                    ┌───────────┐     ┌──────────────────┐     ┌───────────┐
id: ADR-01								   |	                    │ AI-IMP-01 │ ◀── │                  │ ──▶ │ AI-IMP-07 │
tags:							           |	                    └───────────┘     │                  │     └───────────┘
  - Toml								   |	                      │               │                  │
status: in-progress			 	       |	                      │               │      ADR-01      │
depends_on: none						   |	                      ▼               │                  │
created_date: {{date}}					   |	                    ┌───────────┐     │                  │     ┌───────────┐
last_modified: {{date}}		  	       |	                    │ AI-IMP-02 │ ◀── │                  │ ──▶ │  IMP-06   │
related_files: []						   |	                    └───────────┘     └──────────────────┘     └───────────┘
---		 							   |	                      │                 │         │
# cli-rag.toml	       			       |	                      │                 │         │
										   |	                      ▼                 ▼         │
## Objective							   |	     ┌────────┐     ┌───────────┐     ┌────────┐  │
At the time of writing...				   |	     │ IMP-01 │ ◀┐  │ AI-IMP-03 │  ┌─ │ ADR-02 │ ─┼────┐
								           | 	     └────────┘  │  └───────────┘  │  └────────┘  │    │
## Context								   |	       │         │                 │    │         │    │
if we see issues we can...				   |	       │         └─────────────────┘    │         │    │
										   |	       ▼                                ▼         │    │
## Decision					           |	     ┌────────┐                       ┌────────┐  │    │
Tentatively adopt the above...		       |	  ┌▶ │ IMP-05 │ ◀──────────────────── │ ADR-04 │  │    │
						        		   |	  │  └────────┘                       └────────┘  │    │
## Consequences			   		   |	  │    │                                │         │    │
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


## Notes 

### Thoughts about GTD shapes 

#### Approach 1 

The Current concept is a simple [bounded] box with : as a divider. Commands are in a [{curly brace}, {comma separated}: any text that should show up with the reminder goes here.] 

The concept was it then shows up as a `- []` that can be filled on the agenda screen. In the event it's marked out the note would be updated like so;

~~[{TODO@10}, {DATE}: any text that should show up with the reminder goes here.]~~ 

1-10 being intensity. 
or 'high, medium, low'

You could imagine an expanded multi-line version eg:

```
[{TODO@10}: Implement the big new feature 
- [ ] refactor the code 
- [ ] implement the thing 
- [ ] lots of tests
]
```

This is however, not actually for most parsers valid markdown. 

#### Approach 2

Another approach is just having heading flags 

```
## Implement the new big feature 10 [@TODO:10]
- [ ] refactor the code 
- [ ] implement the thing 
- [ ] lots of tests
```

This is dramatically less clear to parse but is arguably the most idiomatic markdown approach. It in theory does not really complicate the prior tagging and can support single item notes. when complete ~~[TODO:10]~~

I dont know how ocd the average person is but im very specific in my headings and don't love adding extra ones in. But this is also the system most suited to "upgrading" an existing note. 

#### Approach 3 

The nuclear option if i'm going to be in emacs anyway is directly honor org syntax. I don't feel great about this option but it's /arguably/ the most standards compliant path in a very strange and very arguably sense. i

The biggest single advantage of org's syntax is i've pinned a very narrow world of "commands" with defined logic where org is a much looser world where this is a lisp hook directly; 

```lisp
(setq org-todo-keywords
      '((sequence "TODO" "FEEDBACK" "VERIFY" "|" "DONE" "DELEGATED")))
```

This could be handled with out a huge lift in the toml or lua configs but the question I think that need be asked is "what if anything does this get me that my style doesn't?"

The raw tracking seems to mostly come down to the narrow extra nuance. This would be a very direct and simple LSP action to update. Potentially given it's a 1:1 swap even simpler than the "object" style of approach 1. These are not automatically mutually exclusive. 

The ease in swapping is traded for the lowered clarity around the question of "what content is part of the note"

### GTD Synthisis 

#### The LUA Escape Hatch 

Given we plan to allow full configs in lua it's likely best to give more open hooks that let you tie into a date system, a todo system a kanban system and let it be defined more freely in lua eg;

```lua
  M.gtd = {
    todo_parser = function(line)
      -- Parse your preferred syntax
      local priority, text = line:match("%[{TODO@(%w+)}: (.+)%]")
      return { priority = priority, text = text }
    end,

    todo_renderer = function(todo)
      -- Render back to your preferred format
      return string.format("[{TODO@%s}: %s]", todo.priority, todo.text)
    end
  }
```

```lua
M.gtd = {
  todo_parser = function(line)
    -- Custom syntax: e.g., parse "**TODO high: Fix bug**" 
    local cmd, priority, text = line:match("%*%*([A-Z]+) (%w+): (.+)%*%*")
    if cmd then
      return { cmd = cmd, priority = priority, text = text }
    end
  end,
  todo_renderer = function(todo)
    -- Render with custom formatting
    return string.format("**%s %s: %s**", todo.cmd or "TODO", todo.priority or "medium", todo.text)
  end,
  -- Bonus: Add a completion hook for AI/agenda updates
  on_complete = function(todo)
    todo.completed_at = os.date("!%Y-%m-%dT%H:%M:%SZ")  -- UTC timestamp
    return todo
  end
}
```

This reduces the load on the toml to simply providing a sensible default. 

#### Inline GTD Headings 

There are two core advantages to this approach
  1. Clean upgrades to existing notes particularly given the [[AI-IMP-* v3]] notes are based around checklists this allows any section to be made a high, tracked priority on the main screen. eg [[AI-IMP-002-graph-path-contracts-alignment {COMPLETE}]]  
  2. least heavy syntax + most like idiomatic markdown. 

**GTD sketch**
Shape: `[@CMD:attr1=value,attr2=value] Optional Text`
- Core behavior includes 
	- Headings
	- Optional Text
	- any `-[]` checklist style items within the heading. Follows until the next heading of the same level. Does not read into sub-headings. Any generic/non-formatted text is ignored. In lua this behavior should be configurable.  
- Attach to headings (e.g., `## Fix Bug [@TODO:rank=high,due=2025-09-01]`) or standalone lines.
- Sub-tasks use standard Markdown checkists below.
- Completion: Strike the flag `~~[@TODO:... ]~~` or toggle sub-checkboxes. Toggling the top level to `-[x]` toggles all children.

At present the only planned top level command is `[@TODO]`. The only planned `:commands` are;

- :due=YYYY-MM-DD 
- :rank=[1,100] or [LOW,MED,HIGH]

however this could easily be expanded as needed e.g.;  

- [@EVENT:time=14:00,location=zoom,due=2025-09-23] Team meeting
  
Importantly all of the following should be valid --commands are additive not required. 

- [@TODO] (defaults to medium priority)
- [@TODO:high] (shorthand for rank=high)
- [@TODO:12] (shorthand for rank=12)
- [@TODO:rank=high,due=2025-09-01] (100% verbose)
  
when a task is completed only it's command block and immediate line are crossed out. `-[]` are also updated dynamically but not crossed out. 

e.g.
~~[@TODO:100,due=2025-09-15] Clean your room and do your homework !!!!~~ 

A floating entry can have a checklist below it. It will read only until the first new line that is either 

1. empty
2. does not start with -[] or - [] 

e.g. 

[@TODO:10] Critical parser bug
  -[] run debugger
  -[] panic when i can't find it
  -[] beg ai for help
  
One of the last major questions is what shape should this take eg; 

```bash
V ToDo !
  * IMP-005 # to allow multiple TODO per note 
    -[] [@TODO:10] Critical parser bug | 
        -[] run debugger
        -[] panic when i can't find it
        -[] beg ai for help
```


```
V ToDo !
  V Critical parser bug, check the blahbahaha and the blohoohoo. Update the toodledee. [IMP-005] | [@TODO:10] | [DUE:TODAY]
	-[] run debugger
	-[] panic when i can't find it
	-[] beg ai for help
```

```
V ToDo !
  V -[] [IMP-005 | @TODO | RANK:HIGH | DUE:TODAY | SUB-TASKS:3] | Critical parser bug, check... 
        V Critical parser bug, check the blahbahaha and the blohoohoo. Update the toodledee. We need to ensure the unit tests all pass.
	      -[x] run debugger and validate bahahaha is crashing. 
 	      -[] panic when i can't find why bloohohoo is threadlocking. 
	      -[] beg ai for help. cry. 
```

Probably the third is the most clear overall. but the overall wordiness on the top line has become too much. the length of the comment line we don't control. mine would be short but the user could do anything like above or even much longer. 

we should test if we crash on overlong strings when we get there. 

a possible final path is doing it more keyed to give a much more compressed dataline eg 

```bash
 V TODO! 
   > [IMP-005|@TODO|H|5D|1/3] critical parser bug...
     V Critical parser bug, check the blahbahaha and the blohoohoo. Update the toodledee. We need to ensure the unit tests all pass.
	   -[x] run debugger and validate bahahaha is crashing. 
 	   -[] panic when i cant find why bloohohoo is threadlocking. 
	   -[] beg ai for help. cry. 
```

`key = [node_id|command|rank|cays till due. (can show negatives)|sub-tasks] first three words... `

I finally feel decent about that one overall. 

A new .toml toggle and lua hook will need to be created to allow tuning of the "garbage collection" for the agenda screen e.g. how long after toggling something off do you have before it's gone and the note updated on disk. 

The overall vision is the 'normal' user just treats it as linear-in-a-repo-with-ai as project management tool but then someone that wants to really configure doesn't fight any of the and just uses big hooks. 


### Thoughts about form factors

#### TUI

#### .nvim 

#10

Meaningfully I am typing this in emacs right now. It and obsidian are what i actually use. 

## Decision
<!-- What is the change that we're proposing and/or doing? -->


## Consequences
<!-- What is easier or more difficult to do because of this change? -->

## Updates
<!-- Changes that happened when the rubber met the road -->

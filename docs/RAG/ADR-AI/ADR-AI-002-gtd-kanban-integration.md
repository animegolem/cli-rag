---
id: ADR-AI-002
tags:
  - gtd
  - kanban
  - task-management
  - neovim
  - tui
  - inline-todos
status: accepted
depends_on: 
  - ADR-001
  - ADR-002
  - ADR-009
  - ADR-AI-001
created_date: 2025-08-30
last_modified: 2025-08-30
related_files: [src/commands/gtd.rs, src/tui/agenda.rs]
---

# ADR-AI-002-gtd-kanban-integration

## Objective
<!-- A concise statement explaining the goal of this decision. -->
Design a unified GTD/Kanban system that provides a command center view in Neovim while maintaining portability and simplicity in the underlying note format.

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->
Users need a centralized view of tasks, deadlines, and project status without leaving their note-taking environment. The vision is a Magit/org-agenda style interface showing:

- Upcoming deadlines and due items
- Inline TODOs extracted from note content  
- Kanban board for workflow visualization
- Collapsible project/schema groupings

Key requirements:
- Must parse inline TODO syntax: `TODO@HIGH: -[] implement feature`
- Support kanban workflow states without complex graph relationships
- Show completed items for configurable time period
- Integrate with LSP for checkbox state updates
- Maintain Obsidian compatibility

### Visual Mockup (human made)

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
	> **Backlog**
	V **In-Progress**
	    L IMP-002 [Introduce config_version with deprecation warnings and an upgrade helper.] # defined by kanban_status and kandban_stausline frontmatter 
	V **Completed**
	    L IMP-004 [Completed — unified initial error codes; multi-schema E200; per-schema cycle policy; JSON/NDJSON codes + locations.]
	> **Cancelled** 

# Bases
v **Templates** 
	1 - IMP-*  
	2 - ADR-*
	3 - ADR-AI-*
v **ADR** 
	- ADR-001-my-first-plan
	- ADR-002-my-happy-place
> **ADR-AI** 
v **IMP** 
	+ IMP-001-database-spike
	+ IMP-002-websocket-spike  
       
TAB: Collapse/Expand | RETURN: Select | SPACE+F: FuzzyFind | G: GraphView
  ```

## Decision
<!-- What is the change that we're proposing and/or doing? -->
Implement GTD through frontmatter fields and inline TODO parsing:

**Frontmatter extensions:**
```yaml
kanban_status: in_progress  # backlog|todo|in_progress|blocked|done|archived
due_date: 2025-09-01
kanban_statusline: "Optional human-readable status"
```

**Inline TODO syntax:**
```markdown
TODO@HIGH: -[] Critical bug fix
TODO@MED: -[] Refactor validation  
TODO: -[] Default priority task
```

**Display logic:**
- Completed items visible for 3 days by default
- TODOs colored by priority in Neovim (HIGH=red, MED=yellow, LOW=green)
- Kanban cards show inline TODOs from their linked notes
- Use cache layer for tracking completion timestamps without modifying notes
- The kanban_statusline is included on the agenda page if filled. 
  
### HumanNOTE(S): When displaying in the GUI we can hydrate with metadata from the index, what note was it tagged from, when was the tag added eg 
	`- TODO@HIGH: -[] Critical parser bug | Note:[[IMP-005]] | Created: 2025-09-01`
    - The kanban_statusline is included on the agenda page if filled. The most idea is maybe it's cut off at X characters but scrolls on hoverover. Not sure the lift. 
    - The actual human typed entry could be like {TODO@MED} for max clarity. the we reuse that syntax as needed. 
    - A more verbose and specific syntax [{TODO@HIGH}: the note lives within the brackets.]
    - When the object is marked with the -[x] on the agenda screen the line in the document gets updated like ~~[{TODO@HIGH}: the note lives within the brackets.]~~ nothing is deleted just marked off. 
  
## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->
**Benefits:**
- Simple, scannable TODO syntax that works in any markdown viewer
- Kanban workflow without graph complexity
- Command center view achievable with existing frontmatter
- LSP can update checkboxes without touching frontmatter

**Tradeoffs:**
- True task dependencies require manual tracking
- No automatic completion percentage from subtasks
- Priority is encoded in TODO syntax, not frontmatter

## Updates
<!-- Changes that happened when the rubber met the road -->
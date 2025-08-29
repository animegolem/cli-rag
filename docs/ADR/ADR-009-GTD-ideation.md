---
id: ADR-009
tags:
  - CLI
  - org-agenda
  - todo
  - tasklist
  - GTD
  - NeoVIM
status: pending
depends_on: 
  - ADR-002
  - ADR-001
  - ADR-004
  - ADR-008
  - ADR-010
created_date: 2025-08-25
last_modified: 2025-08-25
related_files: [.cli-rag.toml]
---

# GTD-ideation

## Objective
<!-- A concise statement explaining the goal of this decision. -->

Goal: The main TUI screen shows a agenda somehow created across notes in front matter maybe even a simple "due now" or a data entry flag 

This could be structured as a `managed_frontmatter` with a check against a due date field maybe?. It's logical but I don't honestly work that way. I more would want to be able to flag things in notes and somehow have that create a todolist with priority buckets. 

Creating an interactive kanban in the TUI that edits the front matter as you make changes could in theory is possible. write changes to the index and then flush out to the file on exiting the Major Mode/TUI.  

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

Straightforward project tool that allows setting a tracked priority field. This should be functionally a validator on a schema that allows you to track any tag for date/etc and add it to GTD. 

Where this could shift is the question "what if the AI could set it's own org agenda?"

> `cli-rag --ToDO`
> 
> 1. TASK-005
> > - one line context note    
> 2. TASK-002
> > - one line context note    
> 3. TASK-001
> - one line context note    

Something like this is the path where it becomes a more key feature. 

Might work something like this eg

**1. The Trigger: `gtd` in `managed_frontmatter`**

Defines for cli-rag that it should monitor gtd: for true/false 

```toml
# In an "ADR" schema
[schema.frontmatter]
    custom_frontmatter = ["id", "title", "status"]
    # By placing 'gtd' here, we declare it as a key with special logic.
    # The system will now look for a corresponding [schema.gtd] block.
    managed_frontmatter = ["depends_on", "gtd"]
```

**2. The Configuration Block: `[schema.gtd]`**

Because `gtd` is a `managed_frontmatter` key the following rules are applied to any note flagged `true`

```toml
# =============================================================================
#                        # --- GTD VALIDATED BEHAVIOR --- #
# =============================================================================
# This block is active because 'gtd' is in the validated_frontmatter list.
# It defines the logic for the 'gtd' key itself and the behavior it enables.
[schema.gtd]
    # This check applies to the 'gtd' key itself.
    # It ensures the key's value is valid for activating/deactivating the task status.
    # So, GTD: "maybe" would be an error.
    legal_entry = [true, false, "[ ]", "[x]"]
    severity = "error"
    
    # This flag tells the system that if the 'gtd' key evaluates to true,
    # then this note should be added to the dedicated GTD index.
    index_on_activate = true

    # These are the additional frontmatter fields that become REQUIRED
    # only when the 'gtd' key is present and evaluates to an active state.
    required_fields_on_activate = ["priority", "status"]

    # Default values to inject when a note is promoted or created as a task.
    [schema.gtd.default_values]
        status = "pending"
        priority = 50
        assignee = "unassigned"

    # Specific checks for the GTD-related fields. These ONLY run
    # if the 'gtd' key is present and active.
    [[schema.gtd.check]]
        type = "frontmatter_fields"
        severity = "error"
        [schema.gtd.check.fields]
            [schema.gtd.check.fields.status]
                legal_entry = ["pending", "active", "blocked", "completed", "deferred"]
            [schema.gtd.check.fields.priority]
                legal_entry = "integer"
                range = [0, 100]
```

then the operation of GTD tracking itself is written to something like json,toml,yaml the idea being the AI managing it's rag can overwrite the priority list directly + tracking additional special logic. we'll see.  

## Decision
<!-- What is the change that we're proposing and/or doing? -->

Most likely defer for v1 and when TUI planning picks consider doing a basic due date version and then start working the implement write features so a checkbox can mark an item done etc little interlinking. it's got to be a slow process and it will be tied to a yet undecided ui. 

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

No direct impact on the core function but a baseline for extending the tool as it gains a gui. It's a polish and not core to 1.0. 

## Updates
<!-- Changes that happened when the rubber met the road -->
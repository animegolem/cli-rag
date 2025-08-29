---
id: ADR-008
tags:
  - AI
  - LLM
  - RAG
  - agentic
  - automation
status: draft
depends_on:
  - ADR-001
created_date: 2025-08-27
last_modified: 2025-08-27
related_files: []
---

# ADR-008-ai-mem.toml

## Objective
<!-- A concise statement explaining the goal of this decision. -->

This tool is designed to serve as a RAG for an AI. The most extreme version of this idea is "what if we make the AI itself write out to a zettelkasten, that it can search, and it's schema's force it to interlink"

This is a somewhat experimental idea to dogfood. In general this should not at first require any new code. This note is a tracker for findings, needs and additional requirements as we find if this idea is workable. 

The some what unclear space is if we should use our own logic for document linking or lean on something like markdown oxide as an LSP. It may make sense to keep our most core graph edges fully locally processed. 

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

```toml
    
# A schema for notes the AI creates and manages for itself.
# The human user does not touch these.
[[schema]]
name = "AI-MEM" 

# =============================================================================
#                           # --- DISCOVERY --- #
# =============================================================================
file_patterns = ["AIM-*.md"]

# =============================================================================
#                    # --- TEMPLATES & GENERATION --- #
# =============================================================================
[schema.new]
    # UUID is perfect for machine-generated, non-sequential IDs.
    id_generator = { strategy = "uuid", prefix = "AIM-" }
    filename_template = "{{id}}-{{title|kebab-case}}.md"

    # The template is just a bare structure for the AI to fill.
    [schema.new.template.note]
        template = """
        ((frontmatter))

        # {{title}}
		
        ## Summary
        {{LOC|100}}

        ## Connections & Rationale
        <!-- Why was this note created and what does it link to? -->
        {{LOC|200}}
        """

# =============================================================================
#                    # --- FRONTMATTER VALIDATION --- #
# =============================================================================
[schema.frontmatter]
    # These are the fields the AI MUST provide.
    custom_frontmatter = ["id", "title", "keywords", "confidence_score", "source_id"]
    unknown_frontmatter_policy = "missing_only" 
    managed_frontmatter = ["depends_on", "created_date"]

# =============================================================================
#                          # --- VALIDATION --- #
# =============================================================================
[schema.rules]
    [schema.rules.frontmatter]
        [schema.rules.frontmatter.confidence_score]
            # Make sure it's a valid float between 0.0 and 1.0
            legal_entry = "^(0(\\.\\d+)?|1(\\.0+)?)$"
            severity = "error"

    [schema.rules.body]
        [schema.rules.body.wikilinks]
            # CRITICAL: Every AI note must link to at least one other note.
            # This forces the graph to be connected.
            min_links = 1
            severity = "error"
```


### Possible Future Requirements (scratchpad) 

```gemini 2.5 pro
**1. A More Powerful Query Language:**
*   **The Need:** The simple `search --query <substr>` will not be enough. The AI will need to ask much more sophisticated questions of its own memory, such as:
    *   "Find all memories with `confidence_score > 0.8` that are connected to `ADR-005`."
    *   "Find all memories created in the last 24 hours with the keyword `database`."
*   **The Implication:** This breathes new life into the deferred `ADR-DRAFT-simple-query-dsl`. A rich, metadata-aware query engine will become a core requirement.

**2. Specialized Agentic Commands:**
*   **The Need:** While a wrapper script can orchestrate the AI, you will eventually want to build first-class commands to support this workflow.
*   **The Implication:** This could lead to a new suite of commands like:
    *   `cli-rag ai create-memory --prompt "Summarize this..."`: A command that takes a prompt, gets the structured JSON from the LLM, and uses the `note_template` to write the file.
    *   `cli-rag ai find-related --id AIM-123e45... --depth 2`: A specialized retrieval command optimized for the AI's own graph.

**3. Advanced Graph Analysis:**
*   **The Need:** Once the AI has created thousands of notes, the human user will need tools to understand the structure of the AI's "mind."
*   **The Implication:** `doctor` and `graph` will need to evolve. You might add flags like:
    *   `doctor --find-orphan-ai-notes`: Find AI memories that have no incoming links from human notes.
    *   `graph --source-of ADR-005`: Generate a graph of all AI memories that were derived from a specific human-written ADR.
```

## Decision
<!-- What is the change that we're proposing and/or doing? -->

Develop a set of best practices and ship with the note as the core example note in the config file. 


## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

Change in mental approach post implementation impacts will be observed and determined. 

## Updates
<!-- Changes that happened when the rubber met the road -->

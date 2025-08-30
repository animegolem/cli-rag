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

`have ai genrerate it's own dogfood here when the time is right`

stuff like so 
```toml
template = """
# Instructions
1. Extract the NPC name from the user's request
2. Identify 2-3 existing notes that would logically connect
3. Generate a confidence_score based on:
   - Explicit user details: 0.8-1.0
   - Inferred from context: 0.5-0.7
   - Speculative additions: 0.2-0.4
"""
```

### Possible Future Requirements (scratchpad) 

Nothing thought trough this is just a place to store conversation snips

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

complaints from opus but i'm somewhat less concerned given the vibe is a bit more "experiment" but for best results this is all likely true. I just want to try the other way for fun. 

```
## The Deeper Issue: Bidirectional Trust

If the AI can write notes that influence its own future retrievals, you're creating a feedback loop. You need:

1. **Immutable human notes**: AI shouldn't edit human-authored content
2. **Trust scoring**: Weight human notes higher in retrieval
3. **Audit trail**: Track which agent (human/AI) created/modified each note

This might look like:


[schema.authorship]
immutable_after_creation = true  # For human schemas
author_type = "human"  # or "ai"
trust_weight = 1.0  # Human notes weighted higher in search
```

```fair
### 2. Circular Dependency Detection

With `depends_on` validation, you'll need cycle detection in your graph traversal. The config doesn't specify how to handle this.

## Critical Design Gap: Memory Lifecycle

If the AI creates thousands of AIM-* notes, you need lifecycle management:

1. **Decay/Archival**: Old memories with low confidence scores that haven't been accessed should fade
2. **Consolidation**: Multiple related memories should potentially merge
3. **Versioning**: When the AI updates its understanding, does it edit or create new notes?

Consider adding:


[schema.lifecycle]
archive_after_days = 90
min_confidence_to_persist = 0.3
consolidation_strategy = "merge_similar"
```

## Decision
<!-- What is the change that we're proposing and/or doing? -->

Develop a set of best practices and ship with the note as the core example note in the config file. 


## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

Change in mental approach post implementation impacts will be observed and determined. 

## Updates
<!-- Changes that happened when the rubber met the road -->

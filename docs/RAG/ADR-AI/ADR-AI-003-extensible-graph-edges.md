---
id: ADR-AI-003
tags:
  - graph
  - edges
  - frontmatter
  - validation
  - extensibility
status: accepted
depends_on: 
  - ADR-001
  - ADR-AI-001
  - ADR-AI-002
created_date: 2025-08-30
last_modified: 2025-08-30
related_files: [src/config.rs, src/validate.rs, src/graph.rs]
---

# ADR-AI-003-extensible-graph-edges

## Objective
<!-- A concise statement explaining the goal of this decision. -->
Create a unified abstraction for graph edge types that allows users to define custom relationships without code changes while maintaining consistent validation and traversal behavior across the system.

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->
The current system frontmatter approach requires manually defining each edge type (depends_on, blocked_by, etc.) with hardcoded validation logic. This creates several problems:

1. **Limited extensibility**: Adding new relationship types requires core code changes
2. **Inconsistent behavior**: Each edge type needs custom validation and graph traversal logic
3. **Use case constraints**: Different domains (ADR planning, RPG notes, project management) need different relationship vocabularies

Current multi-parent pattern works well:
```yaml
depends_on: [ADR-001, ADR-002, ADR-004]
```

But we need this pattern to work for arbitrary relationship types without hardcoding each one. The discussion showed the importance of flexible graph relationships for different use cases, while our own usage demonstrates that most relationships follow the same validation pattern: "ensure referenced nodes exist."

The system should work consistently whether someone defines `depends_on`, `blocked_by`, `character_knows`, or `implements` - they all follow the same node reference validation pattern.

## Decision
<!-- What is the change that we're proposing and/or doing? -->
Implement a three-tier frontmatter classification system that treats graph edges as a distinct, extensible category:

```toml
[schema.frontmatter]
# Explicitly defined system fields with special behavior
system_frontmatter = [
    "node_id",
    "created_date", 
    "last_modified",
    "groups",
    "kanban_status",
    "due_date"
]

# Graph edge fields - automatically validated as node references
graph_edges = [
    "depends_on",
    "blocked_by", 
    "implements",
    "supersedes",
    "parent_of"
    # Users can add more without changing core code
]

# Regular user fields
user_frontmatter = [
    "tags", 
    "status",
    "priority",
    "confidence_score"
]
```

**Automatic behaviors for graph_edges:**
- All values validated as existing node references
- Support both single values and arrays: `depends_on: ADR-001` or `depends_on: [ADR-001, ADR-002]`
- Included in graph traversal commands (`get --depth`, `graph`, `path`)
- Available for cycle detection and orphan analysis
- Indexed for fast lookup and reverse relationships

**Validation rules:**
```toml
[schema.validate.edges]
severity = "error"  # Default for all graph edges
validate_existence = true
allow_cross_schema = true
detect_cycles = true
```

Users can override per-edge:
```toml
[schema.validate.frontmatter.fields]
depends_on = { severity = "warning" }  # Override default
character_knows = { allow_cross_schema = false }  # Domain-specific rules
```

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->
**Benefits:**
- Users can define custom relationship types without core code changes
- Consistent validation behavior across all graph relationships  
- Single abstraction handles simple trees, complex DAGs, and domain-specific relationships
- Graph commands work automatically with new edge types
- Clear separation between temporal fields, relationships, and user metadata

**Tradeoffs:**
- Three-tier system adds conceptual complexity over simple system/user split
- Graph edge fields can't have arbitrary validation (all follow node reference pattern)
- Migration required for existing schemas using hardcoded edge validation

**Examples enabled:**
```yaml
# Project management
depends_on: [ADR-001]
blocked_by: [IMP-003]
implements: [SPEC-002]

# RPG worldbuilding  
character_knows: [NPC-Gandalf, NPC-Aragorn]
location_contains: [ITEM-Ring, NPC-Sauron]
quest_requires: [ITEM-Sword, SKILL-Stealth]
```

## Updates
<!-- Changes that happened when the rubber met the road -->

This content has been updated to be reflected in [[ADR-001-cli-rag.toml]]
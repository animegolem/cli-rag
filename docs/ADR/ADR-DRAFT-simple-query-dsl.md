---
id: ADR-
status: draft
depends: ADR-001
---

# advanced-query-dsl
  
Query Language

  Simple predicate-based queries would be powerful:
  Commands::Query {
      expr: String  // "status:accepted AND tags:security"
  }

I see this more likely as an MCP tool where an AI can just handle the complexity. 
---
id: ADR-AI-001
tags:
  - cache
  - performance
  - index
  - multi-process
  - state-management
status: accepted
depends_on: 
  - ADR-001
  - ADR-003c
created_date: 2025-08-30
last_modified: 2025-08-30
related_files: [src/index.rs, src/cache.rs, src/watch.rs]
---

# ADR-AI-001-three-layer-cache

## Objective
<!-- A concise statement explaining the goal of this decision. -->
Implement a three-layer cache architecture to enable fast access patterns, computed metadata, and session-based tracking without excessive disk writes or repository churn.

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->
The cli-rag system needs to track various types of metadata with different persistence requirements:

1. **Volatile session data** (access patterns, current TODO states) that shouldn't persist between sessions
2. **Computed metadata** (token counts, section ranges, graph traversals) that's expensive to calculate but can be rebuilt
3. **Source of truth** (note frontmatter) that must be preserved and portable

Current challenges:
- Tracking access frequency would cause excessive git churn if written to notes
- Token counting and summarization are expensive operations we don't want to repeat
- Multi-process access (CLI + Neovim + watch) creates potential for race conditions
- Graph traversals and semantic clustering are expensive to recalculate on every query

The PageIndex analysis revealed the value of treating the index as a cache of expensive computations rather than primary storage. This aligns with our principle that "what can be calculated from the graph should be."

## Decision
<!-- What is the change that we're proposing and/or doing? -->
Adopt a three-layer cache architecture with file-based locking for multi-process coordination:

**Layer 1 - Memory Cache (session-only)**
- Access patterns, active TODOs, current graph traversals
- Lost on process exit, never causes disk writes

**Layer 2 - Disk Cache (persistent but rebuildable)**  
- Token counts, section ranges, semantic clusters, computed graph metadata
- Written with debouncing (5 second intervals)
- Can be fully rebuilt from source via `validate --rebuild-cache`

**Layer 3 - Note Files (source of truth)**
- Only user-initiated changes (frontmatter, content)
- Portable, git-trackable, Obsidian-compatible

**Multi-process coordination via file locking:**
```rust
// Simple advisory locking
let lock_file = ".cli-rag/index.lock";
acquire_lock_with_timeout(lock_file, 100ms);
write_cache();
release_lock();
```

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->
**Benefits:**
- No repository churn from access tracking or computed metadata
- Fast retrieval of expensive computations
- Clean separation between user data and system metadata
- Simple crash recovery (cache is rebuildable)

**Tradeoffs:**
- Added complexity of cache invalidation logic
- Potential for stale reads in multi-process scenarios
- Small risk of data loss for in-flight session data on crash

## Updates
<!-- Changes that happened when the rubber met the road -->
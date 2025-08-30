---
id: ADR-010
tags:
  - LUA
  - config
  - configuration
  - template
  - validators
status: draft
depends_on: 
  - ADR-001
  - ADR-002
  - ADR-009
created_date: 2025-08-28
last_modified: 2025-08-28
related_files: [.cli-rag.toml]
---

# the-LUA-escape-hatch

## Objective
<!-- A concise statement explaining the goal of this decision. -->

The .toml has become increasingly complex as it's been iterated on in planning. Some of these features are not practical to support and the ability to grow is limited. A hard locked feature set also creates a natural v2 pressure. 

Instead we will implement a 'LUA Escape Hatch'. This will be possible in the `generation` and `validation` sections of the .toml `[[schema]]` letting custom logic be defined to validate against almost all aspects of the note.  

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

### Core Responsibility Split

- **Rust engine (+ winnow + Markdown parser):** parse once, build fast summaries (front-matter, headings, links, custom tokens, spans).
- **Lua hooks:** generate/validate by _querying those summaries_ and returning results. No disk, no net, no clock (unless you pass a deterministic one).

### Lifecycle 

1. **discover → load → parse**
    - Engine extracts:
        - `frontmatter`: YAML → map
        - `markdown`: headings, code fences, md links
        - `custom`: wikilinks, `{{filters}}`, `{{LOC|N}}`, etc. (via winnow/markdownoxide)
   
2. **generate (optional)**
    - `hooks.generate(ctx, api) -> { frontmatter?, body? }`

3. **validate**
    - `hooks.validate(note, api) -> { diagnostics[] }`

4. **persist/index**
    - Engine writes files, updates graph, caches. Lua never touches IO.
      
### Potential API Scope 

```gpt-5
## Read-only “data surfaces”

Minimal, predictable tables—not the whole AST.

- `ctx.schema` — `"ADR"` / `"NPC"` …
- `ctx.template` — the note template string (for generate).
- `note.frontmatter` — table (read-only snapshot).   
- `note.text` — full body text (string)
- `api.headings(level?) -> { {level, text, span} }`
- `api.links(kind?, options) -> { {kind="md|wikilink|image", target, title?, span} }`
- `api.code_fences(lang?) -> { {lang, info, text, span} }`
- `api.tokens(kind) -> { {kind, value, span} }`
    - e.g., `kind="loc_marker"` for `{{LOC|N}}`
    - e.g., `kind="filter"` for `{{title|kebab}}`
    - e.g., `kind="wikilink"` for `[[ADR-001#Section|alias]]`

- `api.graph(note_id, opts) -> { nodes[], edges[] }` (read-only: outgoing/incoming, depth)    
- `api.exists(id) -> bool`
- `api.resolve(id) -> { path, schema }?`

> **Spans** are `{start, stop}` byte offsets in `note.text`; they let Lua point at exact regions without rewriting the buffer itself.

## Safe “action surfaces”

Lua returns intentions; the engine applies them.
- **Generation**
    - `hooks.id_generator(ctx) -> "003"` (optional)
    - `hooks.filename(ctx) -> "ADR-003-circuit-breaker.md"` (optional override)        
    - `hooks.render_frontmatter(ctx) -> table` (merged by engine) 
    - `hooks.generate(ctx, api) -> { body?, frontmatter? }`

- **Validation**    
    - `hooks.validate(note, api) -> { {severity, message, code?, span?, field?}… }`

- **Edits (optional)**    
    - `api.suggest({ {span, replacement}… })` → engine applies, preserving formatting
    - or keep validation-only (simpler/deterministic)

## Determinism & sandbox
- No `io.*`, no `os.execute`. Provide:
    - `api.now()` that returns an engine-supplied ISO string (fixed/seeded in tests)
    - `api.rand()` seeded by engine per run (for stable IDs if needed)
    - `ctx.store:get/set` small KV for counters (persisted by engine)
```

### API design Intent

```gpt-5
- **Summaries, not ASTs.** Headings/links/fences/tokens cover 95% of policies; AST streaming into Lua is slow and leaky.
- **Spans everywhere.** They let Lua point at text without owning buffers.
- **Pure hooks.** Hooks receive inputs and produce outputs; engine performs side effects.
- **Stable shapes.** All API returns are plain tables with the same fields in every release.
- **Feature toggles live in TOML.** Which hooks fire, which custom tokens you recognize, and schema discovery all stay declarative.
```

## Decision
<!-- What is the change that we're proposing and/or doing? -->

Proceed with implementing the limited escape hatch. Do not foreclose the chance we should just allow an init.lua file to bypass toml and just call the library directly. 

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

**Positive** 
1. We will be able to maintain a smaller, more logical core of statements for the .toml configuration dsl.
2. Users will be able to modify to meet their needs without feature requests. 
3. Users of the neovim plugin will be able to use a langauge they already know. 

**Mixed** 
1. We have to create and maintain a simple API. Ideally we expose this with an OpenAPI contract as documentation. 

## Updates
<!-- Changes that happened when the rubber met the road -->

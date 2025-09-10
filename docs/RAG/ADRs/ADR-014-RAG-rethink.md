---
node_id: ADR-014
tags:
  - RAG
  - retrieval 
status: draft
depends_on:
  - ADR-003c
created_date: 2025-09-01
last_modified: 2025-09-01
related_files: []
---

# ADR-014-RAG-rethink

## Objective
<!-- A concise statement explaining the goal of this decision. -->

Various thoughts coming out of [[NOTE-0002-lerning-bout-rags]] about making the actual retrieval more robust. To copy the core objective section; 

### Hypothesis

```markdown 
The get command is too simplistic in [[ADR-003c-v1.1-CLI-commands]] to really be the ideal retrieval for what I'm going for with cli rag. This issue is i'm not sure what is ideal and because I don't know the territory I'm just naively reinventing wheels. I need to slow down and read and understand what has been found to work or not. 

The issue as I see it; 

##### `get` (alias show)

Resolve a note and (optionally) its neighborhood; print for humans or ai.

**Flags**

- `--id <ID>` (or `--path <P>`)
- `--depth N` (default 2)
- `--include-bidirectional`
- `--format md|ai|json`
    - `md`: stitched doc with frontmatter + appended neighbors
    - `ai`: compact JSON with `{frontmatter, text, neighbors: [...]}` optimized for LLM
    - `json`: full parse summary `{frontmatter, headings, wikilinks, md_links, code_fences, text}`

This is very basic text dumping. We need to have some more stepwise pattern where a get is more unified as an atomic action. You get a full context object or flag it down to metadata only. 

so in this theory a 'full context object' is 

- the files index metadata 
- read out of the file context 
- bidirectional index metadata + frontmatter of cluster region. 
  
then the llm can search or move around exploring the map step by step with an view into what might be related. I have no idea how or if this maps onto existing patterns. 
```

## Context
<!-- What is the issue that we're seeing that is motivating this decision or change? -->

The core retrieval commands are `get` as shown above and arguably `search` `graph` and `ai index`. 

Of those search and graph seem largely okay. Graph I have no major issues with. 

Search is the place a traditional vector rag could be usable. This is where traditional pre-generation techniques could let you ask fuzzy questions of the graph. I think this would be of largely limited value. 

The group command how has room for potential expansion. 

### [{NOTE},{2025/9/8}: The groups command is no longer in the codebase. the general concepts have survived but are handled by the `ai index` sub commands in [[ADR-003d-v1.2-locked-CLI-commands]]]

Microsoft GraphRAG like so; 

```
- The LLM processes the entire private dataset, creating references to all entities and relationships within the source data, which are then used to create an LLM-generated knowledge graph. 
- This graph is then used to create a bottom-up clustering that organizes the data hierarchically into semantic clusters (indicated by using color in Figure 3 below). This partitioning allows for pre-summarization of semantic concepts and themes, which aids in holistic understanding of the dataset. 
- At query time, both of these structures are used to provide materials for the LLM context window when answering a question.
```

The way you could imagine this applying to cli-rag is with a sub command under the ai group that is sort of like an auto setup eg 

`cli-rag ai groups --init` 

This would be similar to the command to create an agents.md|CLAUDE.md etc. 

The llm flow could be something like...
1. command lists un-grouped items. 
2. llm reviews items. 
3. llm uses a command like id 
   
`cli-rag ai groups --add "retrieval" "ADR-003, ADR-007, ADR-006" 

this tags the groups on the ADR and ensures the key value pair exists and is updated in the related schema. 

This all makes sense in theory but may be too complicated in practice. 

Regarding the main git command I feel similarly that it may need to be split into a human accessible command and an AI one. The twist however --I don't really know if the human command is usable at all. 

As a human i'd want to use the neovim extension. or have a command like 

`cli-rag edit <node-id>` 

that just opens the editor. it's not 100% clear to me when as a human i'd just want to have it dumped to std out outside of scripting reason. 

A gpt-5 proposed shape 

```
get 

**Flags**
    --depth N default 0 (main doc only)
    --edges <csv> e.g., depends_on,blocked_by (edge allowlist)
    --include-bidirectional
    --max-fanout <n> default 5
    --max-nodes <n> default 50
    --token-budget <n> default 4000
    --expand <HANDLE|ID> expand a specific neighbor
    --format ai|json|md
    --chunked stream NDJSON blocks (see below)
```

I find this for some reason hard to reason around. It feels over verbose and complicated for what a human is actually going to want to do. 

I guess the tension in some ways is that in my original thought making it atomic meant less choices not more eg all gets just pull a node and its environs. then step to another node in the enviorns etc. but all the calls are basically the same with 1 or two modifiers structure more coming from chaining them. 

The tool as you are describing it would be more powerful and useful. but i'm not sure I myself could reason about and make use of it.  

Biggest issue is i'm still not really sure what I want or what the ai needs/would actually use. 

## Decision
<!-- What is the change that we're proposing and/or doing? -->

This served largely as ideation and the final form is ai index as seen in [[ADR-003d-v1.2-locked-CLI-commands]]

## Consequences
<!-- What becomes easier or more difficult to do because of this change? -->

accepted in principal but not specific. 

## Updates
<!-- Changes that happened when the rubber met the road -->

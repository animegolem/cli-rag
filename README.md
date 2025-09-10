# cli-rag

A CLI based system for creating and managing Obsidian compliant YAML front matter. This creates a simplifed DAG that allows a local "region" to be called by an LLM.

You've found this way too early! Nothing here is ready for production. :) This will all be cleaned up over the next few days. But i would not use this right now.

## JSON/NDJSON surfaces (ACP-aligned)

- `watch --json` emits NDJSON events compatible with ACP-style session updates:
  - `{"sessionUpdate":"validated","ok":true,"docCount":N}`
  - `{"sessionUpdate":"index_written","path":"...","count":N}`
  - `{"sessionUpdate":"groups_written","path":"...","count":N}`
  Each event is a single line of JSON; stderr retains human-readable logs.

- `get --format ai` returns content blocks friendly to LLMs/editors:
  - `{ id, title, file, neighbors:{depends_on,dependents}, content:[ {type:"resource_link",uri}, {type:"text",text} ] }`

- `info --format json` includes protocol metadata:
  - `{ protocolVersion: 1, capabilities: { watchNdjson: true, aiGet: { retrievalVersion: 1 }, pathLocations: true } }`

# AGENTS.md — contracts/

## Scope & Role
- Scope: applies to everything under `contracts/`.
- Contracts are the source of truth. Code and docs must conform to these schemas and conventions.

## Change Control (STOP)
- Any change to contracts (schemas here or `contracts/global-conventions.md`) requires discussion and an entry in `contracts/changelog.md` describing rationale and impact.
- Coordinate before changing exit codes, NDJSON handshake/event order, or version signaling fields (`protocolVersion`, `luaApiVersion`).

## Layout & Naming
- Versioned layout: `contracts/v1/<area>/<name>.schema.json`.
- CLI response schemas live under `contracts/v1/cli/`.
- Keep `$id` aligned with the path (e.g., `contracts/cli/v1/info.schema.json`).

## Versioning & Compatibility
- Prefer additive, backward‑compatible changes (new optional fields). Breaking changes should introduce new versioned paths (e.g., `v2/`) or new schema names.
- Keep casing rules and TOML→JSON mapping aligned with `contracts/global-conventions.md`.

## Locked Conventions
- Treat `contracts/global-conventions.md` as normative for:
  - casing rules (TOML snake_case → JSON camelCase)
  - deterministic ordering
  - exit codes
  - NDJSON watch handshake and event shapes
  - version signaling fields in envelopes

## Validation & CI
- Validate outputs against these schemas during development. `jq` locally is fine; CI schema checks recommended.


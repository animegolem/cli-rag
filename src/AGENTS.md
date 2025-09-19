# AGENTS.md — src/

## Scope & Intent
- Scope: applies to all Rust code under `src/`.
- Implement contracts‑first. If behavior implies a schema change, stop and update `contracts/` first.

## Module Map
- `src/bin/cli-rag.rs`: thin binary wiring CLI to library.
- `src/cli.rs` and `src/commands/*`: Clap surfaces and subcommand handlers.
- `src/{config,model,discovery,index,validate,graph,watch,util}.rs`: core modules.

## Outputs & Protocol
- `--json` outputs must be camelCase and validate against `contracts/v1/**/*.schema.json`.
- Include version signaling where defined (e.g., `protocolVersion`).
- Deterministic ordering per `contracts/global-conventions.md`.
- NDJSON watch: first event `{"event":"watch_start","protocolVersion":1}`.
- Exit codes: follow `contracts/global-conventions.md` (0..5).

## Testing
- Prefer inline unit tests per module (`mod tests { ... }`).
- Assert exact messages/fields where practical; avoid nondeterministic assertions.

## Style & Hygiene
- Rust 2021, 4‑space indent, ~100 cols.
- Run `cargo fmt` and `cargo clippy -D warnings` locally.
- Keep source files under ~350 lines (excluding tests) when practical.

## Stop Points
- If an implementation requires adding/removing fields in outputs, pause and propose schema edits in `contracts/` and log in `contracts/changelog.md` after agreement.


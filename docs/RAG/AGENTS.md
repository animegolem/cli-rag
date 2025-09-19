# AGENTS.md — docs/RAG/

## Scope & Role
- Scope: applies to documentation under `docs/RAG/`.
- Docs (ADRs, AI‑EPIC/IMP, logs) capture intent and plans; contracts remain authoritative for protocol shapes.

## Authoritativeness
- When a doc proposes a new field/shape, update or add the corresponding schema under `contracts/v1/` first (prefer `contracts/v1/cli/` for CLI responses), then align examples in docs.

## Key References
- CLI surfaces and examples: `docs/RAG/ADRs/ADR-003d-v1.2-locked-CLI-commands.md`.
- Locked conventions: `contracts/global-conventions.md`.

## Change Logging
- Record decisions in ADRs/AI‑IMP as usual.
- Contract changes must also be logged in `contracts/changelog.md` after discussion.


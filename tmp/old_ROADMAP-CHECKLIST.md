# adr-rag — Post-Refactor Roadmap & Checklist

This checklist focuses on hardening core behavior first, then UX polish, then power features. We’ll keep scope tight per phase and prune as we learn.

## Handoff

- Status: Phase A core items implemented (write-index removed, de-dup by id, path-aware globs, index versioning, YAML/TOML front-matter + supersedes arrays, AdrDoc Serialize, initial NDJSON). Next: implement front-matter parser abstraction trait, then move to Phase B (tables/colors).

## Phase A — Core Correctness

- [x] Remove `--write-index` flag (simplify: auto-write on success; keep `--dry-run`)
  - [x] Drop flag from CLI and handler
  - [x] Update help/dispatch (README already aligned)
- [x] De-duplicate by `id` across bases in discovery
  - [x] Implement "newest mtime wins" policy in `load_docs` and incremental
  - [x] Keep `validate` conflict reporting for visibility
- [x] Path-aware ignore globs (Windows-friendly)
  - [x] Use `GlobSet::is_match(&Path)`; avoid lossy string conversions
- [x] Index versioning (backward compatible)
  - [x] Write `{ index_version, generated_at, items }`
  - [x] Read legacy array or object seamlessly
- [x] Front-matter parsing robustness
  - [x] Line-based delimiter scan supporting `---` (YAML) and `+++` (TOML), with CRLF + EOF handling
  - [x] Normalize `supersedes`/`superseded_by` from string|array → Vec
  - [ ] Parser abstraction trait; keep `serde_yaml` initially
- [x] `AdrDoc: Serialize`
  - [x] Derive `Serialize`
  - [ ] Centralize JSON/NDJSON printing helpers
- [x] NDJSON support (initial)
  - [x] Add `--format ndjson` for streaming results (search, topics, validate summary)
  - [ ] Extend to other commands as needed

## Phase B — UX Polish

- [ ] Plain output tables (doctor/topics/etc.) via `comfy_table`
- [ ] Colorize errors/warnings in `validate` via `anstream`
- [ ] Friendlier errors (e.g., “Config not found — run init?”)
- [ ] `--debug` logging toggle (env_logger/tracing)

## Phase C — Editor & CLI Ergonomics

- [ ] `edit --id ADR-XXX` (open file via `try_open_editor`)
- [ ] `config edit` (open `.adr-rag.toml`)
- [ ] Interactive `init` (dialoguer) with `--non-interactive` fallback

## Phase D — Links & Suggestions

- [ ] Obsidian-style `[[links]]` check (exact filename match)
  - [ ] Validate unresolved links; cross-base disambiguation strategy
- [ ] Fuzzy suggestions for unknown IDs (get/cluster/path)
  - [ ] Offer closest matches, accept with prompt or `--yes`

## Phase E — New Notes

- [ ] `new` command
  - [ ] Args: `type` (adr|imp), `title`, `depends_on`, `template?`
  - [ ] Filename from title; scaffold legal front-matter/body

## Phase F — Graph Enhancements

- [ ] DOT subgraphs by group/status (optional)
- [ ] Consistent graph JSON schema + tests

## Phase G — Heavy Lifts (Separate RFCs)

- [ ] `rename --id ADR-012 --new-id ADR-012R`
  - [ ] Dry-run diff, backups, atomic writes, re-validate, rollback plan
- [ ] `supersede --old ADR-008 --new ADR-031`
  - [ ] Update both notes’ front-matter; transactional write

## Dependencies & Parsing (Watchlist)

- [ ] YAML crate deprecation follow-up
  - [ ] Keep behind parsing trait; evaluate `serde_yml` path

## Acceptance Targets (per milestone)

- Phase A: de-dup, Windows globs, index versioning, `AdrDoc: Serialize`, NDJSON
- Phase B/C: tables + colors, editor commands, friendlier errors, debug logging
- Phase D: wiki link checks, ID suggestions
- Phase E: `new` scaffold
- Phase F: graph subgraphs + tests

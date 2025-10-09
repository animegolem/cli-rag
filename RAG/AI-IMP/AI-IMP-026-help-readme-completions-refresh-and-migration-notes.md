---
node_id: AI-IMP-026
tags:
  - IMP-LIST
  - Implementation
  - docs
  - help
  - completions
kanban_status: completed
depends_on:
  - AI-EPIC-003
  - AI-IMP-022
  - AI-IMP-025
confidence_score: 0.88
created_date: 2025-09-18
close_date: 2025-10-08
---

# AI-IMP-026-help-readme-completions-refresh-and-migration-notes

## Summary of Issue #1
After unifying AI commands and deprecating legacy `new`, the CLI’s help text, README examples, and shell completions must be refreshed to guide users toward the intended surfaces. Scope: update `--help` descriptions, regenerate completions, rewrite README usage sections (ai index plan/apply; ai new start/submit/cancel/list), and add a concise migration note. Done when `cli-rag --help` and README are consistent with ADR-003d and completions include the new `ai index` path. {LOC|20}

### Out of Scope 
- Changing command behavior or JSON schemas.
- Adding man pages or website docs (tracked separately under ADR-012 follow-ups). {LOC|10}

### Design/Approach  
- Help text: ensure parent/child descriptions clearly separate human vs AI/machine surfaces and that deprecations are labeled.
- Completions: regenerate bash/zsh/fish to include `ai index plan|apply` and `ai new …` subcommands; verify existing CI step still works.
- README: consolidate command overview; modernize examples to the `ai` namespace; add a clear migration box (legacy → ai).
- CI: keep the contracts job unchanged beyond example text; completions step remains a basic smoke.
- Provide a short “quickstart” snippet that sets expectations for config + `ai new` flows. {LOC|25}

### Files to Touch
- `src/cli.rs`: update command and subcommand `about`/help strings (no functional changes).
- `src/commands/completions.rs`: ensure generation covers new subpaths; no logic changes.
- `README.md`: rewrite examples (search, ai index plan/apply, ai new start/submit/cancel/list), add migration note.
- `.github/workflows/ci.yml`: textual updates to example echo blocks only if present; completions step remains.
- `contracts/changelog.md`: note the alignment and docs refresh. {LOC|25}

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE> 

- [x] Help: update subcommand descriptions to reflect unified AI layout (aliases removed).
- [x] Completions: regenerate and validate basic generation in CI for bash and zsh.
- [x] README: update usage sections and examples to `cli-rag ai index …` and `cli-rag ai new …`; add migration box.
- [x] README: include an `ai new start --format json | jq '.noteTemplate'` example to showcase template guidance.
- [x] CI: confirm completions generation still executes successfully.
- [x] Changelog: capture the docs/help alignment changes.

### Acceptance Criteria
**Scenario:** Help output reflects unified commands
GIVEN a built binary
WHEN running `cli-rag --help` and `cli-rag ai --help`
THEN help lists `ai index plan|apply` and `ai new …` (with no legacy aliases).

**Scenario:** README examples match behavior
GIVEN the repository README
WHEN following the examples for `ai index …` and `ai new …`
THEN the commands execute successfully and produce outputs consistent with the contracts schemas.

**Scenario:** Completions include new subpaths
GIVEN generated completions for bash and zsh
WHEN tab-completing after `cli-rag ai`
THEN `index` and `new` are offered with their subcommands.

### Issues Encountered 
{LOC|20}

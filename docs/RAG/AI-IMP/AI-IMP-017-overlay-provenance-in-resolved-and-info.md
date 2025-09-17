---
node_id: AI-IMP-017
tags:
  - IMP-LIST
  - Implementation
  - overlays
  - resolved
  - info
kanban_status: done
depends_on:
  - AI-EPIC-001
confidence_score: 0.83
created_date: 2025-09-15
close_date: 2025-09-17
---

# AI-IMP-017-overlay-provenance-in-resolved-and-info

## Summary of Issue #1
Expose overlay provenance in resolved snapshot (repo_path, user_path) and keep `info` alignment (capabilities.overlaysEnabled true/false). CI already smokes overlays on/off; add provenance assertion.
`info --format json` should emit an `overlays` block mirroring the resolved snapshot to support scripting.

### Out of Scope
- Executing overlay code beyond loading/merging tables; keep current safety posture.

### Design/Approach
- Extend resolved.json overlays block to include `repoPath` and `userPath` (already present), ensure values populate when files exist.
- Add a small `info --format json` field if needed (not required by contracts), or keep provenance only in resolved.json.
- Update CI contracts job to assert provenance fields when overlay files are present.

### Files to Touch
- `src/config/loader.rs` or where resolved is assembled (validate_cmd.rs already writes overlays paths).
- `tests/`: add integration ensuring overlays paths propagate.
- `.github/workflows/ci.yml`: add assertion for repo overlay path.

### Implementation Checklist

<CRITICAL_RULE>
Before marking an item complete on the checklist MUST **stop** and **think**. Have you validated all aspects are **implemented** and **tested**? 
</CRITICAL_RULE>

- [x] Confirm overlays paths written in resolved.json and adjust if necessary.
- [x] Tests: with/without overlay files; paths present/empty.
- [x] CI: assert resolved overlays paths when overlay exists in fixture workdir.

### Acceptance Criteria
GIVEN a repo `.cli-rag.lua`, WHEN running `validate`, THEN `.cli-rag/resolved.json.overlays.repoPath` equals the overlay path; WHEN `--no-lua`, THEN `enabled=false` and paths remain unchanged or empty.

### Issues Encountered
{LOC|20}

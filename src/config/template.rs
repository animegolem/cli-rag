use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn write_template(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let mut f = fs::File::create(path).with_context(|| format!("creating {:?}", path))?;
    f.write_all(TEMPLATE.as_bytes())?;
    Ok(())
}

pub const TEMPLATE: &str = r#"# Repo-local CLI config (cli-rag)

# One or more directories to scan or read an index from.
# Prefer `filepaths`; `bases` is still accepted for backwards-compat.
filepaths = [
  "docs/masterplan",
  # "docs/notes",
]

# Where to read/write the index and semantic groups (paths are relative to each base).
index_relative = "index/adr-index.json"
groups_relative = "index/semantic-groups.json"

# Discovery and semantics
file_patterns = ["ADR-*.md", "ADR-DB-*.md", "IMP-*.md"]
ignore_globs  = ["**/node_modules/**", "**/.obsidian/**"]
allowed_statuses = [
  "draft", "incomplete", "proposed", "accepted",
  "complete", "design", "legacy-reference", "superseded"
]

[defaults]
depth = 2
include_bidirectional = true
include_content = true

# Note Types (Schema) — Optional, per-type rules and validation
#
# Define one or more [[schema]] blocks to validate different note types
# (e.g., ADR vs IMP). Matching is by file_patterns; first match wins.
# Unknown keys policy lets you treat unexpected front-matter as ignore|warn|error.
#
# [[schema]]
# name = "ADR"
# file_patterns = ["ADR-*.md", "ADR-DB-*.md"]
# required = ["id", "tags", "status", "depends_on"]
# unknown_policy = "ignore"   # ignore | warn | error (default: ignore)
# cycle_policy = "warn"        # warn | error | ignore (default: warn)
# filename_template = "{{id}}-{{title}}.md"  # optional filename pattern
# allowed_keys = ["produces", "files_touched"]  # optional pass-through keys
#
# [schema.rules.status]
# allowed = [
#   "draft", "incomplete", "proposed", "accepted",
#   "complete", "design", "legacy-reference", "superseded"
# ]
# severity = "error"          # error | warn
#
# [schema.rules.depends_on]
# type = "array"
# items = { type = "string", regex = "^(ADR|IMP)-\\d+" }
# refers_to_types = ["ADR", "IMP"]
# severity = "error"
#
# [[schema]]
# name = "IMP"
# file_patterns = ["IMP-*.md"]
# required = ["id","tags","depends_on","status","completion_date"]
# unknown_policy = "warn"
#
# [schema.rules.status]
# allowed = ["in-progress","blocked","on-hold","cancelled","done"]
# severity = "error"
#
# [schema.rules.completion_date]
# type = "date"
# format = "%Y-%m-%d"
# severity = "warn"

# Default schemas (enabled): tweak as needed

[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md", "ADR-DB-*.md"]
required = ["id", "tags", "status", "depends_on"]
unknown_policy = "ignore"

[schema.rules.status]
allowed = [
  "draft", "incomplete", "proposed", "accepted",
  "complete", "design", "legacy-reference", "superseded"
]
severity = "error"

[[schema]]
name = "IMP"
file_patterns = ["IMP-*.md"]
required = ["id", "tags", "depends_on", "status"]
unknown_policy = "ignore"

[schema.rules.status]
allowed = ["in-progress", "blocked", "on-hold", "cancelled", "done"]
severity = "error"
"#;

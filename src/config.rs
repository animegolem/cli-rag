use anyhow::{anyhow, Context, Result};
use globset::Glob;
use globset::GlobSetBuilder;
use serde::Deserialize;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct DefaultsCfg {
    #[serde(default = "default_depth")]
    pub depth: usize,
    #[serde(default = "default_true")]
    pub include_bidirectional: bool,
    #[serde(default = "default_true")]
    pub include_content: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SchemaRule {
    #[serde(default)]
    pub allowed: Vec<String>,
    #[serde(rename = "type")]
    pub r#type: Option<String>,
    #[serde(default)]
    pub min_items: Option<usize>,
    #[serde(default)]
    pub regex: Option<String>,
    #[serde(default)]
    pub refers_to_types: Option<Vec<String>>,
    #[serde(default)]
    pub severity: Option<String>, // error | warn
    #[serde(default)]
    pub format: Option<String>, // for date parsing
}

#[derive(Debug, Deserialize, Clone)]
pub struct SchemaCfg {
    pub name: String,
    pub file_patterns: Vec<String>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub unknown_policy: Option<String>, // ignore | warn | error (default ignore)
    #[serde(default)]
    pub allowed_keys: Vec<String>,
    #[serde(default)]
    pub rules: std::collections::BTreeMap<String, SchemaRule>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_bases")]
    pub bases: Vec<PathBuf>,
    #[serde(default = "default_index_rel")]
    pub index_relative: String,
    #[serde(default = "default_groups_rel")]
    pub groups_relative: String,
    #[serde(default = "default_file_patterns")]
    pub file_patterns: Vec<String>,
    #[serde(default = "default_ignore_globs")]
    pub ignore_globs: Vec<String>,
    #[serde(default = "default_allowed_statuses")]
    pub allowed_statuses: Vec<String>,
    #[serde(default = "default_defaults")]
    pub defaults: DefaultsCfg,
    // output config removed (was unused)
    #[serde(default)]
    pub schema: Vec<SchemaCfg>,
}

pub fn default_bases() -> Vec<PathBuf> {
    vec![PathBuf::from("docs/masterplan-v2")]
}
pub fn default_index_rel() -> String {
    "index/adr-index.json".to_string()
}
pub fn default_groups_rel() -> String {
    "index/semantic-groups.json".to_string()
}
pub fn default_file_patterns() -> Vec<String> {
    vec!["ADR-*.md".into(), "ADR-DB-*.md".into(), "IMP-*.md".into()]
}
pub fn default_ignore_globs() -> Vec<String> {
    vec!["**/node_modules/**".into(), "**/.obsidian/**".into()]
}
pub fn default_allowed_statuses() -> Vec<String> {
    vec![
        "draft".into(),
        "incomplete".into(),
        "proposed".into(),
        "accepted".into(),
        "complete".into(),
        "design".into(),
        "legacy-reference".into(),
        "superseded".into(),
    ]
}
pub fn default_depth() -> usize {
    2
}
pub fn default_true() -> bool {
    true
}
pub fn default_defaults() -> DefaultsCfg {
    DefaultsCfg {
        depth: default_depth(),
        include_bidirectional: true,
        include_content: true,
    }
}

pub fn find_config_upwards(explicit: &Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = explicit {
        return Some(p.clone());
    }
    if let Ok(env_path) = env::var("ADR_RAG_CONFIG") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }
    let mut dir = env::current_dir().ok()?;
    loop {
        let candidate = dir.join(".adr-rag.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        let parent = dir.parent();
        match parent {
            Some(p) => dir = p.to_path_buf(),
            None => return None,
        }
    }
}

fn find_all_configs_upwards_chain() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(mut dir) = env::current_dir() {
        loop {
            let candidate = dir.join(".adr-rag.toml");
            if candidate.exists() {
                out.push(candidate);
            }
            match dir.parent() {
                Some(p) => dir = p.to_path_buf(),
                None => break,
            }
        }
    }
    out
}

pub fn load_config(
    path_opt: &Option<PathBuf>,
    base_override: &Option<Vec<PathBuf>>,
) -> Result<(Config, Option<PathBuf>)> {
    // Detect multiple configs in scope (unless an explicit path is provided or ADR_RAG_CONFIG is set)
    let env_cfg = env::var("ADR_RAG_CONFIG").ok().map(PathBuf::from);
    let path = find_config_upwards(path_opt);
    if path_opt.is_none() && env_cfg.is_none() {
        let chain = find_all_configs_upwards_chain();
        if chain.len() > 1 {
            let list = chain
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(anyhow!(
                "E100: Multiple project configs detected. Only one .adr-rag.toml is allowed. Found: {}",
                list
            ));
        }
    }
    let mut cfg: Config = if let Some(ref p) = path {
        let s = fs::read_to_string(p).with_context(|| format!("reading config {:?}", p))?;
        toml::from_str(&s).with_context(|| format!("parsing TOML config {:?}", p))?
    } else {
        Config {
            bases: default_bases(),
            index_relative: default_index_rel(),
            groups_relative: default_groups_rel(),
            file_patterns: default_file_patterns(),
            ignore_globs: default_ignore_globs(),
            allowed_statuses: default_allowed_statuses(),
            defaults: default_defaults(),
            schema: Vec::new(),
        }
    };
    // Env override for bases (comma-separated)
    if let Ok(env_bases) = env::var("ADR_RAG_BASES") {
        let list: Vec<PathBuf> = env_bases
            .split(',')
            .map(|s| PathBuf::from(s.trim()))
            .filter(|p| !p.as_os_str().is_empty())
            .collect();
        if !list.is_empty() {
            cfg.bases = list;
        }
    }
    if let Some(override_bases) = base_override {
        if !override_bases.is_empty() {
            cfg.bases = override_bases.clone();
        }
    }
    // Invariant: unique schema names across the effective config
    if !cfg.schema.is_empty() {
        use std::collections::BTreeMap;
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        for sc in &cfg.schema {
            *seen.entry(sc.name.clone()).or_insert(0) += 1;
        }
        let dups: Vec<String> = seen
            .into_iter()
            .filter_map(|(k, v)| if v > 1 { Some(k) } else { None })
            .collect();
        if !dups.is_empty() {
            return Err(anyhow!(
                "E120: Duplicate schema name(s) detected: {}",
                dups.join(", ")
            ));
        }
    }
    Ok((cfg, path))
}

pub fn write_template(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let mut f = fs::File::create(path).with_context(|| format!("creating {:?}", path))?;
    f.write_all(TEMPLATE.as_bytes())?;
    Ok(())
}

// Helper: compile schema globsets once for reuse across modules.
pub fn build_schema_sets(cfg: &Config) -> Vec<(SchemaCfg, globset::GlobSet)> {
    let mut out = Vec::new();
    for sc in &cfg.schema {
        let mut b = GlobSetBuilder::new();
        for p in &sc.file_patterns {
            if let Ok(g) = Glob::new(p) {
                b.add(g);
            }
        }
        if let Ok(set) = b.build() {
            out.push((sc.clone(), set));
        }
    }
    out
}

pub const TEMPLATE: &str = r#"# Repo-local ADR CLI config (adr-rag)

# One or more directories to scan or read an index from.
bases = [
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write as _;

    fn unique_tmp(prefix: &str) -> PathBuf {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = PathBuf::from("tmp").join(format!("{}_{}", prefix, now));
        fs::create_dir_all(&p).ok();
        p
    }

    struct DirGuard {
        old: PathBuf,
    }
    impl DirGuard {
        fn new(to: &Path) -> Self {
            let old = std::env::current_dir().unwrap();
            std::env::set_current_dir(to).unwrap();
            Self { old }
        }
    }
    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.old);
        }
    }

    #[test]
    fn test_e100_multiple_configs_detected() {
        // Create parent/child each with a .adr-rag.toml and chdir into child.
        let parent = unique_tmp("e100_parent");
        let child = parent.join("child");
        fs::create_dir_all(&child).unwrap();
        fs::File::create(parent.join(".adr-rag.toml")).unwrap();
        fs::File::create(child.join(".adr-rag.toml")).unwrap();
        let _guard = DirGuard::new(&child);
        // Ensure no explicit path/env override
        std::env::remove_var("ADR_RAG_CONFIG");
        let res = load_config(&None, &None);
        assert!(res.is_err(), "expected E100 error");
        let msg = format!("{}", res.unwrap_err());
        assert!(msg.contains("E100"), "missing E100 in: {}", msg);
    }

    #[test]
    fn test_e120_duplicate_schema_names() {
        // Build a minimal valid config file with duplicate schema names
        let dir = unique_tmp("e120_cfg");
        let cfg_path = dir.join(".adr-rag.toml");
        let toml = r#"
bases = ["docs"]
index_relative = "index/adr-index.json"
groups_relative = "index/semantic-groups.json"
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

[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md"]
required = ["id"]

[[schema]]
name = "ADR"
file_patterns = ["ADR-DB-*.md"]
required = ["id"]
"#;
        let mut f = fs::File::create(&cfg_path).unwrap();
        f.write_all(toml.as_bytes()).unwrap();
        assert!(cfg_path.exists(), "test setup failed: cfg file not created");
        let res = load_config(&Some(cfg_path.clone()), &None);
        assert!(res.is_err(), "expected E120 error");
        let msg = format!("{}", res.unwrap_err());
        assert!(msg.contains("E120"), "missing E120 in: {}", msg);
    }
}

use std::path::PathBuf;

use super::schema::DefaultsCfg;

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

pub fn default_config_version() -> String {
    "0.1".to_string()
}

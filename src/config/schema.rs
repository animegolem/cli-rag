use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct DefaultsCfg {
    #[serde(default = "crate::config::defaults::default_depth")]
    pub depth: usize,
    #[serde(default = "crate::config::defaults::default_true")]
    pub include_bidirectional: bool,
    #[serde(default = "crate::config::defaults::default_true")]
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
    #[serde(default)]
    pub import: Vec<String>,
    #[serde(
        default = "crate::config::defaults::default_bases",
        alias = "filepaths"
    )]
    pub bases: Vec<PathBuf>,
    #[serde(default = "crate::config::defaults::default_index_rel")]
    pub index_relative: String,
    #[serde(default = "crate::config::defaults::default_groups_rel")]
    pub groups_relative: String,
    #[serde(default = "crate::config::defaults::default_file_patterns")]
    pub file_patterns: Vec<String>,
    #[serde(default = "crate::config::defaults::default_ignore_globs")]
    pub ignore_globs: Vec<String>,
    #[serde(default = "crate::config::defaults::default_allowed_statuses")]
    pub allowed_statuses: Vec<String>,
    #[serde(default = "crate::config::defaults::default_defaults")]
    pub defaults: DefaultsCfg,
    #[serde(default)]
    pub schema: Vec<SchemaCfg>,
}

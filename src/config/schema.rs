use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct IdGeneratorCfg {
    #[serde(default)]
    pub strategy: String, // increment | datetime | uuid
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub padding: Option<usize>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SchemaNewCfg {
    #[serde(default)]
    pub id_generator: Option<IdGeneratorCfg>,
    #[serde(default)]
    pub filename_template: Option<String>,
    #[serde(default)]
    pub lua_generator: Option<String>,
    #[serde(default)]
    pub output_path: Option<Vec<String>>,
    #[serde(default)]
    pub template: Option<SchemaTemplateCfg>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SchemaTemplateCfg {
    #[serde(default)]
    pub prompt: Option<TemplateStringCfg>,
    #[serde(default)]
    pub note: Option<TemplateStringCfg>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TemplateStringCfg {
    #[serde(default)]
    pub template: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct OverlayInfo {
    pub enabled: bool,
    pub repo_path: Option<PathBuf>,
    pub user_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct AuthoringCfg {
    #[serde(default)]
    pub editor: Option<String>,
    #[serde(default)]
    pub background_watch: Option<bool>,
    #[serde(default)]
    pub output_path: Option<String>,
    #[serde(default)]
    pub destinations: HashMap<String, String>,
}

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
    #[serde(default, rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    #[serde(default)]
    pub globs: Option<Vec<String>>,
    #[serde(default)]
    pub integer: Option<IntegerRule>,
    #[serde(default)]
    pub float: Option<FloatRule>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct IntegerRule {
    #[serde(default)]
    pub min: Option<i64>,
    #[serde(default)]
    pub max: Option<i64>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FloatRule {
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SchemaBodyHeadingsCfg {
    #[serde(default)]
    pub heading_check: Option<String>,
    #[serde(default)]
    pub max_count: Option<usize>,
    #[serde(default)]
    pub severity: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SchemaBodyLineCountCfg {
    #[serde(default)]
    pub scan_policy: Option<String>,
    #[serde(default)]
    pub severity: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SchemaBodyValidateCfg {
    #[serde(default)]
    pub headings: Option<SchemaBodyHeadingsCfg>,
    #[serde(default)]
    pub line_count: Option<SchemaBodyLineCountCfg>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct SchemaValidateCfg {
    #[serde(default)]
    pub severity: Option<String>,
    #[serde(default)]
    pub body: Option<SchemaBodyValidateCfg>,
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
    pub cycle_policy: Option<String>, // warn | error | ignore (default warn)
    #[serde(default)]
    pub filename_template: Option<String>, // e.g., "{{id}}-{{title}}.md"
    #[serde(default)]
    pub new: Option<SchemaNewCfg>,
    #[serde(default)]
    pub allowed_keys: Vec<String>,
    #[serde(default)]
    pub rules: std::collections::BTreeMap<String, SchemaRule>,
    #[serde(default)]
    pub validate: Option<SchemaValidateCfg>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    /// Optional configuration version from TOML (snake_case). Defaults applied in loader.
    #[serde(default)]
    pub config_version: Option<String>,
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
    #[serde(default)]
    pub authoring: AuthoringCfg,

    // Runtime-only overlay metadata (not part of TOML)
    #[serde(skip)]
    pub overlays: OverlayInfo,
}

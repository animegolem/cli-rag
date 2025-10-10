use super::*;
use crate::config::schema::{
    EdgeKindPolicy, SchemaCfg, SchemaEdgesCfg, SchemaValidateCfg, SchemaWikilinksCfg,
};
use crate::config::{
    default_allowed_statuses, default_defaults, default_file_patterns, default_groups_rel,
    default_ignore_globs, default_index_rel, Config,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tempfile::tempdir;

fn build_base_config(schema: SchemaCfg) -> Config {
    Config {
        config_version: Some(crate::config::defaults::default_config_version()),
        import: Vec::new(),
        bases: vec![],
        index_relative: default_index_rel(),
        groups_relative: default_groups_rel(),
        file_patterns: default_file_patterns(),
        ignore_globs: default_ignore_globs(),
        allowed_statuses: default_allowed_statuses(),
        defaults: default_defaults(),
        schema: vec![schema],
        authoring: crate::config::schema::AuthoringCfg::default(),
        overlays: crate::config::schema::OverlayInfo::default(),
    }
}

fn build_schema_with_wikilinks(
    wikilinks: SchemaWikilinksCfg,
    schema_severity: Option<&str>,
) -> SchemaCfg {
    SchemaCfg {
        name: "IMP".into(),
        file_patterns: vec!["AI-IMP-*.md".into()],
        required: vec![],
        unknown_policy: Some("ignore".into()),
        cycle_policy: None,
        filename_template: None,
        new: None,
        allowed_keys: vec![],
        rules: BTreeMap::new(),
        validate: Some(SchemaValidateCfg {
            severity: schema_severity.map(|s| s.into()),
            body: None,
            edges: Some(SchemaEdgesCfg {
                cross_schema: None,
                wikilinks: Some(wikilinks),
                kinds: BTreeMap::<String, EdgeKindPolicy>::new(),
            }),
        }),
    }
}

fn make_doc(path: &Path, id: &str) -> AdrDoc {
    AdrDoc {
        file: path.to_path_buf(),
        id: Some(id.into()),
        title: id.into(),
        tags: vec![],
        status: Some("draft".into()),
        groups: vec![],
        depends_on: vec![],
        supersedes: vec![],
        superseded_by: vec![],
        fm: BTreeMap::new(),
        mtime: None,
        size: None,
    }
}

mod core;
mod cross_and_cycles;
mod links_and_edges;

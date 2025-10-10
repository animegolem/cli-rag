use super::*;

#[test]
fn test_cross_schema_disallowed() {
    use crate::config::schema::{
        CrossSchemaCfg, EdgeKindPolicy, SchemaEdgesCfg, SchemaValidateCfg,
    };

    let mut kinds = BTreeMap::new();
    kinds.insert(
        "depends_on".into(),
        EdgeKindPolicy {
            required: Some("error".into()),
            cycle_detection: Some("warn".into()),
            ..Default::default()
        },
    );
    let edges = SchemaEdgesCfg {
        cross_schema: Some(CrossSchemaCfg {
            allowed_targets: vec!["ADR".into()],
        }),
        kinds,
        ..Default::default()
    };

    let validate_imp = SchemaValidateCfg {
        severity: Some("error".into()),
        body: None,
        edges: Some(edges),
    };

    let sc_imp = SchemaCfg {
        name: "IMP".into(),
        file_patterns: vec!["IMP-*.md".into()],
        required: vec!["id".into()],
        unknown_policy: None,
        cycle_policy: Some("warn".into()),
        filename_template: None,
        new: None,
        allowed_keys: vec!["depends_on".into()],
        rules: BTreeMap::new(),
        validate: Some(validate_imp),
    };

    let sc_log = SchemaCfg {
        name: "LOG".into(),
        file_patterns: vec!["LOG-*.md".into()],
        required: vec!["id".into()],
        unknown_policy: None,
        cycle_policy: None,
        filename_template: None,
        new: None,
        allowed_keys: Vec::new(),
        rules: BTreeMap::new(),
        validate: None,
    };

    let cfg = Config {
        config_version: Some(crate::config::defaults::default_config_version()),
        import: Vec::new(),
        bases: vec![],
        index_relative: default_index_rel(),
        groups_relative: default_groups_rel(),
        file_patterns: default_file_patterns(),
        ignore_globs: default_ignore_globs(),
        allowed_statuses: default_allowed_statuses(),
        defaults: default_defaults(),
        schema: vec![sc_imp, sc_log],
        authoring: crate::config::schema::AuthoringCfg::default(),
        overlays: crate::config::schema::OverlayInfo::default(),
    };

    let mut fm_imp = BTreeMap::new();
    fm_imp.insert(
        "depends_on".into(),
        serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("LOG-001".into())]),
    );

    let imp = AdrDoc {
        file: PathBuf::from("IMP-001.md"),
        id: Some("IMP-001".into()),
        title: "IMP-001".into(),
        tags: vec![],
        status: None,
        groups: vec![],
        depends_on: vec!["LOG-001".into()],
        supersedes: vec![],
        superseded_by: vec![],
        fm: fm_imp,
        mtime: None,
        size: None,
    };

    let log = AdrDoc {
        file: PathBuf::from("LOG-001.md"),
        id: Some("LOG-001".into()),
        title: "LOG-001".into(),
        tags: vec![],
        status: None,
        groups: vec![],
        depends_on: vec![],
        supersedes: vec![],
        superseded_by: vec![],
        fm: BTreeMap::new(),
        mtime: None,
        size: None,
    };

    let report = validate_docs(&cfg, &None, &vec![imp, log]);
    assert!(!report.ok);
    let errs = report.errors.join("\n");
    assert!(errs.contains("edge 'depends_on' references disallowed schema 'LOG'"));
}

#[test]
fn test_cycle_policy_override_to_error() {
    use crate::config::schema::{EdgeKindPolicy, SchemaEdgesCfg, SchemaValidateCfg};

    let mut edges = SchemaEdgesCfg::default();
    edges.kinds.insert(
        "depends_on".into(),
        EdgeKindPolicy {
            cycle_detection: Some("error".into()),
            ..Default::default()
        },
    );

    let validate_cfg = SchemaValidateCfg {
        severity: Some("warning".into()),
        body: None,
        edges: Some(edges),
    };

    let sc = SchemaCfg {
        name: "IMP".into(),
        file_patterns: vec!["IMP-*.md".into()],
        required: vec!["id".into()],
        unknown_policy: None,
        cycle_policy: Some("warn".into()),
        filename_template: None,
        new: None,
        allowed_keys: vec!["depends_on".into()],
        rules: BTreeMap::new(),
        validate: Some(validate_cfg),
    };

    let cfg = Config {
        config_version: Some(crate::config::defaults::default_config_version()),
        import: Vec::new(),
        bases: vec![],
        index_relative: default_index_rel(),
        groups_relative: default_groups_rel(),
        file_patterns: default_file_patterns(),
        ignore_globs: default_ignore_globs(),
        allowed_statuses: default_allowed_statuses(),
        defaults: default_defaults(),
        schema: vec![sc],
        authoring: crate::config::schema::AuthoringCfg::default(),
        overlays: crate::config::schema::OverlayInfo::default(),
    };

    let mut doc_a_fm = BTreeMap::new();
    doc_a_fm.insert(
        "depends_on".into(),
        serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("IMP-002".into())]),
    );
    let mut doc_b_fm = BTreeMap::new();
    doc_b_fm.insert(
        "depends_on".into(),
        serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("IMP-001".into())]),
    );

    let doc_a = AdrDoc {
        file: PathBuf::from("IMP-001.md"),
        id: Some("IMP-001".into()),
        title: "IMP-001".into(),
        tags: vec![],
        status: None,
        groups: vec![],
        depends_on: vec!["IMP-002".into()],
        supersedes: vec![],
        superseded_by: vec![],
        fm: doc_a_fm,
        mtime: None,
        size: None,
    };
    let doc_b = AdrDoc {
        file: PathBuf::from("IMP-002.md"),
        id: Some("IMP-002".into()),
        title: "IMP-002".into(),
        tags: vec![],
        status: None,
        groups: vec![],
        depends_on: vec!["IMP-001".into()],
        supersedes: vec![],
        superseded_by: vec![],
        fm: doc_b_fm,
        mtime: None,
        size: None,
    };

    let report = validate_docs(&cfg, &None, &vec![doc_a, doc_b]);
    assert!(!report.ok);
    let errs = report.errors.join("\n");
    assert!(errs.contains("cycle detected"));
    assert!(
        report.warnings.is_empty(),
        "cycle override should emit error only"
    );
}

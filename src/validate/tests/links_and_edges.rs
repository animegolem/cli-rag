use super::*;

#[test]
fn test_wikilinks_min_outgoing_warning() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("AI-IMP-001.md");
    std::fs::write(&file, "# Title\n\nNo links here.\n").unwrap();

    let schema = build_schema_with_wikilinks(
        SchemaWikilinksCfg {
            min_outgoing: Some(1),
            min_incoming: None,
            severity: Some("warning".into()),
        },
        None,
    );
    let cfg = build_base_config(schema);
    let docs = vec![make_doc(&file, "AI-IMP-001")];
    let report = validate_docs(&cfg, &None, &docs);

    assert!(report.ok);
    assert!(report
        .warnings
        .iter()
        .any(|m| m.contains("wikilinks outgoing unique targets 0 below minimum 1")));
}

#[test]
fn test_wikilinks_min_incoming_error() {
    let dir = tempdir().unwrap();
    let file_a = dir.path().join("AI-IMP-001.md");
    let file_b = dir.path().join("AI-IMP-002.md");
    std::fs::write(&file_a, "# One\n\nNo links.\n").unwrap();
    std::fs::write(&file_b, "# Two\n\nStill no links.\n").unwrap();

    let schema = build_schema_with_wikilinks(
        SchemaWikilinksCfg {
            min_outgoing: None,
            min_incoming: Some(1),
            severity: None,
        },
        None,
    );
    let cfg = build_base_config(schema);
    let docs = vec![
        make_doc(&file_a, "AI-IMP-001"),
        make_doc(&file_b, "AI-IMP-002"),
    ];
    let report = validate_docs(&cfg, &None, &docs);
    assert!(!report.ok);
    assert!(report
        .errors
        .iter()
        .any(|m| m.contains("wikilinks incoming unique referrers 0 below minimum 1")));
}

#[test]
fn test_required_edges_missing_values() {
    use crate::config::schema::{EdgeKindPolicy, SchemaEdgesCfg, SchemaValidateCfg};

    let mut edges = SchemaEdgesCfg::default();
    edges.kinds.insert(
        "implements".into(),
        EdgeKindPolicy {
            required: Some("error".into()),
            ..Default::default()
        },
    );

    let validate_cfg = SchemaValidateCfg {
        severity: Some("error".into()),
        body: None,
        edges: Some(edges),
    };

    let sc_imp = SchemaCfg {
        name: "IMP".into(),
        file_patterns: vec!["IMP-*.md".into()],
        required: vec!["id".into()],
        unknown_policy: None,
        cycle_policy: None,
        filename_template: None,
        new: None,
        allowed_keys: vec!["implements".into()],
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
        schema: vec![sc_imp],
        authoring: crate::config::schema::AuthoringCfg::default(),
        overlays: crate::config::schema::OverlayInfo::default(),
    };

    let mut doc_fm = BTreeMap::new();
    doc_fm.insert("id".into(), serde_yaml::Value::String("IMP-001".into()));
    let doc = AdrDoc {
        file: PathBuf::from("IMP-001.md"),
        id: Some("IMP-001".into()),
        title: "IMP-001".into(),
        tags: vec![],
        status: None,
        groups: vec![],
        depends_on: vec![],
        supersedes: vec![],
        superseded_by: vec![],
        fm: doc_fm,
        mtime: None,
        size: None,
    };

    let docs = vec![doc.clone()];
    let doc_schema = crate::validate::schema_match::compute_doc_schema(&cfg, &docs);
    let mut id_collect_errors = Vec::new();
    let id_map = crate::validate::ids::build_id_map(&docs, &mut id_collect_errors);
    assert!(id_collect_errors.is_empty());

    let mut schema_errors = Vec::new();
    let mut schema_warnings = Vec::new();
    crate::validate::schema_rules::apply_schema_validation(
        &cfg,
        &docs,
        &doc_schema,
        &id_map,
        &mut schema_errors,
        &mut schema_warnings,
    );
    assert!(schema_errors
        .iter()
        .any(|m| m.contains("edge 'implements' missing required references")));

    let report = validate_docs(&cfg, &None, &vec![doc]);
    assert!(!report.ok, "errors: {:?}", report.errors);
    let errs = report.errors.join("\n");
    assert!(
        errs.contains("edge 'implements' missing required references"),
        "errors: {:?}",
        report.errors
    );
}

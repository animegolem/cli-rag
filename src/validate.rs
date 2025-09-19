use crate::config::Config;
use crate::model::AdrDoc;

mod cycles;
mod ids;
mod isolation;
mod refs;
mod report;
mod rules;
mod schema_match;

pub use report::ValidationReport;

// Validate ADR docs against config: statuses, ids, duplicates/conflicts, references.
pub fn validate_docs(cfg: &Config, docs: &Vec<AdrDoc>) -> ValidationReport {
    use std::collections::{BTreeSet, HashMap};
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // Build id map and schema assignment
    let id_to_docs: HashMap<String, Vec<AdrDoc>> = ids::build_id_map(docs, &mut errors);
    let doc_schema: HashMap<String, String> = schema_match::compute_doc_schema(cfg, docs);
    // Multi-schema match detection by file name patterns
    {
        let matches = schema_match::compute_file_schema_matches(cfg, docs);
        for (path, names) in matches {
            if names.len() > 1 {
                errors.push(format!(
                    "{}: multiple schema matches: [{}]",
                    path.display(),
                    names.join(", ")
                ));
            }
        }
    }

    // Status checks (only apply global list if no schema status rule exists)
    rules::check_statuses(cfg, docs, &doc_schema, &mut errors);

    // Duplicates and conflicts
    ids::detect_dups_conflicts(&id_to_docs, &mut errors);

    // Reference existence
    let id_set: BTreeSet<String> = id_to_docs.keys().cloned().collect();
    refs::check_references(docs, &id_set, &mut errors);

    // Schema-based validation (required, unknown policy, rules)
    rules::apply_schema_validation(
        cfg,
        docs,
        &doc_schema,
        &id_to_docs,
        &mut errors,
        &mut warnings,
    );

    // Cycle detection (depends_on graph) — policy per schema: warn|error|ignore
    {
        use std::collections::HashMap as Map;
        let mut adj: Map<String, Vec<String>> = Map::new();
        for (id, lst) in &id_to_docs {
            if let Some(d) = lst.first() {
                adj.insert(id.clone(), d.depends_on.clone());
            }
        }
        for cyc in cycles::find_cycles(&adj) {
            if !cyc.is_empty() {
                // Determine severity from involved schemas
                let mut severity = "warn"; // default to warn to preserve current behavior
                for nid in cyc.iter() {
                    if let Some(sname) = doc_schema.get(nid) {
                        if let Some(sc) = cfg.schema.iter().find(|s| &s.name == sname) {
                            if let Some(policy) = sc.cycle_policy.as_deref() {
                                match policy {
                                    "error" => {
                                        severity = "error";
                                        break;
                                    }
                                    "ignore" => {}
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                let msg = format!("cycle detected: {}", cyc.join(" -> "));
                if let Some(first) = cyc.first() {
                    if let Some(doc) = id_to_docs.get(first).and_then(|v| v.first()) {
                        if severity == "error" {
                            errors.push(format!("{}: {}", doc.file.display(), msg));
                        } else if severity != "ignore" {
                            warnings.push(format!("{}: {}", doc.file.display(), msg));
                        }
                    } else if severity == "error" {
                        errors.push(msg);
                    } else if severity != "ignore" {
                        warnings.push(msg);
                    }
                }
            }
        }
    }

    // Isolation warnings
    isolation::warn_isolated(docs, &id_to_docs, &mut warnings);

    let ok = errors.is_empty();
    ValidationReport {
        ok,
        errors,
        warnings,
        doc_count: docs.len(),
        id_count: id_to_docs.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        default_allowed_statuses, default_defaults, default_file_patterns, default_groups_rel,
        default_ignore_globs, default_index_rel, Config,
    };
    use std::path::PathBuf;

    #[test]
    fn test_validate_docs_invalid_status_and_refs_and_duplicates() {
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
            schema: Vec::new(),
            authoring: crate::config::schema::AuthoringCfg::default(),
            overlays: crate::config::schema::OverlayInfo::default(),
        };
        let d1 = AdrDoc {
            file: PathBuf::from("X.md"),
            id: Some("X".into()),
            title: "X".into(),
            tags: vec![],
            status: Some("weird".into()),
            groups: vec![],
            depends_on: vec!["NOPE".into()],
            supersedes: vec![],
            superseded_by: vec![],
            fm: std::collections::BTreeMap::new(),
            mtime: None,
            size: None,
        };
        let d2 = AdrDoc {
            file: PathBuf::from("A1.md"),
            id: Some("A".into()),
            title: "A v1".into(),
            tags: vec![],
            status: Some("draft".into()),
            groups: vec![],
            depends_on: vec![],
            supersedes: vec![],
            superseded_by: vec![],
            fm: std::collections::BTreeMap::new(),
            mtime: None,
            size: None,
        };
        let d3 = AdrDoc {
            file: PathBuf::from("A2.md"),
            id: Some("A".into()),
            title: "A v2".into(),
            tags: vec![],
            status: Some("draft".into()),
            groups: vec![],
            depends_on: vec![],
            supersedes: vec![],
            superseded_by: vec![],
            fm: std::collections::BTreeMap::new(),
            mtime: None,
            size: None,
        };
        let docs = vec![d1, d2, d3];
        let report = validate_docs(&cfg, &docs);
        assert!(!report.ok);
        let msg = report.errors.join("\n");
        assert!(msg.contains("invalid status"));
        assert!(msg.contains("depends_on 'NOPE' not found"));
        assert!(msg.contains("conflict for id A"));
    }

    #[test]
    fn test_schema_required_unknown_and_refers_to_types() {
        use crate::config::{SchemaCfg, SchemaRule};
        use std::collections::BTreeMap;

        // Build config with two schemas so cross-type ref check can fire
        let mut rules: BTreeMap<String, SchemaRule> = BTreeMap::new();
        rules.insert(
            "depends_on".into(),
            SchemaRule {
                allowed: vec![],
                r#type: Some("array".into()),
                min_items: None,
                regex: None,
                refers_to_types: Some(vec!["ADR".into()]),
                severity: Some("error".into()),
                format: None,
            },
        );
        let sc_adr = SchemaCfg {
            name: "ADR".into(),
            file_patterns: vec!["ADR-*.md".into()],
            required: vec!["id".into(), "tags".into()],
            unknown_policy: Some("warn".into()),
            cycle_policy: None,
            filename_template: None,
            new: None,
            allowed_keys: vec![],
            rules,
        };
        let sc_imp = SchemaCfg {
            name: "IMP".into(),
            file_patterns: vec!["IMP-*.md".into()],
            required: vec!["id".into()],
            unknown_policy: Some("ignore".into()),
            cycle_policy: None,
            filename_template: None,
            new: None,
            allowed_keys: vec![],
            rules: BTreeMap::new(),
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
            schema: vec![sc_adr, sc_imp],
            authoring: crate::config::schema::AuthoringCfg::default(),
            overlays: crate::config::schema::OverlayInfo::default(),
        };

        // ADR doc with empty required 'tags', unknown key in fm, and depends_on an IMP doc
        let mut fm = BTreeMap::new();
        fm.insert("foo".into(), serde_yaml::Value::String("bar".into()));
        fm.insert(
            "depends_on".into(),
            serde_yaml::Value::Sequence(vec![serde_yaml::Value::String("IMP-002".into())]),
        );
        let d1 = AdrDoc {
            file: PathBuf::from("ADR-001.md"),
            id: Some("ADR-001".into()),
            title: "ADR-001".into(),
            tags: vec![],
            status: Some("draft".into()),
            groups: vec![],
            depends_on: vec!["IMP-002".into()],
            supersedes: vec![],
            superseded_by: vec![],
            fm,
            mtime: None,
            size: None,
        };
        let d2 = AdrDoc {
            file: PathBuf::from("IMP-002.md"),
            id: Some("IMP-002".into()),
            title: "IMP-002".into(),
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

        let report = validate_docs(&cfg, &vec![d1, d2]);
        assert!(!report.ok);
        let errs = report.errors.join("\n");
        assert!(errs.contains("missing required 'tags'"));
        assert!(errs.contains("references IMP-002"));
        let warns = report.warnings.join("\n");
        assert!(warns.contains("unknown keys: foo"));
    }

    #[test]
    fn test_warn_on_isolated_adrs() {
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
            schema: Vec::new(),
            authoring: crate::config::schema::AuthoringCfg::default(),
            overlays: crate::config::schema::OverlayInfo::default(),
        };
        let d = AdrDoc {
            file: PathBuf::from("A.md"),
            id: Some("A".into()),
            title: "A".into(),
            tags: vec![],
            status: None,
            groups: vec![],
            depends_on: vec![],
            supersedes: vec![],
            superseded_by: vec![],
            fm: std::collections::BTreeMap::new(),
            mtime: None,
            size: None,
        };
        let report = validate_docs(&cfg, &vec![d]);
        assert!(report.ok); // warnings allowed
        let warns = report.warnings.join("\n");
        assert!(warns.contains("has no graph connections"));
    }
}

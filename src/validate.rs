use crate::config::Config;
use crate::model::AdrDoc;
use std::path::PathBuf;

mod body;
mod cycles;
mod ids;
mod isolation;
mod refs;
mod report;
mod rules;
mod schema_match;
mod schema_rules;
mod wikilinks;

pub use report::ValidationReport;

fn severity_rank_from_str(value: Option<&str>) -> u8 {
    match value.map(|s| s.to_ascii_lowercase()) {
        Some(ref v) if v == "error" => 2,
        Some(ref v) if v == "warning" || v == "warn" => 1,
        Some(ref v) if v == "ignore" => 0,
        _ => 1,
    }
}

// Validate ADR docs against config: statuses, ids, duplicates/conflicts, references.
pub fn validate_docs(
    cfg: &Config,
    cfg_path: &Option<PathBuf>,
    docs: &Vec<AdrDoc>,
) -> ValidationReport {
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
    schema_rules::apply_schema_validation(
        cfg,
        docs,
        &doc_schema,
        &id_to_docs,
        &mut errors,
        &mut warnings,
    );

    body::apply_body_validation(
        cfg,
        cfg_path,
        docs.as_slice(),
        &doc_schema,
        &mut errors,
        &mut warnings,
    );

    wikilinks::apply_wikilink_policy(
        cfg,
        docs.as_slice(),
        &doc_schema,
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
            if cyc.is_empty() {
                continue;
            }

            let mut severity_rank: u8 = severity_rank_from_str(Some("warn"));
            for nid in cyc.iter() {
                if let Some(sname) = doc_schema.get(nid) {
                    if let Some(sc) = cfg.schema.iter().find(|s| &s.name == sname) {
                        severity_rank =
                            severity_rank.max(severity_rank_from_str(sc.cycle_policy.as_deref()));
                        if let Some(dep_policy) = sc
                            .validate
                            .as_ref()
                            .and_then(|v| v.edges.as_ref())
                            .and_then(|edges| edges.kinds.get("depends_on"))
                            .and_then(|policy| policy.cycle_detection.as_deref())
                        {
                            severity_rank =
                                severity_rank.max(severity_rank_from_str(Some(dep_policy)));
                        }
                    }
                }
            }

            let msg = format!("cycle detected: {}", cyc.join(" -> "));
            let (target_errors, target_warnings) = match severity_rank {
                2 => (true, false),
                1 => (false, true),
                _ => (false, false),
            };

            if !target_errors && !target_warnings {
                continue;
            }

            if let Some(first) = cyc.first() {
                if let Some(doc) = id_to_docs.get(first).and_then(|v| v.first()) {
                    if target_errors {
                        errors.push(format!("{}: {}", doc.file.display(), msg));
                    } else if target_warnings {
                        warnings.push(format!("{}: {}", doc.file.display(), msg));
                    }
                } else if target_errors {
                    errors.push(msg.clone());
                } else if target_warnings {
                    warnings.push(msg.clone());
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
mod tests;

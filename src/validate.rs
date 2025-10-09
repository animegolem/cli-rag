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

pub use report::ValidationReport;

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
mod tests;

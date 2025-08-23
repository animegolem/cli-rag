use crate::config::{Config, SchemaCfg, build_schema_sets};
use crate::model::AdrDoc;

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub ok: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub doc_count: usize,
    pub id_count: usize,
}

// Validate ADR docs against config: statuses, ids, duplicates/conflicts, references.
pub fn validate_docs(cfg: &Config, docs: &Vec<AdrDoc>) -> ValidationReport {
    use std::collections::{BTreeSet, HashMap};
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut id_to_docs: HashMap<String, Vec<AdrDoc>> = HashMap::new();
    // Pre-compile schema globsets
    let schema_sets: Vec<(SchemaCfg, globset::GlobSet)> = build_schema_sets(cfg);
    let mut doc_schema: HashMap<String, String> = HashMap::new(); // id -> schema name
    for d in docs {
        if let Some(ref id) = d.id {
            id_to_docs.entry(id.clone()).or_default().push(d.clone());
            // assign schema by file name
            let fname = d.file.file_name().and_then(|s| s.to_str()).unwrap_or("");
            for (sc, set) in &schema_sets {
                if set.is_match(fname) { doc_schema.insert(id.clone(), sc.name.clone()); break; }
            }
        } else {
            errors.push(format!("{}: missing id", d.file.display()));
        }
    }
    // Validate basic status against global list only if no schema rule for status applies
    for d in docs {
        if let Some(ref st) = d.status {
            let mut has_schema_status_rule = false;
            if let Some(ref id) = d.id {
                if let Some(sname) = doc_schema.get(id) {
                    if let Some(sc) = cfg.schema.iter().find(|s| &s.name == sname) {
                        if sc.rules.contains_key("status") { has_schema_status_rule = true; }
                    }
                }
            }
            if !has_schema_status_rule {
                if !cfg.allowed_statuses.iter().any(|s| s == st) {
                    errors.push(format!("{}: invalid status '{}'", d.file.display(), st));
                }
            }
        }
    }
    for (id, lst) in &id_to_docs {
        if lst.len() > 1 {
            let mut titles: BTreeSet<String> = BTreeSet::new();
            let mut statuses: BTreeSet<String> = BTreeSet::new();
            for d in lst {
                titles.insert(d.title.clone());
                if let Some(ref s) = d.status { statuses.insert(s.clone()); }
            }
            let files = lst.iter().map(|d| d.file.display().to_string()).collect::<Vec<_>>().join(", ");
            if titles.len() > 1 || statuses.len() > 1 {
                errors.push(format!("conflict for id {} (metadata differ) in: {}", id, files));
            } else {
                errors.push(format!("duplicate id {} in: {}", id, files));
            }
        }
    }
    let id_set: std::collections::BTreeSet<String> = id_to_docs.keys().cloned().collect();
    for d in docs {
        for dep in &d.depends_on { if !id_set.contains(dep) { errors.push(format!("{}: depends_on '{}' not found", d.file.display(), dep)); } }
        for s in &d.supersedes { if !id_set.contains(s) { errors.push(format!("{}: supersedes '{}' not found", d.file.display(), s)); } }
        for s in &d.superseded_by { if !id_set.contains(s) { errors.push(format!("{}: superseded_by '{}' not found", d.file.display(), s)); } }
    }
    // Schema-based validation (required keys, unknown policy, and rules)
    let reserved: BTreeSet<String> = [
        "id","tags","status","groups","depends_on","supersedes","superseded_by"
    ].into_iter().map(|s| s.to_string()).collect();
    for d in docs {
        // Skip schema checks if we didn't parse front matter (unchanged in incremental mode)
        if d.fm.is_empty() { continue; }
        let mut schema_opt: Option<SchemaCfg> = None;
        if let Some(ref id) = d.id {
            if let Some(sname) = doc_schema.get(id) {
                schema_opt = cfg.schema.iter().find(|s| &s.name == sname).cloned();
            }
        } else {
            continue;
        }
        if let Some(sc) = schema_opt {
            // Required keys: present and non-empty
            for key in &sc.required {
                if let Some(v) = d.fm.get(key) {
                    let empty = match v {
                        serde_yaml::Value::Null => true,
                        serde_yaml::Value::String(s) => s.trim().is_empty(),
                        serde_yaml::Value::Sequence(a) => a.is_empty(),
                        _ => false,
                    };
                    if empty { errors.push(format!("{}: required '{}' is empty", d.file.display(), key)); }
                } else {
                    errors.push(format!("{}: missing required '{}'", d.file.display(), key));
                }
            }
            // Unknown keys handling
            let present: BTreeSet<String> = d.fm.keys().cloned().collect();
            let rule_keys: BTreeSet<String> = sc.rules.keys().cloned().collect();
            let mut known: BTreeSet<String> = reserved.union(&rule_keys).cloned().collect();
            known = known.union(&sc.required.iter().cloned().collect()).cloned().collect();
            known = known.union(&sc.allowed_keys.iter().cloned().collect()).cloned().collect();
            let unknown: Vec<String> = present.difference(&known).cloned().collect();
            let policy = sc.unknown_policy.clone().unwrap_or_else(|| "ignore".into());
            if !unknown.is_empty() {
                match policy.as_str() {
                    "warn" => warnings.push(format!("{}: unknown keys: {}", d.file.display(), unknown.join(", "))),
                    "error" => errors.push(format!("{}: unknown keys: {}", d.file.display(), unknown.join(", "))),
                    _ => {}
                }
            }
            // Apply rules
            for (k, rule) in sc.rules.iter() {
                let sev_err = rule.severity.as_deref().unwrap_or("error") == "error";
                if let Some(val) = d.fm.get(k) {
                    // type checks
                    if let Some(t) = &rule.r#type {
                        match t.as_str() {
                            "array" => if !val.is_sequence() { if sev_err { errors.push(format!("{}: '{}' should be array", d.file.display(), k)); } else { warnings.push(format!("{}: '{}' should be array", d.file.display(), k)); } continue; },
                            "date" => {
                                if let Some(fmt) = &rule.format { if let Some(s) = val.as_str() {
                                    if chrono::NaiveDate::parse_from_str(s, fmt).is_err() {
                                        if sev_err { errors.push(format!("{}: '{}' not a valid date '{}', format {}", d.file.display(), k, s, fmt)); } else { warnings.push(format!("{}: '{}' not a valid date", d.file.display(), k)); }
                                    }
                                } }
                            }
                            _ => {}
                        }
                    }
                    if !rule.allowed.is_empty() {
                        match val {
                            serde_yaml::Value::String(s) => {
                                if !rule.allowed.iter().any(|a| a == s) {
                                    if sev_err { errors.push(format!("{}: '{}' value '{}' not allowed", d.file.display(), k, s)); } else { warnings.push(format!("{}: '{}' value '{}' not allowed", d.file.display(), k, s)); }
                                }
                            }
                            serde_yaml::Value::Sequence(arr) => {
                                for v in arr {
                                    if let Some(s) = v.as_str() { if !rule.allowed.iter().any(|a| a == s) {
                                        if sev_err { errors.push(format!("{}: '{}' contains disallowed '{}'", d.file.display(), k, s)); } else { warnings.push(format!("{}: '{}' contains disallowed '{}'", d.file.display(), k, s)); }
                                    }}
                                }
                            }
                            _ => {}
                        }
                    }
                    if let Some(min) = rule.min_items {
                        if let Some(arr) = val.as_sequence() { if arr.len() < min { if sev_err { errors.push(format!("{}: '{}' must have at least {} items", d.file.display(), k, min)); } else { warnings.push(format!("{}: '{}' must have at least {} items", d.file.display(), k, min)); } } }
                    }
                    if let Some(rx) = &rule.regex {
                        if let Ok(re) = regex::Regex::new(rx) {
                            match val {
                                serde_yaml::Value::String(s) => {
                                    if !re.is_match(s) { if sev_err { errors.push(format!("{}: '{}' does not match regex", d.file.display(), k)); } else { warnings.push(format!("{}: '{}' does not match regex", d.file.display(), k)); } }
                                }
                                serde_yaml::Value::Sequence(arr) => {
                                    for v in arr { if let Some(s) = v.as_str() { if !re.is_match(s) { if sev_err { errors.push(format!("{}: '{}' element does not match regex", d.file.display(), k)); } else { warnings.push(format!("{}: '{}' element does not match regex", d.file.display(), k)); } } } }
                                }
                                _ => {}
                            }
                        }
                    }
                    if let Some(ref_types) = &rule.refers_to_types {
                        // Only applies to arrays of string IDs
                        if let Some(arr) = val.as_sequence() {
                            for v in arr {
                                if let Some(dep_id) = v.as_str() {
                                    if let Some(dep_docs) = id_to_docs.get(dep_id) {
                                        if let Some(dep_doc) = dep_docs.first() {
                                            if let Some(dep_doc_id) = &dep_doc.id {
                                                if let Some(dep_type) = doc_schema.get(dep_doc_id) {
                                                    if !ref_types.iter().any(|t| t == dep_type) {
                                                        if sev_err { errors.push(format!("{}: '{}' references {} of type '{}' not in {:?}", d.file.display(), k, dep_id, dep_type, ref_types)); } else { warnings.push(format!("{}: '{}' references '{}' of type '{}' not in {:?}", d.file.display(), k, dep_id, dep_type, ref_types)); }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Warn on isolated ADRs (no depends_on and no dependents). Valid, but highlighted.
    let mut has_dependent: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    for (id, _ds) in &id_to_docs { has_dependent.insert(id.clone(), false); }
    for d in docs { for dep in &d.depends_on { if let Some(x) = has_dependent.get_mut(dep) { *x = true; } } }
    for d in docs {
        if let Some(ref id) = d.id {
            let depends = d.depends_on.is_empty();
            let depended = !has_dependent.get(id).copied().unwrap_or(false);
            if depends && depended {
                warnings.push(format!("{}: '{}' has no graph connections (valid, but isolated)", d.file.display(), id));
            }
        }
    }
    let ok = errors.is_empty();
    ValidationReport { ok, errors, warnings, doc_count: docs.len(), id_count: id_to_docs.len() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, default_index_rel, default_groups_rel, default_file_patterns, default_ignore_globs, default_allowed_statuses, default_defaults};
    use std::path::PathBuf;

    #[test]
    fn test_validate_docs_invalid_status_and_refs_and_duplicates() {
        let cfg = Config { 
            bases: vec![], index_relative: default_index_rel(), groups_relative: default_groups_rel(),
            file_patterns: default_file_patterns(), ignore_globs: default_ignore_globs(),
            allowed_statuses: default_allowed_statuses(), defaults: default_defaults(), schema: Vec::new()
        };
        let d1 = AdrDoc { file: PathBuf::from("X.md"), id: Some("X".into()), title: "X".into(), tags: vec![], status: Some("weird".into()), groups: vec![], depends_on: vec!["NOPE".into()], supersedes: vec![], superseded_by: vec![], fm: std::collections::BTreeMap::new(), mtime: None, size: None };
        let d2 = AdrDoc { file: PathBuf::from("A1.md"), id: Some("A".into()), title: "A v1".into(), tags: vec![], status: Some("draft".into()), groups: vec![], depends_on: vec![], supersedes: vec![], superseded_by: vec![], fm: std::collections::BTreeMap::new(), mtime: None, size: None };
        let d3 = AdrDoc { file: PathBuf::from("A2.md"), id: Some("A".into()), title: "A v2".into(), tags: vec![], status: Some("draft".into()), groups: vec![], depends_on: vec![], supersedes: vec![], superseded_by: vec![], fm: std::collections::BTreeMap::new(), mtime: None, size: None };
        let docs = vec![d1, d2, d3];
        let report = validate_docs(&cfg, &docs);
        assert!(!report.ok);
        let msg = report.errors.join("\n");
        assert!(msg.contains("invalid status"));
        assert!(msg.contains("depends_on 'NOPE' not found"));
        assert!(msg.contains("conflict for id A"));
    }
}

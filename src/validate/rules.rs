use std::collections::{BTreeSet, HashMap};

use crate::config::{Config, SchemaCfg};
use crate::model::AdrDoc;

pub fn check_statuses(
    cfg: &Config,
    docs: &Vec<AdrDoc>,
    doc_schema: &HashMap<String, String>,
    errors: &mut Vec<String>,
) {
    for d in docs {
        if let Some(ref st) = d.status {
            let mut has_schema_status_rule = false;
            if let Some(ref id) = d.id {
                if let Some(sname) = doc_schema.get(id) {
                    if let Some(sc) = cfg.schema.iter().find(|s| &s.name == sname) {
                        if sc.rules.contains_key("status") {
                            has_schema_status_rule = true;
                        }
                    }
                }
            }
            if !has_schema_status_rule && !cfg.allowed_statuses.iter().any(|s| s == st) {
                errors.push(format!("{}: invalid status '{}'", d.file.display(), st));
            }
        }
    }
}

pub fn apply_schema_validation(
    cfg: &Config,
    docs: &Vec<AdrDoc>,
    doc_schema: &HashMap<String, String>,
    id_to_docs: &HashMap<String, Vec<AdrDoc>>,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    // Schema-based validation (required keys, unknown policy, and rules)
    let reserved: BTreeSet<String> = [
        "id",
        "tags",
        "status",
        "groups",
        "depends_on",
        "supersedes",
        "superseded_by",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect();

    for d in docs {
        // Skip schema checks if we didn't parse front matter (unchanged in incremental mode)
        if d.fm.is_empty() {
            continue;
        }
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
                    if empty {
                        errors.push(format!("{}: required '{}' is empty", d.file.display(), key));
                    }
                } else {
                    errors.push(format!("{}: missing required '{}'", d.file.display(), key));
                }
            }
            // Unknown keys handling
            let present: BTreeSet<String> = d.fm.keys().cloned().collect();
            let rule_keys: BTreeSet<String> = sc.rules.keys().cloned().collect();
            let mut known: BTreeSet<String> = reserved.union(&rule_keys).cloned().collect();
            known = known
                .union(&sc.required.iter().cloned().collect())
                .cloned()
                .collect();
            known = known
                .union(&sc.allowed_keys.iter().cloned().collect())
                .cloned()
                .collect();
            let unknown: Vec<String> = present.difference(&known).cloned().collect();
            let policy = sc.unknown_policy.clone().unwrap_or_else(|| "ignore".into());
            if !unknown.is_empty() {
                match policy.as_str() {
                    "warn" => warnings.push(format!(
                        "{}: unknown keys: {}",
                        d.file.display(),
                        unknown.join(", ")
                    )),
                    "error" => errors.push(format!(
                        "{}: unknown keys: {}",
                        d.file.display(),
                        unknown.join(", ")
                    )),
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
                            "array" => {
                                if !val.is_sequence() {
                                    if sev_err {
                                        errors.push(format!(
                                            "{}: '{}' should be array",
                                            d.file.display(),
                                            k
                                        ));
                                    } else {
                                        warnings.push(format!(
                                            "{}: '{}' should be array",
                                            d.file.display(),
                                            k
                                        ));
                                    }
                                    continue;
                                }
                            }
                            "date" => {
                                if let Some(fmt) = &rule.format {
                                    if let Some(s) = val.as_str() {
                                        if chrono::NaiveDate::parse_from_str(s, fmt).is_err() {
                                            if sev_err {
                                                errors.push(format!(
                                                    "{}: '{}' not a valid date '{}', format {}",
                                                    d.file.display(),
                                                    k,
                                                    s,
                                                    fmt
                                                ));
                                            } else {
                                                warnings.push(format!(
                                                    "{}: '{}' not a valid date",
                                                    d.file.display(),
                                                    k
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if !rule.allowed.is_empty() {
                        match val {
                            serde_yaml::Value::String(s) => {
                                if !rule.allowed.iter().any(|a| a == s) {
                                    if sev_err {
                                        errors.push(format!(
                                            "{}: '{}' value '{}' not allowed",
                                            d.file.display(),
                                            k,
                                            s
                                        ));
                                    } else {
                                        warnings.push(format!(
                                            "{}: '{}' value '{}' not allowed",
                                            d.file.display(),
                                            k,
                                            s
                                        ));
                                    }
                                }
                            }
                            serde_yaml::Value::Sequence(arr) => {
                                for v in arr {
                                    if let Some(s) = v.as_str() {
                                        if !rule.allowed.iter().any(|a| a == s) {
                                            if sev_err {
                                                errors.push(format!(
                                                    "{}: '{}' contains disallowed '{}'",
                                                    d.file.display(),
                                                    k,
                                                    s
                                                ));
                                            } else {
                                                warnings.push(format!(
                                                    "{}: '{}' contains disallowed '{}'",
                                                    d.file.display(),
                                                    k,
                                                    s
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    if let Some(min) = rule.min_items {
                        if let Some(arr) = val.as_sequence() {
                            if arr.len() < min {
                                if sev_err {
                                    errors.push(format!(
                                        "{}: '{}' must have at least {} items",
                                        d.file.display(),
                                        k,
                                        min
                                    ));
                                } else {
                                    warnings.push(format!(
                                        "{}: '{}' must have at least {} items",
                                        d.file.display(),
                                        k,
                                        min
                                    ));
                                }
                            }
                        }
                    }
                    if let Some(rx) = &rule.regex {
                        if let Ok(re) = regex::Regex::new(rx) {
                            match val {
                                serde_yaml::Value::String(s) => {
                                    if !re.is_match(s) {
                                        if sev_err {
                                            errors.push(format!(
                                                "{}: '{}' does not match regex",
                                                d.file.display(),
                                                k
                                            ));
                                        } else {
                                            warnings.push(format!(
                                                "{}: '{}' does not match regex",
                                                d.file.display(),
                                                k
                                            ));
                                        }
                                    }
                                }
                                serde_yaml::Value::Sequence(arr) => {
                                    for v in arr {
                                        if let Some(s) = v.as_str() {
                                            if !re.is_match(s) {
                                                if sev_err {
                                                    errors.push(format!(
                                                        "{}: '{}' element does not match regex",
                                                        d.file.display(),
                                                        k
                                                    ));
                                                } else {
                                                    warnings.push(format!(
                                                        "{}: '{}' element does not match regex",
                                                        d.file.display(),
                                                        k
                                                    ));
                                                }
                                            }
                                        }
                                    }
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
                                                        if sev_err {
                                                            errors.push(format!("{}: '{}' references {} of type '{}' not in {:?}", d.file.display(), k, dep_id, dep_type, ref_types));
                                                        } else {
                                                            warnings.push(format!("{}: '{}' references '{}' of type '{}' not in {:?}", d.file.display(), k, dep_id, dep_type, ref_types));
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
    }
}

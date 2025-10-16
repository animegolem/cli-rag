use super::field_rules::{validate_field_rules, FieldRuleContext};
use crate::config::{Config, SchemaCfg};
use crate::model::AdrDoc;
use std::collections::{BTreeSet, HashMap};

pub fn apply_schema_validation(
    cfg: &Config,
    docs: &Vec<AdrDoc>,
    doc_schema: &HashMap<String, String>,
    id_to_docs: &HashMap<String, Vec<AdrDoc>>,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
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

    for doc in docs {
        if doc.fm.is_empty() {
            continue;
        }
        let schema_cfg = match resolve_schema(cfg, doc, doc_schema) {
            Some(sc) => sc,
            None => continue,
        };

        validate_required_keys(doc, &schema_cfg, errors);
        validate_unknown_keys(doc, &schema_cfg, &reserved, warnings, errors);

        let ctx = FieldRuleContext {
            doc,
            schema_cfg: &schema_cfg,
            doc_schema,
            id_to_docs,
            errors,
            warnings,
        };
        validate_field_rules(ctx);
        validate_edge_policies(doc, &schema_cfg, doc_schema, id_to_docs, errors, warnings);
    }
}

fn resolve_schema(
    cfg: &Config,
    doc: &AdrDoc,
    doc_schema: &HashMap<String, String>,
) -> Option<SchemaCfg> {
    let doc_id = doc.id.as_ref()?;
    let schema_name = doc_schema.get(doc_id)?;
    cfg.schema.iter().find(|s| &s.name == schema_name).cloned()
}

fn validate_required_keys(doc: &AdrDoc, schema: &SchemaCfg, errors: &mut Vec<String>) {
    let doc_path = doc.display_path();
    for key in &schema.required {
        match doc.fm.get(key) {
            Some(value) => {
                let empty = match value {
                    serde_yaml::Value::Null => true,
                    serde_yaml::Value::String(s) => s.trim().is_empty(),
                    serde_yaml::Value::Sequence(seq) => seq.is_empty(),
                    _ => false,
                };
                if empty {
                    errors.push(format!("{}: required '{}' is empty", doc_path, key));
                }
            }
            None => errors.push(format!(
                "{}: missing required '{}'",
                doc_path,
                key
            )),
        }
    }
}

fn validate_unknown_keys(
    doc: &AdrDoc,
    schema: &SchemaCfg,
    reserved: &BTreeSet<String>,
    warnings: &mut Vec<String>,
    errors: &mut Vec<String>,
) {
    let doc_path = doc.display_path();
    let present: BTreeSet<String> = doc.fm.keys().cloned().collect();
    let rule_keys: BTreeSet<String> = schema.rules.keys().cloned().collect();
    let mut known: BTreeSet<String> = reserved.union(&rule_keys).cloned().collect();
    known = known
        .union(&schema.required.iter().cloned().collect())
        .cloned()
        .collect();
    known = known
        .union(&schema.allowed_keys.iter().cloned().collect())
        .cloned()
        .collect();
    let unknown: Vec<String> = present.difference(&known).cloned().collect();
    if unknown.is_empty() {
        return;
    }
    match schema.unknown_policy.as_deref().unwrap_or("ignore") {
        "warn" => warnings.push(format!("{}: unknown keys: {}", doc_path, unknown.join(", "))),
        "error" => errors.push(format!("{}: unknown keys: {}", doc_path, unknown.join(", "))),
        _ => {}
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Severity {
    Ignore = 0,
    Warning = 1,
    Error = 2,
}

impl Severity {
    fn from_str(value: Option<&str>, default: Severity) -> Severity {
        match value.map(|s| s.to_ascii_lowercase()) {
            Some(ref s) if s.trim() == "error" => Severity::Error,
            Some(ref s)
                if {
                    let v = s.trim();
                    v == "warning" || v == "warn"
                } =>
            {
                Severity::Warning
            }
            Some(ref s) if s.trim() == "ignore" => Severity::Ignore,
            _ => default,
        }
    }

    fn emit(self, message: String, errors: &mut Vec<String>, warnings: &mut Vec<String>) {
        match self {
            Severity::Error => errors.push(message),
            Severity::Warning => warnings.push(message),
            Severity::Ignore => {}
        }
    }
}

fn normalize_edge_values(value: &serde_yaml::Value) -> Vec<String> {
    match value {
        serde_yaml::Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Vec::new()
            } else {
                vec![trimmed.to_string()]
            }
        }
        serde_yaml::Value::Sequence(seq) => seq
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn validate_edge_policies(
    doc: &AdrDoc,
    schema_cfg: &SchemaCfg,
    doc_schema: &HashMap<String, String>,
    id_to_docs: &HashMap<String, Vec<AdrDoc>>,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let validate_cfg = match schema_cfg.validate.as_ref() {
        Some(v) => v,
        None => return,
    };
    let edges_cfg = match validate_cfg.edges.as_ref() {
        Some(cfg) => cfg,
        None => return,
    };
    let default_severity = Severity::from_str(validate_cfg.severity.as_deref(), Severity::Error);
    let doc_path = doc.display_path();

    for (edge_kind, policy) in &edges_cfg.kinds {
        let required_severity = Severity::from_str(policy.required.as_deref(), Severity::Ignore);
        let fm_value = doc.fm.get(edge_kind);
        let values = fm_value.map(normalize_edge_values).unwrap_or_default();

        if required_severity != Severity::Ignore && (fm_value.is_none() || values.is_empty()) {
            let msg = format!("{}: edge '{}' missing required references", doc_path, edge_kind);
            required_severity.emit(msg, errors, warnings);
            continue;
        }

        if values.is_empty() {
            continue;
        }

        let id_severity = if required_severity != Severity::Ignore {
            required_severity
        } else {
            default_severity
        };

        if edge_kind != "depends_on" {
            for target in &values {
                if !id_to_docs.contains_key(target) {
                    let msg = format!("{}: edge '{}' references unknown id '{}'", doc_path, edge_kind, target);
                    id_severity.emit(msg, errors, warnings);
                }
            }
        }

        if let Some(cross) = edges_cfg.cross_schema.as_ref() {
            if cross.allowed_targets.is_empty() {
                continue;
            }
            for target in &values {
                let target_schema = doc_schema.get(target);
                if let Some(schema_name) = target_schema {
                    if !cross.allowed_targets.contains(schema_name) {
                        let msg = format!(
                            "{}: edge '{}' references disallowed schema '{}' via '{}'",
                            doc_path, edge_kind, schema_name, target
                        );
                        default_severity.emit(msg, errors, warnings);
                    }
                }
            }
        }
    }
}

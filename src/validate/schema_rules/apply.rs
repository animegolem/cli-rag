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
                    errors.push(format!(
                        "{}: required '{}' is empty",
                        doc.file.display(),
                        key
                    ));
                }
            }
            None => errors.push(format!(
                "{}: missing required '{}'",
                doc.file.display(),
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
        "warn" => warnings.push(format!(
            "{}: unknown keys: {}",
            doc.file.display(),
            unknown.join(", ")
        )),
        "error" => errors.push(format!(
            "{}: unknown keys: {}",
            doc.file.display(),
            unknown.join(", ")
        )),
        _ => {}
    }
}

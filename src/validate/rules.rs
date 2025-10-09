use crate::config::Config;
use crate::model::AdrDoc;
use std::collections::HashMap;

pub fn check_statuses(
    cfg: &Config,
    docs: &Vec<AdrDoc>,
    doc_schema: &HashMap<String, String>,
    errors: &mut Vec<String>,
) {
    for doc in docs {
        if let Some(status) = &doc.status {
            let mut has_schema_status_rule = false;
            if let Some(id) = &doc.id {
                if let Some(schema_name) = doc_schema.get(id) {
                    if let Some(schema_cfg) = cfg.schema.iter().find(|s| &s.name == schema_name) {
                        if schema_cfg.rules.contains_key("status") {
                            has_schema_status_rule = true;
                        }
                    }
                }
            }
            if !has_schema_status_rule && !cfg.allowed_statuses.iter().any(|s| s == status) {
                errors.push(format!(
                    "{}: invalid status '{}'",
                    doc.file.display(),
                    status
                ));
            }
        }
    }
}

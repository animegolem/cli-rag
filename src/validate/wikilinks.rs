use crate::config::{Config, SchemaCfg};
use crate::model::AdrDoc;
use regex::Regex;
use std::collections::{BTreeSet, HashMap};
use std::fs;

pub fn apply_wikilink_policy(
    cfg: &Config,
    docs: &[AdrDoc],
    doc_schema: &HashMap<String, String>,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    if docs.is_empty() {
        return;
    }

    let schemas_with_policy: Vec<&SchemaCfg> = cfg
        .schema
        .iter()
        .filter(|schema| {
            schema
                .validate
                .as_ref()
                .and_then(|v| v.edges.as_ref())
                .and_then(|edges| edges.wikilinks.as_ref())
                .is_some()
        })
        .collect();
    if schemas_with_policy.is_empty() {
        return;
    }

    let mention_re = Regex::new(r"\[\[([A-Za-z]+-[0-9A-Za-z_-]+)\]\]").unwrap();
    let mut outgoing: HashMap<String, BTreeSet<String>> = HashMap::new();
    let mut incoming: HashMap<String, BTreeSet<String>> = HashMap::new();

    for doc in docs {
        let doc_id = match &doc.id {
            Some(id) => id.clone(),
            None => continue,
        };
        let doc_path = doc.display_path();
        let content = match fs::read_to_string(&doc.file) {
            Ok(c) => c,
            Err(err) => {
                warnings.push(format!(
                    "{}: unable to read file for wikilink scan: {}",
                    doc_path,
                    err
                ));
                outgoing.insert(doc_id.clone(), BTreeSet::new());
                continue;
            }
        };
        let mut targets: BTreeSet<String> = BTreeSet::new();
        for caps in mention_re.captures_iter(&content) {
            if let Some(m) = caps.get(1) {
                targets.insert(m.as_str().to_string());
            }
        }
        if !targets.is_empty() {
            for target in &targets {
                incoming
                    .entry(target.clone())
                    .or_default()
                    .insert(doc_id.clone());
            }
        }
        outgoing.insert(doc_id, targets);
    }

    for doc in docs {
        let doc_id = match &doc.id {
            Some(id) => id,
            None => continue,
        };
        let schema_name = match doc_schema.get(doc_id) {
            Some(name) => name,
            None => continue,
        };
        let schema_cfg = match cfg.schema.iter().find(|s| &s.name == schema_name) {
            Some(sc) => sc,
            None => continue,
        };
        let wikilinks_cfg = resolve_wikilinks_cfg(schema_cfg);
        let wikilinks_cfg = match wikilinks_cfg {
            Some(cfg) => cfg,
            None => continue,
        };
        let doc_path = doc.display_path();

        let severity = resolve_severity(
            wikilinks_cfg.severity.as_deref(),
            schema_cfg
                .validate
                .as_ref()
                .and_then(|v| v.severity.as_deref()),
        );

        if let Some(min_outgoing) = wikilinks_cfg.min_outgoing {
            if min_outgoing > 0 {
                let count = outgoing
                    .get(doc_id)
                    .map(|set| set.len())
                    .unwrap_or_default();
                if count < min_outgoing {
                    emit_with_severity(
                        &severity,
                        format!(
                            "{}: wikilinks outgoing unique targets {} below minimum {}",
                            doc_path,
                            count,
                            min_outgoing
                        ),
                        errors,
                        warnings,
                    );
                }
            }
        }

        if let Some(min_incoming) = wikilinks_cfg.min_incoming {
            if min_incoming > 0 {
                let count = incoming
                    .get(doc_id)
                    .map(|set| set.len())
                    .unwrap_or_default();
                if count < min_incoming {
                    emit_with_severity(
                        &severity,
                        format!(
                            "{}: wikilinks incoming unique referrers {} below minimum {}",
                            doc_path,
                            count,
                            min_incoming
                        ),
                        errors,
                        warnings,
                    );
                }
            }
        }
    }
}

fn resolve_wikilinks_cfg(
    schema_cfg: &SchemaCfg,
) -> Option<&crate::config::schema::SchemaWikilinksCfg> {
    schema_cfg
        .validate
        .as_ref()
        .and_then(|v| v.edges.as_ref())
        .and_then(|edges| edges.wikilinks.as_ref())
}

fn resolve_severity(wl: Option<&str>, schema_default: Option<&str>) -> String {
    if let Some(sev) = wl {
        return sev.to_ascii_lowercase();
    }
    if let Some(sev) = schema_default {
        return sev.to_ascii_lowercase();
    }
    "error".to_string()
}

fn emit_with_severity(
    severity: &str,
    message: String,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    match severity {
        "warning" | "warn" => warnings.push(message),
        "ignore" => {}
        _ => errors.push(message),
    }
}

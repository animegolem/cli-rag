use std::collections::HashMap;

use crate::config::{build_schema_sets, Config, SchemaCfg};
use crate::model::AdrDoc;

// Build a map from doc id -> schema name using filename pattern matching (first match wins).
pub fn compute_doc_schema(cfg: &Config, docs: &Vec<AdrDoc>) -> HashMap<String, String> {
    let mut doc_schema: HashMap<String, String> = HashMap::new();
    let schema_sets: Vec<(SchemaCfg, globset::GlobSet)> = build_schema_sets(cfg);
    for d in docs {
        if let Some(ref id) = d.id {
            let fname = d.file.file_name().and_then(|s| s.to_str()).unwrap_or("");
            for (sc, set) in &schema_sets {
                if set.is_match(fname) {
                    doc_schema.insert(id.clone(), sc.name.clone());
                    break;
                }
            }
        }
    }
    doc_schema
}

use std::collections::{BTreeSet, HashMap};

use crate::model::AdrDoc;

// Build id -> docs map and record missing id errors.
pub fn build_id_map(docs: &Vec<AdrDoc>, errors: &mut Vec<String>) -> HashMap<String, Vec<AdrDoc>> {
    let mut id_to_docs: HashMap<String, Vec<AdrDoc>> = HashMap::new();
    for d in docs {
        if let Some(ref id) = d.id {
            id_to_docs.entry(id.clone()).or_default().push(d.clone());
        } else {
            errors.push(format!("{}: missing id", d.file.display()));
        }
    }
    id_to_docs
}

// Detect duplicates and conflicts across docs sharing the same id.
pub fn detect_dups_conflicts(id_to_docs: &HashMap<String, Vec<AdrDoc>>, errors: &mut Vec<String>) {
    for (id, lst) in id_to_docs {
        if lst.len() > 1 {
            let mut titles: BTreeSet<String> = BTreeSet::new();
            let mut statuses: BTreeSet<String> = BTreeSet::new();
            for d in lst {
                titles.insert(d.title.clone());
                if let Some(ref s) = d.status {
                    statuses.insert(s.clone());
                }
            }
            let files = lst
                .iter()
                .map(|d| d.file.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            if titles.len() > 1 || statuses.len() > 1 {
                errors.push(format!(
                    "conflict for id {} (metadata differ) in: {}",
                    id, files
                ));
            } else {
                errors.push(format!("duplicate id {} in: {}", id, files));
            }
        }
    }
}

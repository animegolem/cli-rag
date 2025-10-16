use std::collections::BTreeSet;

use crate::model::AdrDoc;

pub fn check_references(docs: &Vec<AdrDoc>, id_set: &BTreeSet<String>, errors: &mut Vec<String>) {
    for d in docs {
        for dep in &d.depends_on {
            if !id_set.contains(dep) {
                errors.push(format!("{}: depends_on '{}' not found", d.display_path(), dep));
            }
        }
        for s in &d.supersedes {
            if !id_set.contains(s) {
                errors.push(format!("{}: supersedes '{}' not found", d.display_path(), s));
            }
        }
        for s in &d.superseded_by {
            if !id_set.contains(s) {
                errors.push(format!("{}: superseded_by '{}' not found", d.display_path(), s));
            }
        }
    }
}

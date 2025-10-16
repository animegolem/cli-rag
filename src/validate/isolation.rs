use std::collections::HashMap;

use crate::model::AdrDoc;

pub fn warn_isolated(
    docs: &Vec<AdrDoc>,
    id_to_docs: &HashMap<String, Vec<AdrDoc>>,
    warnings: &mut Vec<String>,
) {
    // Warn on isolated ADRs (no depends_on and no dependents). Valid, but highlighted.
    let mut has_dependent: HashMap<String, bool> = HashMap::new();
    for id in id_to_docs.keys() {
        has_dependent.insert(id.clone(), false);
    }
    for d in docs {
        for dep in &d.depends_on {
            if let Some(x) = has_dependent.get_mut(dep) {
                *x = true;
            }
        }
    }
    for d in docs {
        if let Some(ref id) = d.id {
            let depends = d.depends_on.is_empty();
            let depended = !has_dependent.get(id).copied().unwrap_or(false);
            if depends && depended {
                warnings.push(format!(
                    "{}: '{}' has no graph connections (valid, but isolated)",
                    d.display_path(),
                    id
                ));
            }
        }
    }
}

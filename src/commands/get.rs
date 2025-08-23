use anyhow::{Result, anyhow};
use std::fs;

use crate::config::Config;
use crate::commands::output::print_json;
use crate::discovery::load_docs;

pub fn run(cfg: &Config, format: &str, id: String, include_dependents: bool) -> Result<()> {
    let docs = load_docs(cfg)?;
    let mut by_id = std::collections::HashMap::new();
    for d in &docs { if let Some(ref i) = d.id { by_id.insert(i.clone(), d.clone()); } }
    let primary = by_id.get(&id).ok_or_else(|| anyhow!("ADR not found: {}", id))?;
    let deps: Vec<crate::model::AdrDoc> = primary.depends_on.iter().filter_map(|dep| by_id.get(dep).cloned()).collect();
    let mut dependents = Vec::new();
    if include_dependents {
        for d in &docs { if d.depends_on.iter().any(|dep| dep == &id) { dependents.push(d.clone()); } }
    }
    if format == "json" {
        let out = serde_json::json!({
            "id": id,
            "title": primary.title,
            "file": primary.file,
            "tags": primary.tags,
            "status": primary.status,
            "depends_on": deps.iter().filter_map(|d| d.id.clone()).collect::<Vec<_>>(),
            "dependents": dependents.iter().filter_map(|d| d.id.clone()).collect::<Vec<_>>(),
            "content": fs::read_to_string(&primary.file).unwrap_or_default(),
        });
        print_json(&out)?;
    } else {
        println!("# {}: {}\n", id, primary.title);
        if !primary.depends_on.is_empty() {
            println!("## Depends On");
            for d in &deps { println!("- {}: {}", d.id.clone().unwrap_or_default(), d.title); }
            println!("");
        }
        if include_dependents && !dependents.is_empty() {
            println!("## Dependents ({})", dependents.len());
            for d in &dependents { println!("- {}: {}", d.id.clone().unwrap_or_default(), d.title); }
            println!("");
        }
        let content = fs::read_to_string(&primary.file).unwrap_or_default();
        println!("## Content\n\n{}", content);
    }
    Ok(())
}

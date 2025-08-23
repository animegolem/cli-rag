use anyhow::Result;
use serde_json::Value;
use crate::commands::output::{print_json, print_ndjson_iter};

use crate::config::Config;
use crate::discovery::load_docs;

pub fn run(cfg: &Config, format: &str, query: String) -> Result<()> {
    let q = query.to_lowercase();
    let docs = load_docs(cfg)?;
    let mut hits: Vec<&crate::model::AdrDoc> = Vec::new();
    for d in &docs { let id = d.id.clone().unwrap_or_default(); if id.to_lowercase().contains(&q) || d.title.to_lowercase().contains(&q) { hits.push(d); } }
    if format == "json" {
        let arr: Vec<Value> = hits.iter().map(|d| serde_json::json!({
            "id": d.id,
            "title": d.title,
            "file": d.file,
            "tags": d.tags,
            "status": d.status,
        })).collect();
        print_json(&arr)?;
    } else if format == "ndjson" {
        let it = hits.into_iter().map(|d| serde_json::json!({
            "id": d.id,
            "title": d.title,
            "file": d.file,
            "tags": d.tags,
            "status": d.status,
        }));
        print_ndjson_iter::<serde_json::Value, _>(it)?;
    } else {
        for d in hits { println!("{}\t{}\t{}", d.id.clone().unwrap_or_default(), d.title, d.file.display()); }
    }
    Ok(())
}

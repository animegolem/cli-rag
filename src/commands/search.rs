use anyhow::Result;
use crate::commands::output::{print_json, print_ndjson_iter};
use crate::protocol::SearchResult;

use crate::config::Config;
use crate::discovery::load_docs;

pub fn run(cfg: &Config, format: &str, query: String) -> Result<()> {
    let q = query.to_lowercase();
    let docs = load_docs(cfg)?;
    let mut hits: Vec<&crate::model::AdrDoc> = Vec::new();
    for d in &docs { let id = d.id.clone().unwrap_or_default(); if id.to_lowercase().contains(&q) || d.title.to_lowercase().contains(&q) { hits.push(d); } }
    if format == "json" {
        let arr: Vec<SearchResult> = hits
            .iter()
            .filter_map(|d| d.id.as_ref().map(|id| (id, *d)))
            .map(|(id, d)| SearchResult {
                id: id.clone(),
                title: d.title.clone(),
                file: d.file.clone(),
                tags: d.tags.clone(),
                status: d.status.clone(),
                groups: d.groups.clone(),
            })
            .collect();
        print_json(&arr)?;
    } else if format == "ndjson" {
        let it = hits
            .into_iter()
            .filter_map(|d| d.id.as_ref().map(|id| (id.clone(), d)))
            .map(|(id, d)| SearchResult {
                id,
                title: d.title.clone(),
                file: d.file.clone(),
                tags: d.tags.clone(),
                status: d.status.clone(),
                groups: d.groups.clone(),
            });
        print_ndjson_iter::<SearchResult, _>(it)?;
    } else {
        for d in hits { println!("{}\t{}\t{}", d.id.clone().unwrap_or_default(), d.title, d.file.display()); }
    }
    Ok(())
}

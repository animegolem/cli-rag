use crate::cli::OutputFormat;
use crate::commands::output::{print_json, print_ndjson_iter};
use crate::protocol::SearchResult;
use anyhow::Result;

use crate::config::Config;
use crate::discovery::docs_with_source;

pub fn run(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    format: &OutputFormat,
    query: String,
) -> Result<()> {
    let q = query.to_lowercase();
    let (docs, used_unified) = docs_with_source(cfg, cfg_path)?;
    if !used_unified {
        eprintln!("Note: unified index not found; falling back to per-base/scan. Consider `cli-rag validate`.");
    }
    let mut hits: Vec<&crate::model::AdrDoc> = Vec::new();
    for d in &docs {
        let id = d.id.clone().unwrap_or_default();
        if id.to_lowercase().contains(&q) || d.title.to_lowercase().contains(&q) {
            hits.push(d);
        }
    }
    match format {
        OutputFormat::Json => {
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
        }
        OutputFormat::Ndjson => {
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
        }
        OutputFormat::Plain => {
            for d in hits {
                println!(
                    "{}\t{}\t{}",
                    d.id.clone().unwrap_or_default(),
                    d.title,
                    d.file.display()
                );
            }
        }
    }
    Ok(())
}

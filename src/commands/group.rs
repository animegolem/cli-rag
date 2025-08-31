use crate::cli::OutputFormat;
use crate::commands::output::{print_json, print_ndjson_iter, print_ndjson_value};
use crate::protocol::GroupMember;
use anyhow::Result;
use std::fs;

use crate::config::Config;
use crate::discovery::{load_docs, load_docs_unified};

pub fn run(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    format: &OutputFormat,
    topic: String,
    include_content: Option<bool>,
) -> Result<()> {
    let t = topic.to_lowercase();
    let docs = match load_docs_unified(cfg, cfg_path)? { Some(d) => d, None => load_docs(cfg)? };
    let mut matches: Vec<crate::model::AdrDoc> = docs
        .into_iter()
        .filter(|d| d.groups.iter().any(|g| g.to_lowercase().contains(&t)))
        .collect();
    matches.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    let include_content = include_content.unwrap_or(cfg.defaults.include_content);
    match format {
        OutputFormat::Json => {
            let members: Vec<GroupMember> = matches
                .iter()
                .filter_map(|d| d.id.as_ref().map(|id| (id, d)))
                .map(|(id, d)| GroupMember {
                    id: id.clone(),
                    title: d.title.clone(),
                    status: d.status.clone(),
                    groups: d.groups.clone(),
                    file: Some(d.file.clone()),
                })
                .collect();
            let mut out =
                serde_json::json!({"topic": topic, "count": members.len(), "adrs": members});
            if include_content {
                if let Some(obj) = out.as_object_mut() {
                    let contents: Vec<String> = matches
                        .iter()
                        .map(|d| fs::read_to_string(&d.file).unwrap_or_default())
                        .collect();
                    obj.insert("contents".into(), serde_json::json!(contents));
                }
            }
            print_json(&out)?;
        }
        OutputFormat::Ndjson => {
            let members: Vec<GroupMember> = matches
                .iter()
                .filter_map(|d| d.id.as_ref().map(|id| (id, d)))
                .map(|(id, d)| GroupMember {
                    id: id.clone(),
                    title: d.title.clone(),
                    status: d.status.clone(),
                    groups: d.groups.clone(),
                    file: Some(d.file.clone()),
                })
                .collect();
            let header = serde_json::json!({"topic": topic, "count": members.len()});
            print_ndjson_value(&header)?;
            print_ndjson_iter(members)?;
        }
        OutputFormat::Plain => {
            println!("# Semantic Group: {}\n", topic);
            println!("**ADR Count**: {}\n", matches.len());
            println!("## ADRs in this group");
            for d in &matches {
                println!("- {}: {}", d.id.clone().unwrap_or_default(), d.title);
            }
            if include_content {
                println!("\n## Content\n");
                for d in &matches {
                    let content = fs::read_to_string(&d.file).unwrap_or_default();
                    println!(
                        "### {}: {}\n\n{}\n",
                        d.id.clone().unwrap_or_default(),
                        d.title,
                        content
                    );
                }
            }
        }
    }
    Ok(())
}

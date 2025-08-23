use anyhow::Result;
use serde_json::Value;
use crate::commands::output::print_json;
use std::fs;

use crate::config::Config;
use crate::discovery::load_docs;

pub fn run(cfg: &Config, format: &str, topic: String, include_content: Option<bool>) -> Result<()> {
    let t = topic.to_lowercase();
    let docs = load_docs(cfg)?;
    let mut matches: Vec<crate::model::AdrDoc> = docs.into_iter().filter(|d| d.groups.iter().any(|g| g.to_lowercase().contains(&t))).collect();
    matches.sort_by(|a,b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    let include_content = include_content.unwrap_or(cfg.defaults.include_content);
    if format == "json" {
        let arr: Vec<Value> = matches.iter().map(|d| serde_json::json!({
            "id": d.id,
            "title": d.title,
            "file": d.file,
            "tags": d.tags,
            "status": d.status,
        })).collect();
        let mut out = serde_json::json!({"topic": topic, "count": arr.len(), "adrs": arr});
        if include_content {
            if let Some(obj) = out.as_object_mut() {
                let contents: Vec<String> = matches.iter().map(|d| fs::read_to_string(&d.file).unwrap_or_default()).collect();
                obj.insert("contents".into(), serde_json::json!(contents));
            }
        }
        print_json(&out)?;
    } else {
        println!("# Semantic Group: {}\n", topic);
        println!("**ADR Count**: {}\n", matches.len());
        println!("## ADRs in this group");
        for d in &matches { println!("- {}: {}", d.id.clone().unwrap_or_default(), d.title); }
        if include_content {
            println!("\n## Content\n");
            for d in &matches { let content = fs::read_to_string(&d.file).unwrap_or_default(); println!("### {}: {}\n\n{}\n", d.id.clone().unwrap_or_default(), d.title, content); }
        }
    }
    Ok(())
}

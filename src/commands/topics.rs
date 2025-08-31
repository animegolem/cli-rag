use crate::cli::OutputFormat;
use crate::commands::output::{print_json, print_ndjson_iter};
use crate::protocol::TopicCount;
use anyhow::{Context, Result};
use serde_json::Value;

use crate::config::Config;
use crate::discovery::{docs_with_source, load_docs, load_docs_unified};
use std::fs;

pub fn run(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    format: &OutputFormat,
) -> Result<()> {
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, usize> = BTreeMap::new();
    let mut used_groups_file = false;
    'outer: for b in &cfg.bases {
        let path = b.join(&cfg.groups_relative);
        if path.exists() {
            used_groups_file = true;
            let s =
                fs::read_to_string(&path).with_context(|| format!("reading groups {:?}", path))?;
            let v: Value =
                serde_json::from_str(&s).with_context(|| format!("parsing groups {:?}", path))?;
            if let Some(sections) = v.get("sections").and_then(|x| x.as_array()) {
                for sec in sections {
                    let title = sec.get("title").and_then(|x| x.as_str()).unwrap_or("");
                    let mut count = 0usize;
                    if let Some(sels) = sec.get("selectors").and_then(|x| x.as_array()) {
                        for sel in sels {
                            if let Some(ids) = sel.get("anyIds").and_then(|x| x.as_array()) {
                                count += ids.len();
                            }
                        }
                    }
                    *groups.entry(title.to_string()).or_insert(0) += count;
                }
            }
            break 'outer;
        }
    }
    if !used_groups_file {
        let (docs, used_unified) = docs_with_source(cfg, cfg_path)?;
        if !used_unified {
            eprintln!("Note: unified index not found; falling back to per-base/scan. Consider `cli-rag validate`.");
        }
        for d in docs {
            for g in d.groups {
                *groups.entry(g).or_insert(0) += 1;
            }
        }
    }
    match format {
        OutputFormat::Json => {
            let arr: Vec<TopicCount> = groups
                .into_iter()
                .map(|(topic, count)| TopicCount { topic, count })
                .collect();
            print_json(&arr)?;
        }
        OutputFormat::Ndjson => {
            let it = groups
                .into_iter()
                .map(|(topic, count)| TopicCount { topic, count });
            print_ndjson_iter::<TopicCount, _>(it)?;
        }
        OutputFormat::Plain => {
            if groups.is_empty() {
                println!("No semantic groups found");
                return Ok(());
            }
            println!("# Available Semantic Topics\n");
            for (name, count) in groups {
                println!("- {}: {} ADRs", name, count);
            }
        }
    }
    Ok(())
}

use anyhow::Result;

use crate::cli::OutputFormat;
use crate::commands::output::print_json;
use crate::config::Config;
use crate::discovery::{docs_with_source, load_docs, load_docs_unified};
use crate::graph::bfs_path;

pub fn run(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    format: &OutputFormat,
    from: String,
    to: String,
    max_depth: usize,
) -> Result<()> {
    let (docs, used_unified) = docs_with_source(cfg, cfg_path)?;
    if !used_unified {
        eprintln!("Note: unified index not found; falling back to per-base/scan. Consider `cli-rag validate`.");
    }
    let mut by_id = std::collections::HashMap::new();
    for d in &docs {
        if let Some(ref i) = d.id {
            by_id.insert(i.clone(), d.clone());
        }
    }
    let res = bfs_path(&from, &to, max_depth, &by_id);
    match format {
        OutputFormat::Json | OutputFormat::Ndjson => {
            let out = serde_json::json!({"from": from, "to": to, "path": res});
            print_json(&out)?;
        }
        OutputFormat::Plain => {
            println!("# Dependency Path: {} → {}\n", from, to);
            match res {
                Some(path) => {
                    println!("**Path Length**: {} steps\n", path.len().saturating_sub(1));
                    println!("## Path");
                    for (i, node) in path.iter().enumerate() {
                        println!("{}. {}", i + 1, node);
                    }
                }
                None => println!("No path found between {} and {}", from, to),
            }
        }
    }
    Ok(())
}

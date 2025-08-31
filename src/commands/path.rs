use anyhow::Result;

use crate::cli::OutputFormat;
use crate::commands::output::print_json;
use crate::config::Config;
use crate::discovery::docs_with_source;
use crate::graph::bfs_path;
use crate::protocol::ToolCallLocation;

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
        OutputFormat::Json | OutputFormat::Ndjson | OutputFormat::Ai => {
            // Optional locations for each hop if we can detect a mention line
            let mut locations: Vec<serde_json::Value> = Vec::new();
            if let Some(path_ids) = &res {
                for win in path_ids.windows(2) {
                    if let [a, b] = &win {
                        if let (Some(da), Some(db)) = (by_id.get(a), by_id.get(b)) {
                            let (file, needle): (&std::path::PathBuf, &str) =
                                if da.depends_on.iter().any(|x| x == b) {
                                    (&da.file, b.as_str())
                                } else {
                                    (&db.file, a.as_str())
                                };
                            let mut line: Option<u32> = None;
                            if let Ok(content) = std::fs::read_to_string(file) {
                                for (i, l) in content.lines().enumerate() {
                                    if l.contains(needle) {
                                        line = Some((i + 1) as u32);
                                        break;
                                    }
                                }
                            }
                            let loc = ToolCallLocation {
                                path: file.clone(),
                                line,
                            };
                            locations
                                .push(serde_json::to_value(&loc).unwrap_or(serde_json::json!({})));
                        }
                    }
                }
            }
            let out =
                serde_json::json!({"from": from, "to": to, "path": res, "locations": locations});
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

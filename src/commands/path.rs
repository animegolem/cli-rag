use anyhow::Result;

use crate::cli::OutputFormat;
use crate::commands::output::print_json;
use crate::config::{build_schema_sets, Config};
use crate::discovery::docs_with_source;
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
        OutputFormat::Json | OutputFormat::Ndjson | OutputFormat::Ai => {
            // Build contract-shaped output per contracts/v1/cli/path.schema.json
            if let Some(path_ids) = &res {
                // schema inference helper
                let schema_sets = build_schema_sets(cfg);
                let infer_schema = |path: &std::path::Path| -> String {
                    let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    for (sc, set) in &schema_sets {
                        if set.is_match(fname) {
                            return sc.name.clone();
                        }
                    }
                    "UNKNOWN".into()
                };
                // path nodes
                let nodes: Vec<serde_json::Value> = path_ids
                    .iter()
                    .filter_map(|pid| by_id.get(pid))
                    .map(|d| {
                        serde_json::json!({
                            "id": d.id.clone().unwrap_or_default(),
                            "title": d.title,
                            "schema": infer_schema(&d.file),
                        })
                    })
                    .collect();
                // edges along path with kind and locations
                let mut edges: Vec<serde_json::Value> = Vec::new();
                for win in path_ids.windows(2) {
                    if let [a, b] = &win {
                        if let (Some(da), Some(db)) = (by_id.get(a), by_id.get(b)) {
                            // Determine direction and kind
                            let (from_id, to_id, kind) = if da.depends_on.iter().any(|x| x == b)
                            {
                                (a.clone(), b.clone(), "depends_on".to_string())
                            } else if db.depends_on.iter().any(|x| x == a) {
                                (b.clone(), a.clone(), "depends_on".to_string())
                            } else {
                                // assume mentions; try to locate lines in either file
                                (a.clone(), b.clone(), "mentions".to_string())
                            };
                            // Collect locations only for mentions by scanning file content
                            let mut locs: Vec<serde_json::Value> = Vec::new();
                            if kind == "mentions" {
                                // Try a mentions scan in both files; record first hit each
                                let mut push_loc = |file: &std::path::Path, needle: &str| {
                                    if let Ok(content) = std::fs::read_to_string(file) {
                                        for (i, l) in content.lines().enumerate() {
                                            if l.contains(needle) {
                                                locs.push(serde_json::json!({
                                                    "path": file.display().to_string(),
                                                    "line": i as i32 + 1
                                                }));
                                                break;
                                            }
                                        }
                                    }
                                };
                                if let (Some(daid), Some(dbid)) =
                                    (da.id.as_deref(), db.id.as_deref())
                                {
                                    push_loc(&da.file, dbid);
                                    push_loc(&db.file, daid);
                                }
                            }
                            edges.push(serde_json::json!({
                                "from": from_id,
                                "to": to_id,
                                "kind": kind,
                                "locations": locs,
                            }));
                        }
                    }
                }
                let out = serde_json::json!({
                    "protocolVersion": crate::protocol::PROTOCOL_VERSION,
                    "ok": true,
                    "path": nodes,
                    "edges": edges,
                });
                print_json(&out)?;
            } else {
                let out = serde_json::json!({
                    "protocolVersion": crate::protocol::PROTOCOL_VERSION,
                    "ok": false,
                    "path": [],
                    "edges": []
                });
                print_json(&out)?;
            }
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

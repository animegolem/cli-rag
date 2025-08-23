use anyhow::Result;

use crate::config::Config;
use crate::discovery::load_docs;
use crate::graph::bfs_path;
use crate::commands::output::print_json;

pub fn run(cfg: &Config, format: &str, from: String, to: String, max_depth: usize) -> Result<()> {
    let docs = load_docs(cfg)?;
    let mut by_id = std::collections::HashMap::new();
    for d in &docs { if let Some(ref i) = d.id { by_id.insert(i.clone(), d.clone()); } }
    let res = bfs_path(&from, &to, max_depth, &by_id);
    if format == "json" {
        let out = serde_json::json!({"from": from, "to": to, "path": res});
        print_json(&out)?;
    } else {
        println!("# Dependency Path: {} → {}\n", from, to);
        match res {
            Some(path) => {
                println!("**Path Length**: {} steps\n", path.len().saturating_sub(1));
                println!("## Path");
                for (i, node) in path.iter().enumerate() { println!("{}. {}", i+1, node); }
            }
            None => println!("No path found between {} and {}", from, to),
        }
    }
    Ok(())
}

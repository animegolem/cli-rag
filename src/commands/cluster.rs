use anyhow::Result;
use serde_json::Value;
use crate::commands::output::print_json;

use crate::config::Config;
use crate::discovery::load_docs;
use crate::graph::compute_cluster;

pub fn run(cfg: &Config, format: &str, id: String, depth: Option<usize>, include_bidirectional: Option<bool>) -> Result<()> {
    let docs = load_docs(cfg)?;
    let depth = depth.unwrap_or(cfg.defaults.depth);
    let include_bidirectional = include_bidirectional.unwrap_or(cfg.defaults.include_bidirectional);
    let mut by_id = std::collections::HashMap::new();
    for d in &docs { if let Some(ref i) = d.id { by_id.insert(i.clone(), d.clone()); } }
    let cluster = compute_cluster(&id, depth, include_bidirectional, &by_id);
    if format == "json" {
        let arr: Vec<Value> = cluster.iter().map(|(oid, d)| serde_json::json!({
            "id": oid,
            "title": d.title,
            "status": d.status,
        })).collect();
        let out = serde_json::json!({"id": id, "size": cluster.len(), "members": arr});
        print_json(&out)?;
    } else {
        println!("# Dependency Cluster for {}\n", id);
        println!("**Cluster Size**: {}\n", cluster.len());
        println!("## Members");
        for (oid, d) in &cluster { println!("- {}: {}", oid, d.title); }
    }
    Ok(())
}

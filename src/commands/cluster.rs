use crate::cli::OutputFormat;
use crate::commands::output::{print_json, print_ndjson_iter, print_ndjson_value};
use crate::protocol::ClusterMember;
use anyhow::Result;

use crate::config::Config;
use crate::discovery::load_docs;
use crate::graph::compute_cluster;

pub fn run(
    cfg: &Config,
    format: &OutputFormat,
    id: String,
    depth: Option<usize>,
    include_bidirectional: Option<bool>,
) -> Result<()> {
    let docs = load_docs(cfg)?;
    let depth = depth.unwrap_or(cfg.defaults.depth);
    let include_bidirectional = include_bidirectional.unwrap_or(cfg.defaults.include_bidirectional);
    let mut by_id = std::collections::HashMap::new();
    for d in &docs {
        if let Some(ref i) = d.id {
            by_id.insert(i.clone(), d.clone());
        }
    }
    let cluster = compute_cluster(&id, depth, include_bidirectional, &by_id);
    match format {
        OutputFormat::Json => {
            let members: Vec<ClusterMember> = cluster
                .iter()
                .map(|(oid, d)| ClusterMember {
                    id: oid.clone(),
                    title: d.title.clone(),
                    status: d.status.clone(),
                    groups: d.groups.clone(),
                })
                .collect();
            let out = serde_json::json!({"root": id, "size": cluster.len(), "members": members});
            print_json(&out)?;
        }
        OutputFormat::Ndjson => {
            let header = serde_json::json!({"root": id, "count": cluster.len()});
            print_ndjson_value(&header)?;
            let members = cluster.iter().map(|(oid, d)| ClusterMember {
                id: oid.clone(),
                title: d.title.clone(),
                status: d.status.clone(),
                groups: d.groups.clone(),
            });
            print_ndjson_iter(members)?;
        }
        OutputFormat::Plain => {
            println!("# Dependency Cluster for {}\n", id);
            println!("**Cluster Size**: {}\n", cluster.len());
            println!("## Members");
            for (oid, d) in &cluster {
                println!("- {}: {}", oid, d.title);
            }
        }
    }
    Ok(())
}

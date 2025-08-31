use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::cli::GraphFormat;
use crate::commands::output::print_json;
use crate::config::Config;
use crate::discovery::docs_with_source;
use crate::graph::compute_cluster;
use crate::model::AdrDoc;

#[derive(Debug, Clone)]
struct Edge {
    from: String,
    to: String,
}

fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn cluster_edges(cluster: &BTreeMap<String, AdrDoc>) -> Vec<Edge> {
    let members: BTreeSet<String> = cluster.keys().cloned().collect();
    let mut edges = Vec::new();
    for (id, doc) in cluster.iter() {
        for dep in &doc.depends_on {
            if members.contains(dep) {
                edges.push(Edge {
                    from: id.clone(),
                    to: dep.clone(),
                });
            }
        }
    }
    edges
}

pub(crate) fn render_mermaid(cluster: &BTreeMap<String, AdrDoc>) -> String {
    let mut out = String::from("flowchart LR\n");
    // Node declarations
    for (id, doc) in cluster.iter() {
        let var = sanitize_id(id);
        let label = format!("{}: {}", id, doc.title.replace('"', "\\\""));
        out.push_str(&format!("  {}[\"{}\"]\n", var, label));
    }
    // Edges
    for e in cluster_edges(cluster) {
        let from = sanitize_id(&e.from);
        let to = sanitize_id(&e.to);
        out.push_str(&format!("  {} --> {}\n", from, to));
    }
    out
}

pub(crate) fn render_dot(cluster: &BTreeMap<String, AdrDoc>) -> String {
    let mut out = String::from("digraph {\n");
    // Nodes
    for (id, doc) in cluster.iter() {
        let label = format!("{}: {}", id, doc.title.replace('"', "\\\""));
        out.push_str(&format!("  \"{}\" [label=\"{}\"];\n", id, label));
    }
    // Edges
    for e in cluster_edges(cluster) {
        out.push_str(&format!("  \"{}\" -> \"{}\";\n", e.from, e.to));
    }
    out.push_str("}\n");
    out
}

pub fn run(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    format: &GraphFormat,
    id: String,
    depth: Option<usize>,
    include_bidirectional: Option<bool>,
) -> Result<()> {
    let (docs, used_unified) = docs_with_source(cfg, cfg_path)?;
    if !used_unified {
        eprintln!("Note: unified index not found; falling back to per-base/scan. Consider `cli-rag validate`.");
    }
    let depth = depth.unwrap_or(cfg.defaults.depth);
    let include_bidirectional = include_bidirectional.unwrap_or(cfg.defaults.include_bidirectional);
    let mut by_id: HashMap<String, AdrDoc> = HashMap::new();
    for d in &docs {
        if let Some(ref i) = d.id {
            by_id.insert(i.clone(), d.clone());
        }
    }
    if !by_id.contains_key(&id) {
        return Err(anyhow!("ADR not found: {}", id));
    }
    let cluster = compute_cluster(&id, depth, include_bidirectional, &by_id);
    match format {
        GraphFormat::Json => {
            let members: Vec<serde_json::Value> = cluster
                .iter()
                .map(|(oid, d)| {
                    serde_json::json!({
                        "id": oid,
                        "title": d.title,
                        "status": d.status,
                    })
                })
                .collect();
            let edges: Vec<serde_json::Value> = cluster_edges(&cluster)
                .into_iter()
                .map(|e| serde_json::json!({"from": e.from, "to": e.to}))
                .collect();
            let out = serde_json::json!({
                "root": id,
                "members": members,
                "edges": edges,
                "depth": depth,
                "bidirectional": include_bidirectional,
            });
            print_json(&out)?;
        }
        GraphFormat::Dot => {
            let s = render_dot(&cluster);
            println!("{}", s);
        }
        GraphFormat::Mermaid => {
            let s = render_mermaid(&cluster);
            println!("{}", s);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn doc(id: &str, title: &str, deps: Vec<&str>) -> AdrDoc {
        AdrDoc {
            file: PathBuf::from(format!("{}.md", id)),
            id: Some(id.to_string()),
            title: title.to_string(),
            tags: vec![],
            status: None,
            groups: vec![],
            depends_on: deps.into_iter().map(|s| s.to_string()).collect(),
            supersedes: vec![],
            superseded_by: vec![],
            fm: BTreeMap::new(),
            mtime: None,
            size: None,
        }
    }

    #[test]
    fn test_render_mermaid_and_dot() {
        let mut cluster: BTreeMap<String, AdrDoc> = BTreeMap::new();
        cluster.insert("ADR-001".into(), doc("ADR-001", "Root", vec!["ADR-002"]));
        cluster.insert("ADR-002".into(), doc("ADR-002", "Child", vec![]));
        let mm = render_mermaid(&cluster);
        assert!(mm.contains("flowchart LR"));
        assert!(mm.contains("ADR_001 --> ADR_002"));
        let dot = render_dot(&cluster);
        assert!(dot.contains("digraph"));
        assert!(dot.contains("\"ADR-001\" -> \"ADR-002\""));
    }
}

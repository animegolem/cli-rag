use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

use crate::config::Config;

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    let mut s = String::with_capacity(out.len() * 2);
    for b in out {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

pub fn run(
    cfg: &Config,
    cfg_path: &Option<PathBuf>,
    edges_opt: Option<Vec<String>>,
    min_cluster_size: usize,
    schema_filter: Option<String>,
    output: PathBuf,
) -> Result<()> {
    let cfg_dir = cfg_path.as_ref().and_then(|p| p.parent()).ok_or_else(|| {
        anyhow!("Cannot locate config directory; pass --config and run `validate` first")
    })?;
    let index_path = cfg_dir.join(&cfg.index_relative);
    if !index_path.exists() {
        return Err(anyhow!(
            "Unified index not found at {}. Run `cli-rag validate` first.",
            index_path.display()
        ));
    }
    let data = std::fs::read(&index_path)
        .with_context(|| format!("reading unified index {}", index_path.display()))?;
    let source_hash = format!("sha256:{}", sha256_hex(&data));
    let root: Value = serde_json::from_slice(&data)
        .with_context(|| format!("parsing unified index {}", index_path.display()))?;

    let nodes_v = root
        .get("nodes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Invalid unified index: missing nodes[]"))?;
    let edges_v = root
        .get("edges")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow!("Invalid unified index: missing edges[]"))?;

    // Build id -> schema map and the initial vertex set
    let mut id_schema: HashMap<String, String> = HashMap::new();
    for n in nodes_v {
        if let (Some(id), Some(schema)) = (
            n.get("id").and_then(|v| v.as_str()),
            n.get("schema").and_then(|v| v.as_str()),
        ) {
            id_schema.insert(id.to_string(), schema.to_string());
        }
    }
    // Edge kinds to include
    let mut edge_kinds: HashSet<String> = HashSet::new();
    if let Some(list) = edges_opt {
        for k in list {
            edge_kinds.insert(k);
        }
    } else {
        edge_kinds.insert("depends_on".into());
        edge_kinds.insert("mentions".into());
    }

    // Optional schema filter
    let schema_filter = schema_filter.map(|s| s.to_string());

    // Build undirected graph: adjacency and undirected edge set for density
    let mut vertices: BTreeSet<String> = BTreeSet::new();
    let mut adj: HashMap<String, BTreeSet<String>> = HashMap::new();
    let mut undirected: BTreeSet<(String, String)> = BTreeSet::new();

    // Seed vertices based on schema filter
    for (id, sc) in &id_schema {
        if let Some(sf) = &schema_filter {
            if sc != sf {
                continue;
            }
        }
        vertices.insert(id.clone());
        adj.entry(id.clone()).or_default();
    }

    for e in edges_v {
        let kind = e.get("kind").and_then(|v| v.as_str()).unwrap_or("");
        if !edge_kinds.contains(kind) {
            continue;
        }
        let from = match e.get("from").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => continue,
        };
        let to = match e.get("to").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => continue,
        };
        // Respect schema filter
        if let Some(sf) = &schema_filter {
            let sf_from = id_schema.get(from).map(|s| s == sf).unwrap_or(false);
            let sf_to = id_schema.get(to).map(|s| s == sf).unwrap_or(false);
            if !(sf_from && sf_to) {
                continue;
            }
        }
        // Only add if both endpoints are known vertices
        if !(vertices.contains(from) && vertices.contains(to)) {
            continue;
        }
        // Undirected pair with stable ordering
        let (a, b) = if from <= to { (from, to) } else { (to, from) };
        undirected.insert((a.to_string(), b.to_string()));
        adj.entry(from.to_string())
            .or_default()
            .insert(to.to_string());
        adj.entry(to.to_string())
            .or_default()
            .insert(from.to_string());
    }

    // Connected components over vertices that appear (including isolated ones)
    let mut visited: HashSet<String> = HashSet::new();
    let mut clusters: Vec<Vec<String>> = Vec::new();
    for v in vertices.iter() {
        if visited.contains(v) {
            continue;
        }
        // BFS from v
        let mut q = vec![v.clone()];
        visited.insert(v.clone());
        let mut comp: Vec<String> = Vec::new();
        while let Some(cur) = q.pop() {
            comp.push(cur.clone());
            if let Some(nei) = adj.get(&cur) {
                for n in nei {
                    if !visited.contains(n) {
                        visited.insert(n.clone());
                        q.push(n.clone());
                    }
                }
            }
        }
        // Only keep clusters meeting the minimum size
        if comp.len() >= min_cluster_size {
            comp.sort();
            clusters.push(comp);
        }
    }

    // Deterministic order: by smallest member ID
    clusters.sort_by(|a, b| a[0].cmp(&b[0]));

    // Representatives and metrics
    #[derive(Clone)]
    struct ClusterOut {
        cluster_id: String,
        members: Vec<String>,
        representatives: Vec<String>,
        size: usize,
        density: f64,
    }
    let mut out_clusters: Vec<ClusterOut> = Vec::new();
    for (idx, members) in clusters.iter().enumerate() {
        // degree within cluster
        let mut deg: BTreeMap<String, usize> = BTreeMap::new();
        for m in members {
            let d = adj
                .get(m)
                .map(|s| s.intersection(&members.iter().cloned().collect()).count())
                .unwrap_or(0);
            deg.insert(m.clone(), d);
        }
        // representatives: top 2 by degree desc then id asc
        let mut reps: Vec<(String, usize)> = deg.iter().map(|(k, v)| (k.clone(), *v)).collect();
        reps.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let representatives: Vec<String> = reps.into_iter().take(2).map(|p| p.0).collect();
        // density: undirected edges within members / (n*(n-1)/2)
        let n = members.len();
        let mut edge_count = 0usize;
        if n >= 2 {
            let set_members: BTreeSet<String> = members.iter().cloned().collect();
            for (a, b) in undirected.iter() {
                if set_members.contains(a) && set_members.contains(b) {
                    edge_count += 1;
                }
            }
        }
        let denom = if n < 2 { 1.0 } else { (n * (n - 1) / 2) as f64 };
        let density = if n < 2 {
            0.0
        } else {
            (edge_count as f64) / denom
        };
        let cluster_id = format!("c_{:04}", idx + 1);
        out_clusters.push(ClusterOut {
            cluster_id,
            members: members.clone(),
            representatives,
            size: n,
            density,
        });
    }

    // Build JSON output per contract
    let edges_sorted: BTreeSet<String> = edge_kinds.iter().cloned().collect();
    let schema_val = match &schema_filter {
        Some(s) => serde_json::Value::String(s.clone()),
        None => serde_json::Value::Null,
    };
    let params = serde_json::json!({
        "edges": edges_sorted,
        "minClusterSize": min_cluster_size as i64,
        "schema": schema_val,
    });
    let clusters_json: Vec<Value> = out_clusters
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "clusterId": c.cluster_id,
                "members": c.members,
                "representatives": c.representatives,
                "metrics": { "size": c.size as i64, "density": c.density },
                "label": "",
                "summary": "",
                "tags": [],
            })
        })
        .collect();
    let out = serde_json::json!({
        "version": 1,
        "generatedAt": Utc::now().to_rfc3339(),
        "sourceIndexHash": source_hash,
        "params": params,
        "clusters": clusters_json,
    });
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&output, serde_json::to_string_pretty(&out)?)
        .with_context(|| format!("writing plan to {}", output.display()))?;
    eprintln!("Wrote AI index plan: {}", output.display());
    Ok(())
}

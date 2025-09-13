use anyhow::{Context, Result};

use crate::config::Config;
use crate::model::AdrDoc;

use super::per_base::load_docs as load_docs_legacy;

/// Attempt to load a unified index located at the config directory joined with
/// `cfg.index_relative`. Pass the full config path if known.
pub fn load_docs_unified(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
) -> Result<Option<Vec<AdrDoc>>> {
    use serde_json::Value;
    let cfg_dir = match cfg_path.as_ref().and_then(|p| p.parent()) {
        Some(d) => d,
        None => return Ok(None),
    };
    let unified_path = cfg_dir.join(&cfg.index_relative);
    if !unified_path.exists() {
        return Ok(None);
    }
    let data = std::fs::read_to_string(&unified_path)
        .with_context(|| format!("reading unified index {:?}", unified_path))?;
    let root: Value = serde_json::from_str(&data)
        .with_context(|| format!("parsing unified index {:?}", unified_path))?;
    // New unified index format: {version, generatedAt, docCount, nodes[], edges[]}
    if let (Some(nodes), Some(edges)) = (root.get("nodes"), root.get("edges")) {
        use std::collections::HashMap;
        let nodes = nodes.as_array().cloned().unwrap_or_default();
        let edges = edges.as_array().cloned().unwrap_or_default();
        // Build depends_on from edges of kind "depends_on"
        let mut deps_map: HashMap<String, Vec<String>> = HashMap::new();
        for e in edges {
            let kind = e.get("kind").and_then(|v| v.as_str()).unwrap_or("");
            if kind != "depends_on" {
                continue;
            }
            if let (Some(from), Some(to)) = (
                e.get("from").and_then(|v| v.as_str()),
                e.get("to").and_then(|v| v.as_str()),
            ) {
                deps_map
                    .entry(from.to_string())
                    .or_default()
                    .push(to.to_string());
            }
        }
        let mut out_docs = Vec::new();
        for n in nodes {
            let id = n.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
            let title = n
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let path_rel = n.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let file = cfg_dir.join(path_rel);
            // Derive tags/status from frontmatter if present
            let mut fm_map: std::collections::BTreeMap<String, serde_yaml::Value> =
                std::collections::BTreeMap::new();
            let (tags, status) = match n.get("frontmatter") {
                Some(Value::Object(obj)) => {
                    // tags/status convenience
                    let tags = obj
                        .get("tags")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_else(Vec::new);
                    let status = obj
                        .get("status")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    // populate fm_map with simple JSON→YAML conversion for strings/arrays
                    for (k, v) in obj {
                        let yv = match v {
                            Value::String(s) => serde_yaml::Value::from(s.clone()),
                            Value::Array(a) => {
                                let mut seq = Vec::new();
                                for it in a {
                                    if let Some(ss) = it.as_str() {
                                        seq.push(serde_yaml::Value::from(ss.to_string()));
                                    }
                                }
                                serde_yaml::Value::Sequence(seq)
                            }
                            _ => serde_yaml::Value::Null,
                        };
                        fm_map.insert(k.clone(), yv);
                    }
                    (tags, status)
                }
                _ => (Vec::new(), None),
            };
            let id_str = id.clone().unwrap_or_default();
            out_docs.push(AdrDoc {
                file,
                id,
                title,
                tags,
                status,
                groups: Vec::new(),
                depends_on: deps_map.remove(&id_str).unwrap_or_default(),
                supersedes: Vec::new(),
                superseded_by: Vec::new(),
                fm: fm_map,
                mtime: None,
                size: None,
            });
        }
        return Ok(Some(out_docs));
    }
    // Legacy unified index with {items:[]}
    let mut docs = Vec::new();
    let items = root
        .get("items")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for it in items {
        let file_str = it.get("file").and_then(|v| v.as_str()).unwrap_or("");
        if file_str.is_empty() {
            continue;
        }
        let file = std::path::PathBuf::from(file_str);
        let id = it.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
        let title = it
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let tags = it
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(Vec::new);
        let status = it
            .get("status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let groups = it
            .get("groups")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(Vec::new);
        let depends_on = it
            .get("depends_on")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_else(Vec::new);
        let supersedes = match it.get("supersedes") {
            Some(Value::Array(a)) => a
                .iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect(),
            Some(Value::String(s)) => vec![s.to_string()],
            _ => Vec::new(),
        };
        let superseded_by = match it.get("superseded_by") {
            Some(Value::Array(a)) => a
                .iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect(),
            Some(Value::String(s)) => vec![s.to_string()],
            _ => Vec::new(),
        };
        let mtime = it.get("mtime").and_then(|v| v.as_u64());
        let size = it.get("size").and_then(|v| v.as_u64());
        docs.push(AdrDoc {
            file,
            id,
            title,
            tags,
            status,
            groups,
            depends_on,
            supersedes,
            superseded_by,
            fm: std::collections::BTreeMap::new(),
            mtime,
            size,
        });
    }
    Ok(Some(docs))
}

/// Helper to prefer unified index and indicate which path was used.
pub fn docs_with_source(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
) -> Result<(Vec<AdrDoc>, bool)> {
    if let Some(d) = load_docs_unified(cfg, cfg_path)? {
        Ok((d, true))
    } else {
        Ok((load_docs_legacy(cfg)?, false))
    }
}

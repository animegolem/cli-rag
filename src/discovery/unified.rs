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

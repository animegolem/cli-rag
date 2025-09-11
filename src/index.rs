use anyhow::{Context, Result};
use chrono::Utc;
use regex::Regex;
use serde_json::json;
use std::fs;
// no direct std::time imports needed here

use crate::config::{build_schema_sets, Config};
use crate::model::AdrDoc;

pub fn write_indexes(
    cfg: &Config,
    docs: &Vec<AdrDoc>,
    _force: bool,
    _auto_write: bool,
    config_dir: Option<&std::path::Path>,
) -> Result<()> {
    // Write a single authoritative unified index in the config directory
    let dir = match config_dir {
        Some(d) => d,
        None => {
            // If we don't know the config dir, do nothing (validation dry-run)
            return Ok(());
        }
    };

    // Precompute schema by filename matcher
    let schema_sets = build_schema_sets(cfg);
    let fname_schema = |path: &std::path::Path| -> String {
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        for (sc, set) in &schema_sets {
            if set.is_match(fname) {
                return sc.name.clone();
            }
        }
        "UNKNOWN".into()
    };

    // Helper for relative path string under project root
    let rel_to_root = |path: &std::path::Path| -> String {
        path.strip_prefix(dir)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
    };

    // Nodes
    let mut nodes: Vec<serde_json::Value> = Vec::new();
    // Edges (FM-derived)
    let mut edges: Vec<serde_json::Value> = Vec::new();

    // Mentions regex: [[ID]] where ID has at least one dash
    let mention_re: Regex = Regex::new(r"\[\[([A-Za-z]+-[0-9A-Za-z_-]+)\]\]").unwrap();

    for d in docs {
        let id = match &d.id {
            Some(s) => s.clone(),
            None => continue, // skip unnamed docs for contract nodes
        };
        let schema = fname_schema(&d.file);
        let path_str = rel_to_root(&d.file);
        // frontmatter map (keys only or values if present)
        let mut fm_obj = serde_json::Map::new();
        for (k, v) in &d.fm {
            // Try to round-trip serde_yaml::Value → serde_json::Value; fallback to null
            let jv = match serde_json::to_value(v) {
                Ok(val) => val,
                Err(_) => serde_json::Value::Null,
            };
            fm_obj.insert(k.clone(), jv);
        }
        let frontmatter = serde_json::Value::Object(fm_obj);
        nodes.push(json!({
            "id": id,
            "schema": schema,
            "title": d.title,
            "path": path_str,
            "frontmatter": frontmatter,
        }));

        // depends_on edges
        if let Some(from) = &d.id {
            for dep in &d.depends_on {
                edges.push(json!({"from": from, "to": dep, "kind": "depends_on"}));
            }
            for s in &d.supersedes {
                edges.push(json!({"from": from, "to": s, "kind": "supersedes"}));
            }
            for sb in &d.superseded_by {
                edges.push(json!({"from": from, "to": sb, "kind": "superseded_by"}));
            }
        }

        // Scan file content for wikilink mentions and record locations
        if let Ok(content) = fs::read_to_string(&d.file) {
            let mut seen_on_line: std::collections::HashSet<(String, u32)> =
                std::collections::HashSet::new();
            for (i, line) in content.lines().enumerate() {
                for cap in mention_re.captures_iter(line) {
                    if let Some(m) = cap.get(1) {
                        let target = m.as_str().to_string();
                        let line_no = (i + 1) as u32;
                        // dedupe same target on same line
                        if seen_on_line.insert((target.clone(), line_no)) {
                            edges.push(json!({
                                "from": &id,
                                "to": target,
                                "kind": "mentions",
                                "locations": [{"path": path_str, "line": line_no}]
                            }));
                        }
                    }
                }
            }
        }
    }

    // Root object
    let out = json!({
        "version": 1,
        "generatedAt": Utc::now().to_rfc3339(),
        "docCount": nodes.len(),
        "nodes": nodes,
        "edges": edges,
    });

    let out_path = dir.join(&cfg.index_relative);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&out_path, serde_json::to_string_pretty(&out)?)
        .with_context(|| format!("writing unified index to {}", out_path.display()))?;
    eprintln!(
        "Wrote unified index: {} ({} nodes, {} edges)",
        out_path.display(),
        out.get("nodes")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0),
        out.get("edges")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0)
    );
    Ok(())
}

pub fn write_groups_config(cfg: &Config, docs: &Vec<AdrDoc>) -> Result<()> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut by_group: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for d in docs {
        if let Some(ref id) = d.id {
            for g in &d.groups {
                by_group.entry(g.clone()).or_default().insert(id.clone());
            }
        }
    }
    let mut sections = Vec::new();
    for (title, ids) in by_group {
        sections.push(serde_json::json!({ "title": title, "selectors": [ { "anyIds": ids.into_iter().collect::<Vec<_>>() } ] }));
    }
    for base in &cfg.bases {
        let out_path = base.join(&cfg.groups_relative);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        let body = serde_json::json!({ "sections": sections });
        fs::write(&out_path, serde_json::to_string_pretty(&body)?)
            .with_context(|| format!("writing groups to {}", out_path.display()))?;
        eprintln!("Wrote groups: {}", out_path.display());
    }
    Ok(())
}

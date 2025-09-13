use anyhow::{anyhow, Result};
use std::collections::BTreeSet;
use std::fs;

use crate::cli::OutputFormat;
use crate::commands::output::print_json;
use crate::config::Config;
use crate::discovery::docs_with_source;
use crate::config::build_schema_sets;
use crate::protocol::ContentBlock;

fn build_outline(path: &std::path::Path, lines_per_heading: usize) -> serde_json::Value {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let mut outline: Vec<serde_json::Value> = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut buffer: Vec<String> = Vec::new();
    let mut flush = |heading: &mut Option<String>, buf: &mut Vec<String>, out: &mut Vec<serde_json::Value>| {
        if let Some(h) = heading.take() {
            let first: Vec<String> = buf.iter().take(lines_per_heading).cloned().collect();
            out.push(serde_json::json!({"heading": h, "firstLines": first}));
            buf.clear();
        }
    };
    for line in content.lines() {
        let lt = line.trim_start();
        if lt.starts_with('#') {
            // New heading
            flush(&mut current_heading, &mut buffer, &mut outline);
            // Capture heading text after hashes and space
            let head = lt.trim_start_matches('#').trim_start().to_string();
            current_heading = Some(head);
        } else if current_heading.is_some() {
            buffer.push(line.to_string());
        }
    }
    // Flush last
    {
        let _ = flush(&mut current_heading, &mut buffer, &mut outline);
    }
    serde_json::Value::Array(outline)
}

pub fn run(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    format: &OutputFormat,
    id: String,
    include_dependents: bool,
    neighbor_style: Option<String>,
    depth: Option<usize>,
    max_fanout: Option<usize>,
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
    let primary = by_id
        .get(&id)
        .ok_or_else(|| anyhow!("ADR not found: {}", id))?;
    let deps: Vec<crate::model::AdrDoc> = primary
        .depends_on
        .iter()
        .filter_map(|dep| by_id.get(dep).cloned())
        .collect();
    let mut dependents = Vec::new();
    if include_dependents {
        for d in &docs {
            if d.depends_on.iter().any(|dep| dep == &id) {
                dependents.push(d.clone());
            }
        }
    }
    match format {
        OutputFormat::Json | OutputFormat::Ndjson => {
            // Emit ai_get contract-shaped JSON
            let protocol_version = crate::protocol::PROTOCOL_VERSION;
            let retrieval_version = 1;
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
            let root_schema = infer_schema(&primary.file);
            let path_str = primary.file.to_string_lossy().to_string();
            // frontmatter to JSON object
            let mut fm_obj = serde_json::Map::new();
            for (k, v) in &primary.fm {
                let jv = match serde_json::to_value(v) {
                    Ok(val) => val,
                    Err(_) => serde_json::Value::Null,
                };
                fm_obj.insert(k.clone(), jv);
            }
            // GTD hints on root
            let kanban_status = primary
                .fm
                .get("kanban_status")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let kanban_status_line = primary
                .fm
                .get("kanban_statusline")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let due_date = primary
                .fm
                .get("due_date")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Neighbor parameters and policy
            let style = neighbor_style
                .as_deref()
                .map(|s| s.to_lowercase())
                .unwrap_or_else(|| "metadata".to_string());
            let depth = depth.unwrap_or(1);
            let max_fanout = max_fanout.unwrap_or(5);
            if style == "full" && depth > 1 {
                eprintln!("Policy violation: neighborStyle=full with depth>1 (NEIGHBORS_FULL_DEPTH_GT1)");
                std::process::exit(2);
            }

            // Build adjacency maps for BFS
            let mut by_id = std::collections::HashMap::new();
            for d in &docs {
                if let Some(ref i) = d.id {
                    by_id.insert(i.clone(), d.clone());
                }
            }
            let mut out_edges: std::collections::HashMap<String, Vec<(String, &'static str)>> =
                std::collections::HashMap::new();
            let mut in_edges: std::collections::HashMap<String, Vec<(String, &'static str)>> =
                std::collections::HashMap::new();
            for d in &docs {
                if let Some(ref from) = d.id {
                    for dep in &d.depends_on {
                        out_edges
                            .entry(from.clone())
                            .or_default()
                            .push((dep.clone(), "depends_on"));
                        in_edges
                            .entry(dep.clone())
                            .or_default()
                            .push((from.clone(), "dependent"));
                    }
                }
            }
            // BFS
            let mut q: std::collections::VecDeque<(String, usize)> = std::collections::VecDeque::new();
            let mut dist: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
            let mut discovered_from: std::collections::HashMap<String, (String, String)> =
                std::collections::HashMap::new(); // neighbor -> (from, edge)
            dist.insert(id.clone(), 0);
            q.push_back((id.clone(), 0));
            while let Some((cur, dlevel)) = q.pop_front() {
                if dlevel >= depth { continue; }
                // explore both directions
                for (nbr, edge) in out_edges.get(&cur).cloned().unwrap_or_default() {
                    if !dist.contains_key(&nbr) {
                        dist.insert(nbr.clone(), dlevel + 1);
                        discovered_from.insert(nbr.clone(), (cur.clone(), edge.to_string()));
                        q.push_back((nbr, dlevel + 1));
                    }
                }
                for (nbr, edge) in in_edges.get(&cur).cloned().unwrap_or_default() {
                    if !dist.contains_key(&nbr) {
                        dist.insert(nbr.clone(), dlevel + 1);
                        discovered_from.insert(nbr.clone(), (cur.clone(), edge.to_string()));
                        q.push_back((nbr, dlevel + 1));
                    }
                }
            }

            // Materialize neighbors (exclude root)
            let mut neighbors: Vec<serde_json::Value> = Vec::new();
            let mut seen: BTreeSet<String> = BTreeSet::new();
            for (nid, dlevel) in dist.iter() {
                if nid == &id { continue; }
                if let Some(d) = by_id.get(nid) {
                    if !seen.insert(nid.clone()) { continue; }
                    let (from, edge) = discovered_from
                        .get(nid)
                        .cloned()
                        .unwrap_or((id.clone(), "depends_on".to_string()));
                    let mut obj = serde_json::json!({
                        "id": nid,
                        "title": d.title,
                        "schema": infer_schema(&d.file),
                        "path": d.file.to_string_lossy().to_string(),
                        "distance": *dlevel as i64,
                        "discoveredFrom": from,
                        "edge": edge,
                        "status": d.status,
                        "tags": d.tags,
                        "kanbanStatus": d.fm.get("kanban_status").and_then(|v| v.as_str()),
                        "kanbanStatusLine": d.fm.get("kanban_statusline").and_then(|v| v.as_str()),
                        "dueDate": d.fm.get("due_date").and_then(|v| v.as_str()),
                        "lastModified": serde_json::Value::Null,
                        "score": serde_json::Value::Null,
                    });
                    if let Ok(md) = fs::metadata(&d.file) {
                        if let Ok(m) = md.modified() {
                            let dt: chrono::DateTime<chrono::Utc> = m.into();
                            obj["lastModified"] = serde_json::json!(dt.to_rfc3339());
                        }
                    }
                    // Style handling for neighbor content fields
                    if style == "outline" {
                        let outline = build_outline(&d.file, 2);
                        obj.as_object_mut()
                            .unwrap()
                            .insert("contentOutline".to_string(), outline);
                    } else if style == "full" {
                        let body = fs::read_to_string(&d.file).unwrap_or_default();
                        obj.as_object_mut().unwrap().insert(
                            "content".to_string(),
                            serde_json::json!([{"type":"text","text": body}]),
                        );
                    }
                    neighbors.push(obj);
                }
            }

            // Deterministic ordering: distance asc → score desc → lastModified desc → id asc
            neighbors.sort_by(|a, b| {
                let da = a["distance"].as_i64().unwrap_or(0);
                let db = b["distance"].as_i64().unwrap_or(0);
                da.cmp(&db)
                    .then_with(|| {
                        let sa = a["score"].as_f64().unwrap_or(f64::NEG_INFINITY);
                        let sb = b["score"].as_f64().unwrap_or(f64::NEG_INFINITY);
                        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .then_with(|| {
                        let la = a["lastModified"].as_str();
                        let lb = b["lastModified"].as_str();
                        match (la, lb) {
                            (Some(aa), Some(bb)) => bb.cmp(aa),
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                            _ => std::cmp::Ordering::Equal,
                        }
                    })
                    .then_with(|| a["id"].as_str().unwrap_or("").cmp(b["id"].as_str().unwrap_or("")))
            });

            // Apply fanout limit after sort
            if neighbors.len() > max_fanout {
                neighbors.truncate(max_fanout);
            }
            let content_text = fs::read_to_string(&primary.file).unwrap_or_default();
            let out = serde_json::json!({
                "protocolVersion": protocol_version,
                "retrievalVersion": retrieval_version,
                "id": id,
                "schema": root_schema,
                "title": primary.title,
                "file": path_str,
                "frontmatter": serde_json::Value::Object(fm_obj),
                "kanbanStatus": kanban_status,
                "kanbanStatusLine": kanban_status_line,
                "dueDate": due_date,
                "content": [
                    {"type": "text", "text": content_text}
                ],
                "neighbors": neighbors,
                "limits": {"depth": depth as i64, "maxFanout": max_fanout as i64}
            });
            print_json(&out)?;
        }
        OutputFormat::Ai => {
            let mut blocks: Vec<ContentBlock> = Vec::new();
            let path_str = primary.file.to_string_lossy().to_string();
            blocks.push(ContentBlock::ResourceLink {
                uri: path_str.clone(),
                description: None,
                mime_type: None,
                title: Some(primary.title.clone()),
                annotations: None,
            });
            let content = fs::read_to_string(&primary.file).unwrap_or_default();
            blocks.push(ContentBlock::Text {
                text: content,
                annotations: None,
            });
            let out = serde_json::json!({
                "id": id,
                "title": primary.title,
                "file": primary.file,
                "tags": primary.tags,
                "status": primary.status,
                "neighbors": {"depends_on": deps.iter().filter_map(|d| d.id.clone()).collect::<Vec<_>>(),
                               "dependents": dependents.iter().filter_map(|d| d.id.clone()).collect::<Vec<_>>()},
                "content": blocks,
            });
            print_json(&out)?;
        }
        OutputFormat::Plain => {
            println!("# {}: {}\n", id, primary.title);
            if !primary.depends_on.is_empty() {
                println!("## Depends On");
                for d in &deps {
                    println!("- {}: {}", d.id.clone().unwrap_or_default(), d.title);
                }
                println!();
            }
            if include_dependents && !dependents.is_empty() {
                println!("## Dependents ({})", dependents.len());
                for d in &dependents {
                    println!("- {}: {}", d.id.clone().unwrap_or_default(), d.title);
                }
                println!();
            }
            let content = fs::read_to_string(&primary.file).unwrap_or_default();
            println!("## Content\n\n{}", content);
        }
    }
    Ok(())
}

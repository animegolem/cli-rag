use anyhow::{Context, Result};
use std::fs;
use std::time::{Duration, SystemTime};

use crate::config::{Config, build_schema_sets};
use crate::model::AdrDoc;

pub fn write_indexes(cfg: &Config, docs: &Vec<AdrDoc>, _force: bool, _auto_write: bool) -> Result<()> {
    let schema_sets = build_schema_sets(cfg);
    for base in &cfg.bases {
        let mut list = Vec::new();
        for d in docs {
            if d.file.starts_with(base) {
                let mut note_type: Option<String> = None;
                let fname = d.file.file_name().and_then(|s| s.to_str()).unwrap_or("");
                for (sc, set) in &schema_sets { if set.is_match(fname) { note_type = Some(sc.name.clone()); break; } }
                list.push(serde_json::json!({
                    "file": d.file.strip_prefix(base).unwrap_or(&d.file).to_string_lossy(),
                    "id": d.id.clone().unwrap_or_default(),
                    "title": d.title,
                    "tags": d.tags,
                    "status": d.status.clone().unwrap_or_default(),
                    "depends_on": d.depends_on,
                    "supersedes": d.supersedes,
                    "superseded_by": d.superseded_by,
                    "groups": d.groups,
                    "type": note_type,
                    "mtime": d.mtime,
                    "size": d.size,
                }));
            }
        }
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::from_secs(0)).as_secs();
        let wrapper = serde_json::json!({
            "index_version": 1,
            "generated_at": now,
            "items": list,
        });
        let out_path = base.join(&cfg.index_relative);
        if let Some(parent) = out_path.parent() { fs::create_dir_all(parent).ok(); }
        fs::write(&out_path, serde_json::to_string_pretty(&wrapper)?).with_context(|| format!("writing index to {}", out_path.display()))?;
        eprintln!("Wrote index: {} ({} entries)", out_path.display(), wrapper.get("items").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0));
    }
    Ok(())
}

pub fn write_groups_config(cfg: &Config, docs: &Vec<AdrDoc>) -> Result<()> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut by_group: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for d in docs { if let Some(ref id) = d.id { for g in &d.groups { by_group.entry(g.clone()).or_default().insert(id.clone()); } } }
    let mut sections = Vec::new();
    for (title, ids) in by_group { sections.push(serde_json::json!({ "title": title, "selectors": [ { "anyIds": ids.into_iter().collect::<Vec<_>>() } ] })); }
    for base in &cfg.bases {
        let out_path = base.join(&cfg.groups_relative);
        if let Some(parent) = out_path.parent() { fs::create_dir_all(parent).ok(); }
        let body = serde_json::json!({ "sections": sections });
        fs::write(&out_path, serde_json::to_string_pretty(&body)?).with_context(|| format!("writing groups to {}", out_path.display()))?;
        eprintln!("Wrote groups: {}", out_path.display());
    }
    Ok(())
}

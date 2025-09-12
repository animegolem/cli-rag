use crate::cli::OutputFormat;
use crate::commands::output::{print_json, print_ndjson_iter};
use anyhow::Result;

use crate::config::{build_schema_sets, Config};
use crate::discovery::docs_with_source;

pub fn run(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    format: &OutputFormat,
    query: String,
) -> Result<()> {
    let q = query.to_lowercase();
    let (docs, used_unified) = docs_with_source(cfg, cfg_path)?;
    if !used_unified {
        eprintln!("Note: unified index not found; falling back to per-base/scan. Consider `cli-rag validate`.");
    }
    let mut hits: Vec<&crate::model::AdrDoc> = Vec::new();
    for d in &docs {
        let id = d.id.clone().unwrap_or_default();
        if id.to_lowercase().contains(&q) || d.title.to_lowercase().contains(&q) {
            hits.push(d);
        }
    }
    // Deterministic ordering: (score desc placeholder=0) -> lastModified desc -> id asc
    // Compute lastModified timestamps and schema names for emitters
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
    let mut enriched: Vec<serde_json::Value> = hits
        .iter()
        .filter_map(|d| d.id.as_ref().map(|id| (id.clone(), *d)))
        .map(|(id, d)| {
            let last_modified = std::fs::metadata(&d.file)
                .and_then(|md| md.modified())
                .ok()
                .map(|st| {
                    let dt: chrono::DateTime<chrono::Utc> = st.into();
                    dt.to_rfc3339()
                });
            serde_json::json!({
                "kind": "note",
                "id": id,
                "title": d.title,
                "schema": infer_schema(&d.file),
                "path": d.file.display().to_string(),
                "tags": d.tags,
                "status": d.status,
                "kanbanStatusLine": serde_json::Value::Null,
                "kanbanStatus": serde_json::Value::Null,
                "score": serde_json::Value::Null,
                "lastModified": last_modified,
                "lastAccessed": serde_json::Value::Null,
            })
        })
        .collect();
    // sort by lastModified desc then id asc
    enriched.sort_by(|a, b| {
        let lm_a = a.get("lastModified").and_then(|v| v.as_str());
        let lm_b = b.get("lastModified").and_then(|v| v.as_str());
        match (lm_a, lm_b) {
            (Some(aa), Some(bb)) => {
                let ord = bb.cmp(aa); // desc
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            (Some(_), None) => return std::cmp::Ordering::Less,
            (None, Some(_)) => return std::cmp::Ordering::Greater,
            (None, None) => {}
        }
        let ida = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let idb = b.get("id").and_then(|v| v.as_str()).unwrap_or("");
        ida.cmp(idb)
    });
    match format {
        OutputFormat::Json | OutputFormat::Ai => {
            let body = serde_json::json!({"results": enriched});
            print_json(&body)?;
        }
        OutputFormat::Ndjson => {
            // For NDJSON, emit each result as a JSON object consistent with envelope items
            print_ndjson_iter::<serde_json::Value, _>(enriched.into_iter())?;
        }
        OutputFormat::Plain => {
            for v in &enriched {
                println!(
                    "{}\t{}\t{}",
                    v["id"].as_str().unwrap_or(""),
                    v["title"].as_str().unwrap_or(""),
                    v["path"].as_str().unwrap_or("")
                );
            }
        }
    }
    Ok(())
}

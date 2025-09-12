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
    // Compute lastModified timestamps and schema names for emitters; also extract GTD items
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
    let mut enriched: Vec<serde_json::Value> = Vec::new();
    for d in hits {
        let Some(id) = d.id.as_ref() else { continue };
        let schema = infer_schema(&d.file);
        let path_str = d.file.display().to_string();
        let last_modified = std::fs::metadata(&d.file)
            .and_then(|md| md.modified())
            .ok()
            .map(|st| {
                let dt: chrono::DateTime<chrono::Utc> = st.into();
                dt.to_rfc3339()
            });
        // Note item
        let mut kanban_status_line: Option<String> = None;
        if let Some(v) = d.fm.get("kanban_statusline").and_then(|v| v.as_str()) {
            kanban_status_line = Some(v.to_string());
        }
        let mut kanban_status: Option<String> = None;
        if let Some(v) = d.fm.get("kanban_status").and_then(|v| v.as_str()) {
            kanban_status = Some(v.to_string());
        }
        enriched.push(serde_json::json!({
            "kind": "note",
            "id": id,
            "title": d.title,
            "schema": schema,
            "path": path_str,
            "tags": d.tags,
            "status": d.status,
            "kanbanStatusLine": kanban_status_line,
            "kanbanStatus": kanban_status,
            "score": serde_json::Value::Null,
            "lastModified": last_modified,
            "lastAccessed": serde_json::Value::Null,
        }));

        // Kanban item (optional) if frontmatter has kanban_status
        if let Some(status) = d.fm.get("kanban_status").and_then(|v| v.as_str()) {
            let due_date =
                d.fm.get("due_date")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            let ksl =
                d.fm.get("kanban_statusline")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            enriched.push(serde_json::json!({
                "kind": "kanban",
                "id": format!("{}-KANBAN", id),
                "noteId": id,
                "schema": schema,
                "path": path_str,
                "kanbanStatus": status,
                "kanbanStatusLine": ksl,
                "dueDate": due_date,
            }));
        }

        // TODO items in body: lines with Markdown checkboxes - [ ] or - [x]
        if let Ok(content) = std::fs::read_to_string(&d.file) {
            for (i, line) in content.lines().enumerate() {
                let trimmed = line.trim_start();
                let is_todo = trimmed.starts_with("- [ ] ")
                    || trimmed.starts_with("- [x] ")
                    || trimmed.starts_with("- [X] ");
                if !is_todo {
                    continue;
                }
                let text = trimmed.chars().skip(6).collect::<String>();
                enriched.push(serde_json::json!({
                    "kind": "todo",
                    "id": format!("{}#L{}", id, i+1),
                    "noteId": id,
                    "schema": schema,
                    "path": path_str,
                    "line": (i+1) as i64,
                    "priority": serde_json::Value::Null,
                    "priorityScore": serde_json::Value::Null,
                    "text": text,
                    "dueDate": serde_json::Value::Null,
                    "source": "body",
                    "span": serde_json::Value::Null,
                    "createdAt": serde_json::Value::Null,
                    "completedAt": serde_json::Value::Null,
                }));
            }
        }
    }
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

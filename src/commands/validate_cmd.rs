use anyhow::Result;

use crate::cli::OutputFormat;
use crate::commands::output::{print_json, print_ndjson_iter, print_ndjson_value};
use crate::config::Config;
use crate::discovery::incremental_collect_docs;
use crate::index::{write_groups_config, write_indexes};
use crate::protocol::{ToolCallLocation, ValidateHeader, ValidateIssue};
use crate::validate::validate_docs;

pub fn run(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    format: &OutputFormat,
    write_groups: bool,
    dry_run: bool,
    full_rescan: bool,
) -> Result<()> {
    let docs = incremental_collect_docs(cfg, full_rescan)?;
    let report = validate_docs(cfg, &docs);
    // Helper to derive location from message when possible
    fn derive_location(message: &str) -> Option<(String, ToolCallLocation)> {
        // Expected leading pattern: "/path/to/file.md: ..."
        let mut parts = message.splitn(2, ':');
        let file = parts.next()?.to_string();
        if file.is_empty() {
            return None;
        }
        // Try to infer a needle to search for a line number
        let needle = if let Some(rest) = parts.next() {
            // depends_on 'ID' not found
            if let Some(idx) = rest.find("'") {
                let rest2 = &rest[idx + 1..];
                rest2.find("'").map(|end| rest2[..end].to_string())
            } else {
                None
            }
        } else {
            None
        };
        let path = std::path::PathBuf::from(&file);
        let mut line: Option<u32> = None;
        if let Some(needle) = needle {
            if let Ok(content) = std::fs::read_to_string(&path) {
                for (i, l) in content.lines().enumerate() {
                    if l.contains(&needle) {
                        line = Some((i + 1) as u32);
                        break;
                    }
                }
            }
        }
        Some((file, ToolCallLocation { path, line }))
    }
    fn classify_code(message: &str, kind: &str) -> Option<String> {
        let m = message;
        // config loader codes may appear as E100/E110/E120 already
        if m.contains("multiple schema matches") {
            return Some("E200".into());
        }
        if m.contains("missing required") || m.contains("required '") {
            return Some("E220".into());
        }
        if m.contains("invalid status") {
            return Some("E212".into());
        }
        if m.contains("duplicate id") {
            return Some("E213".into());
        }
        if m.contains("conflict for id") {
            return Some("E214".into());
        }
        if m.contains("depends_on '") || m.contains("supersedes '") || m.contains("superseded_by '")
        {
            return Some("E230".into());
        }
        if m.contains("unknown keys") {
            return Some(if kind == "warning" { "W221" } else { "E221" }.into());
        }
        if m.contains("should be array")
            || m.contains("not a valid date")
            || m.contains("value '")
            || m.contains("contains disallowed")
            || m.contains("must have at least")
            || m.contains("does not match regex")
        {
            return Some("E225".into());
        }
        if m.contains("references") && m.contains("not in") {
            return Some("E231".into());
        }
        if m.contains("cycle detected") {
            return Some(if kind == "warning" { "W240" } else { "E240" }.into());
        }
        if m.contains("has no graph connections") {
            return Some("W250".into());
        }
        None
    }
    match format {
        OutputFormat::Json | OutputFormat::Ai => {
            // Build diagnostics array per contracts/cli/validate_result.schema.json
            let mut diagnostics: Vec<serde_json::Value> = Vec::new();
            for m in &report.errors {
                let (file, _loc) = derive_location(m).unwrap_or((
                    String::new(),
                    ToolCallLocation {
                        path: std::path::PathBuf::new(),
                        line: None,
                    },
                ));
                let code = classify_code(m, "error").unwrap_or_else(|| "E000".to_string());
                diagnostics.push(serde_json::json!({
                    "severity": "error",
                    "code": code,
                    "msg": m,
                    "path": if file.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(file) }
                }));
            }
            for m in &report.warnings {
                let (file, _loc) = derive_location(m).unwrap_or((
                    String::new(),
                    ToolCallLocation {
                        path: std::path::PathBuf::new(),
                        line: None,
                    },
                ));
                let code = classify_code(m, "warning").unwrap_or_else(|| "W000".to_string());
                diagnostics.push(serde_json::json!({
                    "severity": "warning",
                    "code": code,
                    "msg": m,
                    "path": if file.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(file) }
                }));
            }
            let obj = serde_json::json!({
                "ok": report.ok,
                "docCount": docs.len(),
                "diagnostics": diagnostics,
            });
            print_json(&obj)?;
        }
        OutputFormat::Ndjson => {
            // Emit a header then each error and warning as individual typed records
            let header = ValidateHeader {
                ok: report.ok,
                doc_count: docs.len(),
            };
            print_ndjson_value(&serde_json::to_value(&header)?)?;
            let errs = report.errors.iter().map(|m| {
                let (file, location) = derive_location(m).unwrap_or((
                    String::new(),
                    ToolCallLocation {
                        path: std::path::PathBuf::new(),
                        line: None,
                    },
                ));
                let code = classify_code(m, "error");
                ValidateIssue {
                    kind: "error".into(),
                    file: if file.is_empty() { None } else { Some(file) },
                    message: m.clone(),
                    code,
                    location: if location.path.as_os_str().is_empty() {
                        None
                    } else {
                        Some(location)
                    },
                }
            });
            let warns = report.warnings.iter().map(|m| {
                let (file, location) = derive_location(m).unwrap_or((
                    String::new(),
                    ToolCallLocation {
                        path: std::path::PathBuf::new(),
                        line: None,
                    },
                ));
                let code = classify_code(m, "warning");
                ValidateIssue {
                    kind: "warning".into(),
                    file: if file.is_empty() { None } else { Some(file) },
                    message: m.clone(),
                    code,
                    location: if location.path.as_os_str().is_empty() {
                        None
                    } else {
                        Some(location)
                    },
                }
            });
            print_ndjson_iter(errs.chain(warns))?;
        }
        OutputFormat::Plain => {
            if report.ok {
                println!("Validation OK ({} docs)", docs.len());
            } else {
                eprintln!("Validation failed:");
                for e in &report.errors {
                    eprintln!(" - {}", e);
                }
            }
            if !report.warnings.is_empty() {
                eprintln!("Warnings:");
                for w in &report.warnings {
                    eprintln!(" - {}", w);
                }
            }
        }
    }
    if report.ok && !dry_run {
        let cfg_dir = cfg_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p as &std::path::Path);
        write_indexes(cfg, &docs, true, true, cfg_dir)?;
        // Write resolved config snapshot (camelCase) aligned to contracts/v1/resolved_config.json
        if let Some(root) = cfg_dir {
            let project_root = root.to_path_buf();
            let resolved_path = project_root.join(".cli-rag/resolved.json");
            if let Some(parent) = resolved_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let scan = serde_json::json!({
                "filepaths": cfg.bases.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                "indexPath": project_root.join(&cfg.index_relative).display().to_string(),
                "hashMode": "mtime",
                "indexStrategy": "content",
                "ignoreGlobs": cfg.ignore_globs,
                "ignoreSymlinks": true
            });
            let authoring = serde_json::json!({
                "editor": "nvim",
                "backgroundWatch": true
            });
            let graph = serde_json::json!({
                "depth": cfg.defaults.depth as i64,
                "includeBidirectional": cfg.defaults.include_bidirectional,
                "ai": {"depth": 1, "defaultFanout": 5, "includeBidirectional": true, "neighborStyle": "metadata", "outlineLines": 2}
            });
            let templates = serde_json::json!({
                "import": cfg.import
            });
            let schemas: Vec<serde_json::Value> = cfg
                .schema
                .iter()
                .map(|s| serde_json::json!({"name": s.name, "filePatterns": s.file_patterns}))
                .collect();
            let resolved = serde_json::json!({
                "protocolVersion": crate::protocol::PROTOCOL_VERSION,
                "configVersion": "0.1",
                "luaApiVersion": 1,
                "projectRoot": project_root.display().to_string(),
                "scan": scan,
                "authoring": authoring,
                "graph": graph,
                "templates": templates,
                "schemas": schemas
            });
            let _ = std::fs::write(
                &resolved_path,
                serde_json::to_string_pretty(&resolved).unwrap_or_else(|_| "{}".into()),
            );
        }
    }
    if write_groups && !dry_run {
        write_groups_config(cfg, &docs)?;
    }
    if !report.ok {
        std::process::exit(1);
    }
    Ok(())
}

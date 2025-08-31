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
            let errors: Vec<ValidateIssue> = report
                .errors
                .iter()
                .map(|m| {
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
                })
                .collect();
            let warnings: Vec<ValidateIssue> = report
                .warnings
                .iter()
                .map(|m| {
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
                })
                .collect();
            let obj = serde_json::json!({
                "ok": report.ok,
                "doc_count": docs.len(),
                "errors": errors,
                "warnings": warnings,
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
    }
    if write_groups && !dry_run {
        write_groups_config(cfg, &docs)?;
    }
    if !report.ok {
        std::process::exit(1);
    }
    Ok(())
}

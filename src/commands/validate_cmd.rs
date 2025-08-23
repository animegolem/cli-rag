use anyhow::Result;

use crate::config::Config;
use crate::discovery::incremental_collect_docs;
use crate::index::{write_indexes, write_groups_config};
use crate::commands::output::{print_json, print_ndjson_value};
use crate::validate::validate_docs;

pub fn run(cfg: &Config, format: &str, write_groups: bool, dry_run: bool, full_rescan: bool) -> Result<()> {
    let docs = incremental_collect_docs(cfg, full_rescan)?;
    let report = validate_docs(cfg, &docs);
    if format == "json" {
        let obj = serde_json::json!({ "ok": report.ok, "errors": report.errors, "warnings": report.warnings });
        print_json(&obj)?;
    } else if format == "ndjson" {
        // Emit a header then each error and warning as individual records
        let header = serde_json::json!({ "ok": report.ok, "doc_count": docs.len() });
        print_ndjson_value(&header)?;
        for e in &report.errors { print_ndjson_value(&serde_json::json!({"type":"error","message": e}))?; }
        for w in &report.warnings { print_ndjson_value(&serde_json::json!({"type":"warning","message": w}))?; }
    } else {
        if report.ok { println!("Validation OK ({} docs)", docs.len()); } else {
            eprintln!("Validation failed:");
            for e in &report.errors { eprintln!(" - {}", e); }
        }
        if !report.warnings.is_empty() {
            eprintln!("Warnings:");
            for w in &report.warnings { eprintln!(" - {}", w); }
        }
    }
    if report.ok && !dry_run { write_indexes(cfg, &docs, true, true)?; }
    if write_groups && !dry_run { write_groups_config(cfg, &docs)?; }
    if !report.ok { std::process::exit(1); }
    Ok(())
}

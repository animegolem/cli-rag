use anyhow::Result;

use crate::cli::OutputFormat;
use crate::commands::output::{print_json, print_ndjson_iter, print_ndjson_value};
use crate::config::Config;
use crate::discovery::incremental_collect_docs;
use crate::index::{write_groups_config, write_indexes};
use crate::protocol::{ValidateHeader, ValidateIssue};
use crate::validate::validate_docs;

pub fn run(
    cfg: &Config,
    format: &OutputFormat,
    write_groups: bool,
    dry_run: bool,
    full_rescan: bool,
) -> Result<()> {
    let docs = incremental_collect_docs(cfg, full_rescan)?;
    let report = validate_docs(cfg, &docs);
    match format {
        OutputFormat::Json => {
            let errors: Vec<ValidateIssue> = report
                .errors
                .iter()
                .map(|m| ValidateIssue {
                    kind: "error".into(),
                    file: None,
                    message: m.clone(),
                    code: None,
                })
                .collect();
            let warnings: Vec<ValidateIssue> = report
                .warnings
                .iter()
                .map(|m| ValidateIssue {
                    kind: "warning".into(),
                    file: None,
                    message: m.clone(),
                    code: None,
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
            let errs = report.errors.iter().map(|m| ValidateIssue {
                kind: "error".into(),
                file: None,
                message: m.clone(),
                code: None,
            });
            let warns = report.warnings.iter().map(|m| ValidateIssue {
                kind: "warning".into(),
                file: None,
                message: m.clone(),
                code: None,
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
        write_indexes(cfg, &docs, true, true)?;
    }
    if write_groups && !dry_run {
        write_groups_config(cfg, &docs)?;
    }
    if !report.ok {
        std::process::exit(1);
    }
    Ok(())
}

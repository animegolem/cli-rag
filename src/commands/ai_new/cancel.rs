use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::commands::output::print_json;
use crate::config::Config;

use super::store::{CancelResponse, DraftRecord, SubmitDiagnostic, SubmitFailure};
use super::utils::resolve_project_root;
use crate::cli::OutputFormat;

pub fn cancel(
    _cfg: &Config,
    cfg_path: &Option<PathBuf>,
    draft_id: Option<String>,
    format: &OutputFormat,
) -> Result<()> {
    super::utils::ensure_json_output(format);
    let project_root = resolve_project_root(cfg_path)?;
    let drafts_dir = project_root.join(".cli-rag/drafts");
    let mut available: Vec<DraftRecord> = Vec::new();
    if drafts_dir.exists() {
        for entry in fs::read_dir(&drafts_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_file() {
                continue;
            }
            if !entry.file_name().to_string_lossy().ends_with(".json") {
                continue;
            }
            let data = fs::read_to_string(entry.path())?;
            if let Ok(record) = serde_json::from_str::<DraftRecord>(&data) {
                available.push(record);
            }
        }
    }
    available.sort_by(|a, b| a.draft_id.cmp(&b.draft_id));

    let target_id = match draft_id {
        Some(id) => id,
        None => {
            if available.is_empty() {
                emit_failure(
                    "".to_string(),
                    "NO_ACTIVE_DRAFTS",
                    "No active drafts to cancel".into(),
                )
            } else if available.len() == 1 {
                available[0].draft_id.clone()
            } else {
                let options = available
                    .iter()
                    .map(|d| format!("{} ({})", d.draft_id, d.title))
                    .collect::<Vec<_>>()
                    .join(", ");
                emit_failure(
                    "".to_string(),
                    "MULTIPLE_DRAFTS",
                    format!(
                        "Multiple drafts active; cancel one with --draft <ID>. Available: {}",
                        options
                    ),
                )
            }
        }
    };

    let draft_path = drafts_dir.join(format!("{}.json", target_id));
    if !draft_path.exists() {
        emit_failure(target_id, "DRAFT_NOT_FOUND", "Draft not found".into());
    }
    fs::remove_file(&draft_path)?;
    let response = CancelResponse {
        ok: true,
        draft_id: target_id,
    };
    print_json(&response)?;
    Ok(())
}

#[cold]
fn emit_failure(draft_id: String, code: &str, message: String) -> ! {
    let failure = SubmitFailure {
        ok: false,
        draft_id,
        diagnostics: vec![SubmitDiagnostic {
            severity: "error".into(),
            code: code.into(),
            message,
            heading: None,
            max: None,
            actual: None,
        }],
    };
    let _ = print_json(&failure);
    std::process::exit(3);
}

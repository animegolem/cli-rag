use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::commands::output::print_json;
use crate::config::Config;

use super::store::{CancelResponse, SubmitDiagnostic, SubmitFailure};
use super::utils::resolve_project_root;
use crate::cli::OutputFormat;

pub fn cancel(
    _cfg: &Config,
    cfg_path: &Option<PathBuf>,
    draft_id: String,
    format: &OutputFormat,
) -> Result<()> {
    super::utils::ensure_json_output(format);
    let project_root = resolve_project_root(cfg_path)?;
    let drafts_dir = project_root.join(".cli-rag/drafts");
    let draft_path = drafts_dir.join(format!("{}.json", draft_id));
    if !draft_path.exists() {
        let failure = SubmitFailure {
            ok: false,
            draft_id,
            diagnostics: vec![SubmitDiagnostic {
                severity: "error".into(),
                code: "DRAFT_NOT_FOUND".into(),
                message: "Draft not found".into(),
                heading: None,
                max: None,
                actual: None,
            }],
        };
        print_json(&failure)?;
        std::process::exit(3);
    }
    fs::remove_file(&draft_path)?;
    let response = CancelResponse { ok: true, draft_id };
    print_json(&response)?;
    Ok(())
}

use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::commands::output::print_json;
use crate::config::Config;
use crate::discovery::docs_with_source;
use crate::model::parse_front_matter_and_title;
use crate::validate::validate_docs;

use super::payload::{
    assemble_note, check_readonly, load_submit_payload, validate_sections, SubmitRequest,
};
use super::store::{DraftRecord, SubmitDiagnostic, SubmitFailure, SubmitSuccess};
use super::utils::{path_relative_to, resolve_project_root};
use crate::cli::OutputFormat;

pub fn submit(
    cfg: &Config,
    cfg_path: &Option<PathBuf>,
    request: SubmitRequest,
    format: &OutputFormat,
) -> Result<()> {
    super::utils::ensure_json_output(format);
    let project_root = resolve_project_root(cfg_path)?;
    let drafts_dir = project_root.join(".cli-rag/drafts");
    let SubmitRequest {
        draft_id,
        input,
        allow_oversize,
    } = request;
    let draft_path = drafts_dir.join(format!("{}.json", draft_id));
    if !draft_path.exists() {
        let failure = SubmitFailure {
            ok: false,
            draft_id,
            diagnostics: vec![super::store::SubmitDiagnostic {
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
    let data = fs::read_to_string(&draft_path)?;
    let record: DraftRecord = serde_json::from_str(&data)?;
    if record.is_expired(chrono::Utc::now().timestamp()) {
        fs::remove_file(&draft_path).ok();
        let failure = SubmitFailure {
            ok: false,
            draft_id: record.draft_id.clone(),
            diagnostics: vec![super::store::SubmitDiagnostic {
                severity: "error".into(),
                code: "DRAFT_EXPIRED".into(),
                message: "Draft expired".into(),
                heading: None,
                max: None,
                actual: None,
            }],
        };
        print_json(&failure)?;
        std::process::exit(3);
    }

    let payload = load_submit_payload(&input)?;
    let readonly_diagnostics = check_readonly(&record, &payload);
    if !readonly_diagnostics.is_empty() {
        let failure = SubmitFailure {
            ok: false,
            draft_id: record.draft_id.clone(),
            diagnostics: readonly_diagnostics,
        };
        print_json(&failure)?;
        std::process::exit(2);
    }
    if !allow_oversize {
        let diagnostics = validate_sections(&record, &payload);
        if !diagnostics.is_empty() {
            let failure = SubmitFailure {
                ok: false,
                draft_id: record.draft_id.clone(),
                diagnostics,
            };
            print_json(&failure)?;
            std::process::exit(2);
        }
    }
    let final_note = assemble_note(&record, payload)?;
    let target_path = record.target_path(&project_root);
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).ok();
    }

    let (mut docs, _used_unified) = docs_with_source(cfg, cfg_path)?;
    let temp_doc = parse_front_matter_and_title(&final_note, &target_path);
    docs.push(temp_doc);
    let report = validate_docs(cfg, cfg_path, &docs);
    if !report.ok {
        let diagnostics = report
            .errors
            .into_iter()
            .map(|msg| SubmitDiagnostic {
                severity: "error".into(),
                code: "VALIDATION".into(),
                message: msg,
                heading: None,
                max: None,
                actual: None,
            })
            .chain(report.warnings.into_iter().map(|msg| SubmitDiagnostic {
                severity: "warning".into(),
                code: "VALIDATION".into(),
                message: msg,
                heading: None,
                max: None,
                actual: None,
            }))
            .collect();
        let failure = SubmitFailure {
            ok: false,
            draft_id: record.draft_id.clone(),
            diagnostics,
        };
        print_json(&failure)?;
        std::process::exit(2);
    }

    fs::write(&target_path, final_note)?;
    fs::remove_file(&draft_path).ok();

    let path_display = path_relative_to(&target_path, &project_root);
    let success = SubmitSuccess {
        ok: true,
        path: path_display,
        id: record.id,
        schema: record.schema,
    };
    print_json(&success)?;
    Ok(())
}

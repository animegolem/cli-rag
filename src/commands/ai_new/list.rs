use anyhow::Result;
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

use crate::commands::output::print_json;
use crate::config::Config;

use super::store::{DraftRecord, ListDraft, ListResponse};
use super::utils::resolve_project_root;
use crate::cli::OutputFormat;

pub fn list(
    _cfg: &Config,
    cfg_path: &Option<PathBuf>,
    stale_days: Option<u64>,
    format: &OutputFormat,
) -> Result<()> {
    super::utils::ensure_json_output(format);
    let project_root = resolve_project_root(cfg_path)?;
    let drafts_dir = project_root.join(".cli-rag/drafts");
    let mut drafts: Vec<ListDraft> = Vec::new();
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
                let created = chrono::DateTime::<Utc>::from_timestamp(record.created_at, 0)
                    .unwrap_or_else(Utc::now);
                let expires = created + chrono::Duration::seconds(record.ttl_seconds as i64);
                let age_days = (Utc::now() - created).num_days() as u64;
                if let Some(limit) = stale_days {
                    if age_days < limit {
                        continue;
                    }
                }
                drafts.push(ListDraft {
                    draft_id: record.draft_id,
                    schema: record.schema,
                    id: record.id,
                    title: record.title,
                    filename: record.filename,
                    created_at: created.to_rfc3339(),
                    expires_at: expires.to_rfc3339(),
                    ttl_seconds: record.ttl_seconds,
                });
            }
        }
    }
    drafts.sort_by(|a, b| a.draft_id.cmp(&b.draft_id));
    let response = ListResponse { drafts };
    print_json(&response)?;
    Ok(())
}

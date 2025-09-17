use crate::cli::OutputFormat;
use crate::config::Config;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use super::store::DraftRecord;

pub fn ensure_json_output(format: &OutputFormat) {
    if !matches!(format, OutputFormat::Json | OutputFormat::Ai) {
        eprintln!("info: ai new outputs JSON; ignoring --format override");
    }
}

pub fn resolve_project_root(cfg_path: &Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = cfg_path {
        if let Some(dir) = path.parent() {
            return Ok(dir.to_path_buf());
        }
    }
    Ok(std::env::current_dir()?)
}

pub fn generate_draft_id() -> String {
    format!("dft_{}", uuid::Uuid::new_v4().simple())
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    let hex = out.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    format!("sha256:{}", hex)
}

pub fn path_to_string(p: &Path) -> String {
    p.to_string_lossy().to_string()
}

pub fn path_relative_to(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string_lossy().to_string())
}

pub fn determine_filename(cfg: &Config, schema: &str, id: &str, title: &str) -> String {
    let schema_tpl_from_new: Option<String> = cfg
        .schema
        .iter()
        .find(|s| s.name == schema)
        .and_then(|s| s.new.as_ref())
        .and_then(|n| n.filename_template.clone());
    let schema_tpl_legacy: Option<String> = cfg
        .schema
        .iter()
        .find(|s| s.name == schema)
        .and_then(|s| s.filename_template.clone());
    if let Some(tpl) = schema_tpl_from_new.or(schema_tpl_legacy) {
        crate::commands::new_helpers::render_filename_template(&tpl, id, title, schema)
    } else {
        format!("{}.md", id)
    }
}

pub fn extract_primary_heading(record: &DraftRecord) -> String {
    if !record.primary_heading.is_empty() {
        let mut heading = record.primary_heading.clone();
        if !heading.ends_with('\n') {
            heading.push('\n');
        }
        heading.push('\n');
        heading
    } else {
        format!("# {}\n\n", record.title)
    }
}

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::path::{Path, PathBuf};

pub const DEFAULT_TTL_SECONDS: u64 = 86_400;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeadingConstraint {
    pub name: String,
    pub max_lines: u64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FrontmatterConstraint {
    pub allowed: Vec<String>,
    pub readonly: Vec<String>,
    pub enums: JsonMap<String, JsonValue>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DraftConstraints {
    pub headings: Vec<HeadingConstraint>,
    pub heading_strictness: String,
    pub frontmatter: FrontmatterConstraint,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DraftRecord {
    pub draft_id: String,
    pub schema: String,
    pub id: String,
    pub title: String,
    pub filename: String,
    pub base: String,
    pub created_at: i64,
    pub ttl_seconds: u64,
    pub note_template: String,
    pub seed_frontmatter: JsonValue,
    pub constraints: DraftConstraints,
    pub instructions: String,
    pub content_hash: String,
    pub primary_heading: String,
}

impl DraftRecord {
    pub fn draft_path(&self, drafts_dir: &Path) -> PathBuf {
        drafts_dir.join(format!("{}.json", self.draft_id))
    }

    pub fn is_expired(&self, now_ts: i64) -> bool {
        now_ts.saturating_sub(self.created_at) > self.ttl_seconds as i64
    }

    pub fn target_path(&self, project_root: &Path) -> PathBuf {
        let base_path = PathBuf::from(&self.base);
        let base_abs = if base_path.is_absolute() {
            base_path
        } else {
            project_root.join(base_path)
        };
        base_abs.join(&self.filename)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartResponse {
    pub draft_id: String,
    pub schema: String,
    pub id: String,
    pub title: String,
    pub filename: String,
    pub note_template: String,
    pub seed_frontmatter: JsonValue,
    pub constraints: DraftConstraints,
    pub instructions: String,
    pub ttl_seconds: u64,
    pub content_hash: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelResponse {
    pub ok: bool,
    pub draft_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitSuccess {
    pub ok: bool,
    pub path: String,
    pub id: String,
    pub schema: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitDiagnostic {
    pub severity: String,
    pub code: String,
    pub message: String,
    pub heading: Option<String>,
    pub max: Option<u64>,
    pub actual: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitFailure {
    pub ok: bool,
    pub draft_id: String,
    pub diagnostics: Vec<SubmitDiagnostic>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDraft {
    pub draft_id: String,
    pub schema: String,
    pub id: String,
    pub title: String,
    pub filename: String,
    pub created_at: String,
    pub expires_at: String,
    pub ttl_seconds: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse {
    pub drafts: Vec<ListDraft>,
}

pub fn extract_frontmatter_json(note: &str) -> Result<JsonValue> {
    if note.starts_with("---\n") {
        if let Some(end) = note.find("\n---\n") {
            let fm_content = &note[4..end];
            let yaml_val: serde_yaml::Value = serde_yaml::from_str(fm_content)?;
            let json_val = serde_json::to_value(yaml_val)?;
            return Ok(json_val);
        }
    }
    Ok(JsonValue::Object(JsonMap::new()))
}

pub fn build_constraints(template_raw: &str, seed_frontmatter: &JsonValue) -> DraftConstraints {
    let mut headings: Vec<HeadingConstraint> = Vec::new();
    let mut current: Option<HeadingConstraint> = None;
    for line in template_raw.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(h) = current.take() {
                headings.push(h);
            }
            current = Some(HeadingConstraint {
                name: rest.trim().to_string(),
                max_lines: 0,
            });
        } else if let Some(ref mut h) = current {
            if let Some(loc) = extract_loc(line) {
                h.max_lines = loc;
            }
        }
    }
    if let Some(h) = current.take() {
        headings.push(h);
    }
    for h in headings.iter_mut() {
        if h.max_lines == 0 {
            h.max_lines = 200;
        }
    }

    let mut allowed = Vec::new();
    if let Some(obj) = seed_frontmatter.as_object() {
        for key in obj.keys() {
            allowed.push(key.clone());
        }
    }
    if !allowed.contains(&"id".to_string()) {
        allowed.push("id".to_string());
    }
    let frontmatter = FrontmatterConstraint {
        allowed,
        readonly: vec!["id".to_string()],
        enums: JsonMap::new(),
    };
    DraftConstraints {
        headings,
        heading_strictness: "missing_only".into(),
        frontmatter,
    }
}

fn extract_loc(line: &str) -> Option<u64> {
    let trimmed = line.trim();
    if let Some(stripped) = trimmed.strip_prefix("{{LOC|") {
        if let Some(end) = stripped.find("}}") {
            return stripped[..end].parse().ok();
        }
    }
    None
}

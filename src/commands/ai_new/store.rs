use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::commands::ai_new::template_utils::extract_heading_constraints;
use crate::config::SchemaCfg;

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
    #[serde(default)]
    pub allowed: Vec<String>,
    #[serde(default)]
    pub readonly: Vec<String>,
    #[serde(default)]
    pub enums: JsonMap<String, JsonValue>,
    #[serde(default)]
    pub globs: JsonMap<String, JsonValue>,
    #[serde(default)]
    pub integers: JsonMap<String, JsonValue>,
    #[serde(default)]
    pub floats: JsonMap<String, JsonValue>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DraftConstraints {
    pub headings: Vec<HeadingConstraint>,
    pub heading_strictness: String,
    #[serde(default)]
    pub heading_severity: String,
    #[serde(default)]
    pub max_heading_count: Option<u64>,
    #[serde(default)]
    pub line_count_scan_policy: String,
    #[serde(default)]
    pub line_count_severity: String,
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

pub fn build_constraints(
    template_raw: &str,
    seed_frontmatter: &JsonValue,
    schema_cfg: Option<&SchemaCfg>,
) -> DraftConstraints {
    let headings = extract_heading_constraints(template_raw);

    let mut allowed: BTreeSet<String> = BTreeSet::new();
    if let Some(obj) = seed_frontmatter.as_object() {
        allowed.extend(obj.keys().cloned());
    }
    let mut heading_strictness = "missing_only".to_string();
    let mut heading_severity = "error".to_string();
    let mut max_heading_count: Option<u64> = None;
    let mut line_count_scan_policy = "on_creation".to_string();
    let mut line_count_severity = "error".to_string();

    if let Some(schema) = schema_cfg {
        allowed.extend(schema.required.iter().cloned());
        allowed.extend(schema.allowed_keys.iter().cloned());
        allowed.extend(schema.rules.keys().cloned());
        if let Some(validate) = &schema.validate {
            if let Some(body) = &validate.body {
                if let Some(headings_cfg) = &body.headings {
                    if let Some(check) = &headings_cfg.heading_check {
                        heading_strictness = check.clone();
                    }
                    if let Some(max) = headings_cfg.max_count {
                        max_heading_count = Some(max as u64);
                    }
                    if let Some(sev) = &headings_cfg.severity {
                        heading_severity = sev.clone();
                    } else if let Some(sev) = &validate.severity {
                        heading_severity = sev.clone();
                    }
                }
                if let Some(line_cfg) = &body.line_count {
                    if let Some(policy) = &line_cfg.scan_policy {
                        line_count_scan_policy = policy.clone();
                    }
                    if let Some(sev) = &line_cfg.severity {
                        line_count_severity = sev.clone();
                    } else if let Some(sev) = &validate.severity {
                        line_count_severity = sev.clone();
                    }
                }
            }
        }
    }
    allowed.insert("id".to_string());

    let mut readonly: BTreeSet<String> = ["id", "created_date", "last_modified"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    if let Some(obj) = seed_frontmatter.as_object() {
        for key in obj.keys() {
            if key == "created_date" || key == "last_modified" {
                readonly.insert(key.clone());
            }
        }
    }

    let mut enums = JsonMap::new();
    let mut globs = JsonMap::new();
    let mut integers = JsonMap::new();
    let mut floats = JsonMap::new();
    if let Some(schema) = schema_cfg {
        for (field, rule) in &schema.rules {
            if let Some(values) = rule.enum_values.as_ref().filter(|v| !v.is_empty()) {
                let items = values
                    .iter()
                    .cloned()
                    .map(JsonValue::String)
                    .collect::<Vec<JsonValue>>();
                enums.insert(field.clone(), JsonValue::Array(items));
            } else if !rule.allowed.is_empty() {
                let items = rule
                    .allowed
                    .iter()
                    .cloned()
                    .map(JsonValue::String)
                    .collect::<Vec<JsonValue>>();
                enums.insert(field.clone(), JsonValue::Array(items));
            }
            if let Some(patterns) = &rule.globs {
                if !patterns.is_empty() {
                    let items = patterns
                        .iter()
                        .cloned()
                        .map(JsonValue::String)
                        .collect::<Vec<JsonValue>>();
                    globs.insert(field.clone(), JsonValue::Array(items));
                }
            }
            if let Some(int_rule) = &rule.integer {
                let mut obj = JsonMap::new();
                if let Some(min) = int_rule.min {
                    obj.insert("min".into(), JsonValue::Number(JsonNumber::from(min)));
                }
                if let Some(max) = int_rule.max {
                    obj.insert("max".into(), JsonValue::Number(JsonNumber::from(max)));
                }
                if !obj.is_empty() {
                    integers.insert(field.clone(), JsonValue::Object(obj));
                }
            }
            if let Some(float_rule) = &rule.float {
                let mut obj = JsonMap::new();
                if let Some(min) = float_rule.min {
                    if let Some(num) = JsonNumber::from_f64(min) {
                        obj.insert("min".into(), JsonValue::Number(num));
                    }
                }
                if let Some(max) = float_rule.max {
                    if let Some(num) = JsonNumber::from_f64(max) {
                        obj.insert("max".into(), JsonValue::Number(num));
                    }
                }
                if !obj.is_empty() {
                    floats.insert(field.clone(), JsonValue::Object(obj));
                }
            }
        }
    }

    let frontmatter = FrontmatterConstraint {
        allowed: allowed.into_iter().collect(),
        readonly: readonly.into_iter().collect(),
        enums,
        globs,
        integers,
        floats,
    };
    DraftConstraints {
        headings,
        heading_strictness,
        heading_severity,
        max_heading_count,
        line_count_scan_policy,
        line_count_severity,
        frontmatter,
    }
}

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use super::store::{DraftRecord, SubmitDiagnostic};
use super::utils::extract_primary_heading;

#[derive(Deserialize)]
struct SubmitJsonPayload {
    #[serde(default)]
    frontmatter: JsonMap<String, JsonValue>,
    #[serde(default)]
    sections: JsonMap<String, JsonValue>,
}

#[derive(Clone)]
pub enum SubmitInput {
    Stdin,
    Sections(PathBuf),
    Markdown(PathBuf),
}

pub struct SubmitRequest {
    pub draft_id: String,
    pub input: SubmitInput,
    pub allow_oversize: bool,
}

#[derive(Clone)]
pub struct SubmitPayload {
    pub frontmatter: JsonMap<String, JsonValue>,
    pub sections: HashMap<String, String>,
}

pub fn load_submit_payload(input: &SubmitInput) -> Result<SubmitPayload> {
    match input {
        SubmitInput::Stdin => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            parse_json_payload(&buf)
        }
        SubmitInput::Sections(path) => {
            let data = fs::read_to_string(path)
                .with_context(|| format!("reading sections payload {}", path.display()))?;
            parse_json_payload(&data)
        }
        SubmitInput::Markdown(path) => {
            let data = fs::read_to_string(path)
                .with_context(|| format!("reading markdown payload {}", path.display()))?;
            parse_markdown_payload(&data)
        }
    }
}

pub fn validate_sections(record: &DraftRecord, payload: &SubmitPayload) -> Vec<SubmitDiagnostic> {
    let mut diagnostics = Vec::new();
    for heading in &record.constraints.headings {
        let content = payload
            .sections
            .get(&heading.name)
            .cloned()
            .unwrap_or_default();
        let line_count = content.lines().count() as u64;
        if heading.max_lines > 0 && line_count > heading.max_lines {
            diagnostics.push(SubmitDiagnostic {
                severity: "error".into(),
                code: "LOC_LIMIT".into(),
                message: format!(
                    "Section '{}' exceeds max lines ({} > {})",
                    heading.name, line_count, heading.max_lines
                ),
                heading: Some(heading.name.clone()),
                max: Some(heading.max_lines),
                actual: Some(line_count),
            });
        }
    }
    diagnostics
}

pub fn assemble_note(record: &DraftRecord, payload: SubmitPayload) -> Result<String> {
    let mut fm_map = match record.seed_frontmatter.clone() {
        JsonValue::Object(map) => map,
        _ => JsonMap::new(),
    };
    for (k, v) in payload.frontmatter {
        fm_map.insert(k, v);
    }
    fm_map.insert("id".into(), JsonValue::String(record.id.clone()));
    let fm_yaml = serde_yaml::to_string(&fm_map)?;

    let mut sections_out = String::new();
    for heading in &record.constraints.headings {
        let content = payload
            .sections
            .get(&heading.name)
            .cloned()
            .unwrap_or_default();
        sections_out.push_str(&format!("## {}\n{}\n\n", heading.name, content.trim_end()));
    }

    let mut note = String::new();
    note.push_str("---\n");
    note.push_str(&fm_yaml);
    if !note.ends_with('\n') {
        note.push('\n');
    }
    note.push_str("---\n\n");
    note.push_str(&extract_primary_heading(record));
    note.push_str(&sections_out);
    Ok(note)
}

fn parse_json_payload(data: &str) -> Result<SubmitPayload> {
    let parsed: SubmitJsonPayload = serde_json::from_str(data)?;
    let mut sections = HashMap::new();
    for (k, v) in parsed.sections {
        let text = match v {
            JsonValue::String(s) => s,
            JsonValue::Array(arr) => arr
                .into_iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect::<Vec<String>>()
                .join("\n"),
            _ => v.to_string(),
        };
        sections.insert(k, text);
    }
    Ok(SubmitPayload {
        frontmatter: parsed.frontmatter,
        sections,
    })
}

fn parse_markdown_payload(data: &str) -> Result<SubmitPayload> {
    let fm_json = super::store::extract_frontmatter_json(data)?;
    let mut sections = HashMap::new();
    let mut body = data;
    if data.starts_with("---\n") {
        if let Some(end) = data.find("\n---\n") {
            body = &data[end + 5..];
        }
    }
    let mut current_heading: Option<String> = None;
    let mut current_body = String::new();
    for line in body.lines() {
        if line.starts_with("## ") {
            if let Some(h) = current_heading.take() {
                sections.insert(h, current_body.trim_end().to_string());
                current_body = String::new();
            }
            current_heading = Some(line.trim_start_matches("## ").trim().to_string());
        } else if line.starts_with('#') {
            continue;
        } else if current_heading.is_some() {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }
    if let Some(h) = current_heading.take() {
        sections.insert(h, current_body.trim_end().to_string());
    }
    let mut fm_map = JsonMap::new();
    if let Some(obj) = fm_json.as_object() {
        for (k, v) in obj {
            fm_map.insert(k.clone(), v.clone());
        }
    }
    Ok(SubmitPayload {
        frontmatter: fm_map,
        sections,
    })
}

use crate::commands::ai_new::template_utils::{
    default_template, extract_heading_constraints, load_repo_template,
};
use crate::commands::lua_integration::lua_new_hooks;
use crate::config::Config;
use crate::model::AdrDoc;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

pub fn apply_body_validation(
    cfg: &Config,
    cfg_path: &Option<PathBuf>,
    docs: &[AdrDoc],
    doc_schema: &HashMap<String, String>,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    for doc in docs {
        let doc_id = match &doc.id {
            Some(id) => id,
            None => continue,
        };
        let schema_name = match doc_schema.get(doc_id) {
            Some(name) => name,
            None => continue,
        };
        let schema_cfg = match cfg.schema.iter().find(|s| &s.name == schema_name) {
            Some(sc) => sc,
            None => continue,
        };
        let body_cfg = match schema_cfg.validate.as_ref().and_then(|v| v.body.as_ref()) {
            Some(body) => body,
            None => continue,
        };
        if body_cfg.headings.is_none() && body_cfg.line_count.is_none() {
            continue;
        }

        let heading_policy = body_cfg
            .headings
            .as_ref()
            .and_then(|h| h.heading_check.as_deref())
            .unwrap_or("ignore");
        let heading_severity = body_cfg
            .headings
            .as_ref()
            .and_then(|h| h.severity.as_deref())
            .or(schema_cfg
                .validate
                .as_ref()
                .and_then(|v| v.severity.as_deref()))
            .unwrap_or("error");
        let max_heading_count = body_cfg.headings.as_ref().and_then(|h| h.max_count);

        let line_policy = body_cfg
            .line_count
            .as_ref()
            .and_then(|l| l.scan_policy.as_deref())
            .unwrap_or("on_creation");
        let line_severity = body_cfg
            .line_count
            .as_ref()
            .and_then(|l| l.severity.as_deref())
            .or(schema_cfg
                .validate
                .as_ref()
                .and_then(|v| v.severity.as_deref()))
            .unwrap_or("error");

        let template_raw = resolve_template_note(cfg, cfg_path, schema_name, &doc.title, docs)
            .unwrap_or_else(default_template);
        let expected_headings = extract_heading_constraints(&template_raw);
        let expected_names: Vec<String> =
            expected_headings.iter().map(|h| h.name.clone()).collect();

        let content = match fs::read_to_string(&doc.file) {
            Ok(c) => c,
            Err(err) => {
                push_with_severity(
                    format!(
                        "{}: unable to read file for body validation: {}",
                        doc.file.display(),
                        err
                    ),
                    "warning",
                    errors,
                    warnings,
                );
                continue;
            }
        };
        let actual_sections = collect_note_headings(&content);
        let actual_names: Vec<String> = actual_sections.iter().map(|(n, _)| n.clone()).collect();

        match heading_policy {
            "exact" => {
                if actual_names != expected_names {
                    push_with_severity(
                        format!(
                            "{}: headings do not match template (expected {:?}, found {:?})",
                            doc.file.display(),
                            expected_names,
                            actual_names
                        ),
                        heading_severity,
                        errors,
                        warnings,
                    );
                }
            }
            "missing_only" => {
                let actual_set: BTreeSet<&String> = actual_names.iter().collect();
                let missing: Vec<&String> = expected_names
                    .iter()
                    .filter(|name| !actual_set.contains(*name))
                    .collect();
                if !missing.is_empty() {
                    push_with_severity(
                        format!(
                            "{}: missing required headings: {}",
                            doc.file.display(),
                            missing
                                .iter()
                                .map(|s| s.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ),
                        heading_severity,
                        errors,
                        warnings,
                    );
                }
            }
            _ => {}
        }

        if let Some(max_allowed) = max_heading_count {
            if actual_names.len() > max_allowed {
                push_with_severity(
                    format!(
                        "{}: heading count {} exceeds max {}",
                        doc.file.display(),
                        actual_names.len(),
                        max_allowed
                    ),
                    heading_severity,
                    errors,
                    warnings,
                );
            }
        }

        if line_policy == "on_validate" {
            let section_map: HashMap<&str, usize> = actual_sections
                .iter()
                .map(|(name, lines)| (name.as_str(), *lines))
                .collect();
            for heading in expected_headings {
                if heading.max_lines == 0 {
                    continue;
                }
                if let Some(actual) = section_map.get(heading.name.as_str()) {
                    if *actual as u64 > heading.max_lines {
                        push_with_severity(
                            format!(
                                "{}: heading '{}' exceeds max lines ({} > {})",
                                doc.file.display(),
                                heading.name,
                                actual,
                                heading.max_lines
                            ),
                            line_severity,
                            errors,
                            warnings,
                        );
                    }
                }
            }
        }
    }
}

fn resolve_template_note(
    cfg: &Config,
    cfg_path: &Option<PathBuf>,
    schema: &str,
    title: &str,
    docs: &[AdrDoc],
) -> Option<String> {
    let artifacts = lua_new_hooks(cfg, cfg_path, schema, title, docs);
    if let Some(note) = artifacts.template_note {
        return Some(note);
    }
    if let Some(sc) = cfg.schema.iter().find(|s| s.name == schema) {
        if let Some(toml_note) = sc
            .new
            .as_ref()
            .and_then(|n| n.template.as_ref())
            .and_then(|t| t.note.as_ref())
            .and_then(|p| p.template.clone())
        {
            return Some(toml_note);
        }
    }
    load_repo_template(cfg_path, schema).ok().flatten()
}

fn collect_note_headings(content: &str) -> Vec<(String, usize)> {
    let mut body = content.replace("\r\n", "\n");
    if body.starts_with("---\n") || body.starts_with("+++\n") {
        let delim = if body.starts_with("---\n") {
            "---"
        } else {
            "+++"
        };
        let start = 4;
        let needle = format!("\n{}\n", delim);
        if let Some(end) = body[start..].find(&needle).map(|i| start + i) {
            let body_start = end + needle.len();
            body = body[body_start..].to_string();
        }
    }
    let mut sections: Vec<(String, usize)> = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_lines: usize = 0;
    let mut in_code_block = false;
    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
        }
        if !in_code_block && trimmed.starts_with("## ") {
            if let Some(name) = current_name.take() {
                sections.push((name, current_lines));
            }
            current_name = Some(trimmed.trim_start_matches("## ").trim().to_string());
            current_lines = 0;
        } else if current_name.is_some() {
            current_lines += 1;
        }
    }
    if let Some(name) = current_name.take() {
        sections.push((name, current_lines));
    }
    sections
}

fn push_with_severity(
    message: String,
    severity: &str,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    match severity {
        "warning" => warnings.push(message),
        "ignore" => {}
        _ => errors.push(message),
    }
}

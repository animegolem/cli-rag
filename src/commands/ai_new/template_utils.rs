use anyhow::{Context, Result};
use std::path::PathBuf;

use super::store::HeadingConstraint;

pub fn extract_heading_constraints(template_raw: &str) -> Vec<HeadingConstraint> {
    let mut headings: Vec<HeadingConstraint> = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_max: u64 = 0;
    for line in template_raw.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(name) = current_name.take() {
                headings.push(HeadingConstraint {
                    name,
                    max_lines: if current_max == 0 { 200 } else { current_max },
                });
            }
            current_name = Some(rest.trim().to_string());
            current_max = 0;
        } else if current_name.is_some() {
            if let Some(loc) = extract_loc(line) {
                current_max = loc;
            }
        }
    }
    if let Some(name) = current_name.take() {
        headings.push(HeadingConstraint {
            name,
            max_lines: if current_max == 0 { 200 } else { current_max },
        });
    }
    headings
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

pub fn load_repo_template(cfg_path: &Option<PathBuf>, schema: &str) -> Result<Option<String>> {
    if let Some(cfgp) = cfg_path {
        if let Some(dir) = cfgp.parent() {
            let p = dir
                .join(".cli-rag/templates")
                .join(format!("{}.md", schema));
            if p.exists() {
                return std::fs::read_to_string(&p)
                    .with_context(|| format!("reading template {:?}", p))
                    .map(Some);
            }
        }
    }
    Ok(None)
}

pub fn default_template() -> String {
    "---\n{{frontmatter}}\ncreated_date: {{date}}\nlast_modified: {{date}}\nrelated_files: []\n---\n\n# {{title}}\n\n## Objective\n{{LOC|80}}\n\n## Context\n{{LOC|200}}\n\n## Decision\n{{LOC|120}}\n\n".to_string()
}

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::commands::lua_integration::lua_new_hooks;
use crate::commands::new_helpers::{generate_initial_id, render_template, resolve_destination_dir};
use crate::commands::output::print_json;
use crate::config::Config;
use crate::discovery::docs_with_source;

use super::store::{
    build_constraints, extract_frontmatter_json, DraftRecord, StartResponse, DEFAULT_TTL_SECONDS,
};
use super::utils::{
    determine_filename, generate_draft_id, path_to_string, resolve_project_root, sha256_hex,
};
use crate::cli::OutputFormat;

pub fn start(
    cfg: &Config,
    cfg_path: &Option<PathBuf>,
    schema: String,
    title_opt: Option<String>,
    id_override: Option<String>,
    format: &OutputFormat,
) -> Result<()> {
    super::utils::ensure_json_output(format);
    if cfg.bases.is_empty() {
        return Err(anyhow!("No bases configured; run `cli-rag init` first"));
    }
    let project_root = resolve_project_root(cfg_path)?;
    let drafts_dir = project_root.join(".cli-rag/drafts");
    fs::create_dir_all(&drafts_dir).ok();
    let destination_dir = resolve_destination_dir(cfg, cfg_path, &schema, None)?;

    let (docs, _used_unified) = docs_with_source(cfg, cfg_path)?;
    let mut id = if let Some(explicit) = id_override {
        explicit
    } else {
        generate_initial_id(cfg, &schema, &docs)
    };
    if docs.iter().any(|d| d.id.as_deref() == Some(id.as_str())) {
        return Err(anyhow!("ID {} already exists; pass --id to override", id));
    }
    let mut title = title_opt.unwrap_or_else(|| id.clone());
    if title.trim().is_empty() {
        title = id.clone();
    }

    let (lua_id_override, fm_overrides) = lua_new_hooks(cfg, cfg_path, &schema, &title, &docs);
    if let Some(newid) = lua_id_override {
        id = newid;
    }

    let template_raw = load_schema_template(cfg_path, &schema)?;
    let mut note_template = render_template(template_raw.clone(), &id, &title);
    if let Some(fm_map) = fm_overrides {
        if note_template.starts_with("---\n") {
            if let Some(end) = note_template.find("\n---\n") {
                let fm_content = &note_template[4..end];
                let rest = &note_template[end + 5..];
                if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(fm_content) {
                    use serde_yaml::{Mapping, Value};
                    let mut map = match val {
                        Value::Mapping(m) => m,
                        _ => Mapping::new(),
                    };
                    for (k, v) in fm_map {
                        map.insert(Value::String(k), v);
                    }
                    let yaml = serde_yaml::to_string(&Value::Mapping(map)).unwrap_or_default();
                    let front = format!("---\n{}---\n", yaml);
                    note_template = format!("{}{}", front, rest);
                }
            }
        }
    }
    let seed_frontmatter = extract_frontmatter_json(&note_template)?;
    let constraints = build_constraints(&template_raw, &seed_frontmatter);
    let instructions = format!(
        "Generate content for {} using the reserved id {} and provided headings.",
        schema, id
    );
    let content_hash = sha256_hex(note_template.as_bytes());

    let filename = determine_filename(cfg, &schema, &id, &title);
    let record = DraftRecord {
        draft_id: generate_draft_id(),
        schema: schema.clone(),
        id: id.clone(),
        title: title.clone(),
        filename: filename.clone(),
        base: path_to_string(&destination_dir),
        created_at: chrono::Utc::now().timestamp(),
        ttl_seconds: DEFAULT_TTL_SECONDS,
        note_template: note_template.clone(),
        seed_frontmatter: seed_frontmatter.clone(),
        constraints: constraints.clone(),
        instructions: instructions.clone(),
        content_hash: content_hash.clone(),
        primary_heading: extract_primary_heading_literal(&template_raw),
    };

    let target = record.target_path(&project_root);
    if target.exists() {
        return Err(anyhow!(
            "Target note already exists at {}",
            target.display()
        ));
    }
    let draft_path = record.draft_path(&drafts_dir);
    let record_json = serde_json::to_string_pretty(&record)?;
    fs::write(&draft_path, record_json)
        .with_context(|| format!("writing draft {:?}", draft_path))?;

    let response = StartResponse {
        draft_id: record.draft_id.clone(),
        schema,
        id,
        title,
        filename,
        note_template,
        seed_frontmatter,
        constraints,
        instructions,
        ttl_seconds: DEFAULT_TTL_SECONDS,
        content_hash,
    };
    print_json(&response)?;
    Ok(())
}

fn load_schema_template(cfg_path: &Option<PathBuf>, schema: &str) -> Result<String> {
    if let Some(cfgp) = cfg_path {
        if let Some(dir) = cfgp.parent() {
            let p = dir
                .join(".cli-rag/templates")
                .join(format!("{}.md", schema));
            if p.exists() {
                return fs::read_to_string(&p).with_context(|| format!("reading template {:?}", p));
            }
        }
    }
    Ok("---\nid: {{id}}\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# {{id}}: {{title}}\n\n## Objective\n{{LOC|80}}\n\n## Context\n{{LOC|200}}\n\n## Decision\n{{LOC|120}}\n\n".to_string())
}

fn extract_primary_heading_literal(template_raw: &str) -> String {
    for line in template_raw.lines() {
        if line.starts_with('#') {
            let mut heading = line.trim().to_string();
            if !heading.ends_with('\n') {
                heading.push('\n');
            }
            heading.push('\n');
            return heading;
        }
    }
    String::new()
}

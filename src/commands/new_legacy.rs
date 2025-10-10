use anyhow::Result;
use std::fs;
use std::path::PathBuf;

use crate::cli::OutputFormat;
use crate::commands::ai_new::template_utils::{default_template, load_repo_template};
use crate::commands::new_helpers::{
    generate_initial_id, render_filename_template, render_template, resolve_destination_dir,
    TemplateVars,
};
use crate::config::Config;
use crate::discovery::docs_with_source;

pub fn run(
    cfg: &Config,
    cfg_path: &Option<PathBuf>,
    schema: String,
    title_opt: Option<String>,
    id_opt: Option<String>,
    filename_tpl_override: Option<String>,
    _format: &OutputFormat,
) -> Result<()> {
    eprintln!("'cli-rag new' is deprecated; prefer 'cli-rag ai new start|submit'.");

    let (docs, _used_unified) = docs_with_source(cfg, cfg_path)?;
    let mut id = id_opt.unwrap_or_else(|| generate_initial_id(cfg, &schema, &docs));
    if id.trim().is_empty() {
        id = generate_initial_id(cfg, &schema, &docs);
    }
    let title = title_opt.unwrap_or_else(|| id.clone());

    // Resolve project root (directory containing the config, or CWD)
    let project_root = if let Some(p) = cfg_path {
        p.parent()
            .map(|d| d.to_path_buf())
            .unwrap_or(std::env::current_dir()?)
    } else {
        std::env::current_dir()?
    };
    let base_dir = resolve_destination_dir(cfg, cfg_path, &schema, None)?;

    // Determine filename
    let filename = if let Some(tpl) = filename_tpl_override.as_deref() {
        render_filename_template(tpl, &id, &title, &schema)
    } else {
        // Mirror ai_new filename resolution without depending on its private module
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
            render_filename_template(&tpl, &id, &title, &schema)
        } else {
            format!("{}.md", id)
        }
    };

    // Choose template: repo template or minimal default
    let template_source = load_repo_template(cfg_path, &schema)?.unwrap_or_else(default_template);
    let vars = TemplateVars {
        id: &id,
        title: &title,
        schema: &schema,
        filename: &filename,
    };
    let note_body = render_template(template_source, &vars);

    let target = base_dir.join(filename);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&target, note_body)?;
    println!(
        "{}",
        target
            .strip_prefix(&project_root)
            .unwrap_or(&target)
            .display()
    );
    Ok(())
}

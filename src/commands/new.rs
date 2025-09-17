use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::commands::lua_integration::lua_new_hooks;
use crate::commands::new_helpers::{
    generate_initial_id, render_filename_template, render_template,
};
use crate::config::Config;
use crate::discovery::docs_with_source;
use crate::util::try_open_editor;
#[allow(unused_imports)]
use mlua::{Function as LuaFunction, Table as LuaTable};

#[allow(clippy::too_many_arguments)]
pub fn run(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    schema: String,
    title_opt: Option<String>,
    filename_template: Option<String>,
    dest_base: Option<std::path::PathBuf>,
    normalize_title: bool,
    print_body: bool,
    dry_run: bool,
    edit: bool,
) -> Result<()> {
    if cfg.bases.is_empty() {
        return Err(anyhow!(
            "No bases configured; please run `cli-rag init` first"
        ));
    }
    // Determine destination base
    let base = if let Some(dest) = dest_base {
        // Try to find matching configured base (canonicalized)
        let dest_c = std::fs::canonicalize(&dest).unwrap_or(dest.clone());
        let mut found: Option<&std::path::PathBuf> = None;
        for b in &cfg.bases {
            let bc = std::fs::canonicalize(b).unwrap_or(b.clone());
            if bc == dest_c {
                found = Some(b);
                break;
            }
        }
        found.ok_or_else(|| {
            anyhow!(
                "--dest-base does not match any configured base: {}",
                dest_c.display()
            )
        })?
    } else {
        &cfg.bases[0]
    };
    let (docs, _used_unified) = docs_with_source(cfg, cfg_path)?;
    // Determine id based on schema.new.id_generator if present
    let mut id = generate_initial_id(cfg, &schema, &docs);
    let mut title = title_opt.unwrap_or_else(|| id.clone());
    if normalize_title {
        use heck::ToTitleCase;
        title = title.to_title_case();
    }

    // Lua hooks: id_generator(schema, ctx) and render_frontmatter(schema, title?, ctx)
    let (id_override, fm_overrides) = lua_new_hooks(cfg, cfg_path, &schema, &title, &docs);
    if let Some(newid) = id_override {
        id = newid;
    }

    // Find template under config dir if available
    let mut tmpl_path: Option<PathBuf> = None;
    if let Some(cfgp) = cfg_path {
        if let Some(dir) = cfgp.parent() {
            let p = dir
                .join(".cli-rag/templates")
                .join(format!("{}.md", schema));
            if p.exists() {
                tmpl_path = Some(p);
            }
        }
    }
    let body_raw = if let Some(p) = tmpl_path {
        fs::read_to_string(&p).with_context(|| format!("reading template {:?}", p))?
    } else {
        String::from(
            "---\nid: {{id}}\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# {{id}}: {{title}}\n\n",
        )
    };
    let mut body = render_template(body_raw, &id, &title);
    // If Lua provided frontmatter overrides, merge into the YAML block
    if let Some(fm_map) = fm_overrides {
        if body.starts_with("---\n") {
            if let Some(end) = body.find("\n---\n") {
                let fm_content = &body[4..end];
                let rest = &body[end + 5..];
                if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(fm_content) {
                    use serde_yaml::{Mapping, Value};
                    let mut map = match val {
                        Value::Mapping(m) => m,
                        _ => Mapping::new(),
                    };
                    // apply overrides map
                    for (k, v) in fm_map {
                        map.insert(Value::String(k), v);
                    }
                    let yaml = serde_yaml::to_string(&Value::Mapping(map)).unwrap_or_default();
                    let front = format!("---\n{}---\n", yaml);
                    body = format!("{}{}", front, rest);
                }
            }
        }
    }
    // Determine filename template: CLI flag > schema.new.filename_template > schema.filename_template
    let schema_tpl_from_new: Option<String> = if filename_template.is_none() {
        cfg.schema
            .iter()
            .find(|s| s.name == schema)
            .and_then(|s| s.new.as_ref())
            .and_then(|n| n.filename_template.clone())
    } else {
        None
    };
    let schema_tpl_legacy: Option<String> =
        if filename_template.is_none() && schema_tpl_from_new.is_none() {
            cfg.schema
                .iter()
                .find(|s| s.name == schema)
                .and_then(|s| s.filename_template.clone())
        } else {
            None
        };
    let tpl_eff: Option<String> = filename_template
        .or(schema_tpl_from_new)
        .or(schema_tpl_legacy);
    let initial_name = if let Some(tpl) = &tpl_eff {
        render_filename_template(tpl, &id, &title, &schema)
    } else {
        format!("{}.md", id)
    };
    let mut out_path = base.join(&initial_name);

    if print_body {
        print!("{}", body);
        return Ok(());
    }
    if dry_run {
        println!("Would write {}", out_path.display());
        println!("Id: {}", id);
        println!("Title: {}", title);
        println!("Preview:\n{}", body);
        return Ok(());
    }
    if edit {
        // Edit in a temporary file first; only persist if the user saves
        let mut tmp = std::env::temp_dir();
        tmp.push(format!(".cli-rag-new-{}.md", id));
        fs::write(&tmp, &body).with_context(|| format!("writing temp note {:?}", tmp))?;
        let before_mtime: Option<SystemTime> = fs::metadata(&tmp).and_then(|m| m.modified()).ok();
        if let Err(e) = try_open_editor(&tmp) {
            eprintln!("Note: could not open editor automatically: {}", e);
            return Ok(());
        }
        let after_mtime: Option<SystemTime> = fs::metadata(&tmp).and_then(|m| m.modified()).ok();
        let edited = match (before_mtime, after_mtime) {
            (Some(a), Some(b)) => b > a,
            _ => false,
        };
        let final_body = fs::read_to_string(&tmp).unwrap_or_default();
        // Decide whether to persist: if file content changed or mtime increased
        if !edited && final_body == body {
            // Treat as cancelled; do not create note
            let _ = fs::remove_file(&tmp);
            println!("Cancelled (no changes saved)");
            return Ok(());
        }
        // Ensure we don't overwrite an existing file; bump numeric suffix if needed
        if out_path.exists() {
            let prefix = schema.clone();
            let re = Regex::new(&format!(r"^{}-(\d+)$", regex::escape(&prefix))).unwrap();
            let mut n: usize = 1;
            if let Some(caps) = re.captures(&id) {
                if let Some(m) = caps.get(1) {
                    n = m.as_str().parse::<usize>().unwrap_or(1);
                }
            }
            loop {
                n += 1;
                let newid = format!("{}-{:03}", prefix, n);
                // Recompute filename if template provided
                let cand_name = if let Some(tpl) = &tpl_eff {
                    render_filename_template(tpl, &newid, &title, &schema)
                } else {
                    format!("{}.md", newid)
                };
                let candidate = base.join(&cand_name);
                if !candidate.exists() {
                    out_path = candidate;
                    break;
                }
            }
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&out_path, final_body).with_context(|| format!("writing note {:?}", out_path))?;
        let _ = fs::remove_file(&tmp);
        println!("Wrote {}", out_path.display());
        Ok(())
    } else {
        // Non-edit path: write immediately
        // Ensure we don't overwrite an existing file; bump numeric suffix if needed
        if out_path.exists() {
            let prefix = schema.clone();
            let re = Regex::new(&format!(r"^{}-(\d+)$", regex::escape(&prefix))).unwrap();
            let mut n: usize = 1;
            if let Some(caps) = re.captures(&id) {
                if let Some(m) = caps.get(1) {
                    n = m.as_str().parse::<usize>().unwrap_or(1);
                }
            }
            loop {
                n += 1;
                let newid = format!("{}-{:03}", prefix, n);
                let cand_name = if let Some(tpl) = &tpl_eff {
                    render_filename_template(tpl, &newid, &title, &schema)
                } else {
                    format!("{}.md", newid)
                };
                let candidate = base.join(&cand_name);
                if !candidate.exists() {
                    out_path = candidate;
                    break;
                }
            }
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::write(&out_path, body).with_context(|| format!("writing note {:?}", out_path))?;
        println!("Wrote {}", out_path.display());
        Ok(())
    }
}

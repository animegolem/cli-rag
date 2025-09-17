use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::commands::lua_integration::lua_new_hooks;
use crate::config::Config;
use crate::discovery::docs_with_source;
use crate::util::try_open_editor;
#[allow(unused_imports)]
use mlua::{Function as LuaFunction, Table as LuaTable};
use uuid::Uuid;

fn render_template(mut s: String, id: &str, title: &str) -> String {
    s = s.replace("{{id}}", id);
    s = s.replace("{{title}}", title);
    let now = chrono::Local::now();
    s = s.replace("{{date}}", &now.format("%Y-%m-%d").to_string());
    s = s.replace("{{time}}", &now.format("%H:%M").to_string());
    // Minimal LOC token: {{LOC|N}} -> N blank lines
    // We do a simple scan for patterns; non-greedy
    let re = Regex::new(r"\{\{LOC\|(\d+)\}\}").ok();
    if let Some(re) = re {
        s = re
            .replace_all(&s, |caps: &regex::Captures| {
                let n: usize = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                "\n".repeat(n)
            })
            .to_string();
    }
    // Frontmatter injection token: ((frontmatter)) -> YAML key-value lines (no --- guards)
    if s.contains("((frontmatter))") {
        let fm = format!(
            "id: {}\n{}{}{}",
            id, "tags: []\n", "status: draft\n", "depends_on: []\n"
        );
        s = s.replace("((frontmatter))", &fm);
    }
    // Merge/ensure YAML frontmatter keys (between first pair of --- guards)
    if s.starts_with("---\n") {
        if let Some(end) = s.find("\n---\n") {
            let fm_content = &s[4..end];
            let rest = &s[end + 5..];
            if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(fm_content) {
                use serde_yaml::{Mapping, Value};
                let mut map = match val {
                    Value::Mapping(m) => m,
                    _ => Mapping::new(),
                };
                // System precedence: id always set to computed
                map.insert(Value::String("id".into()), Value::String(id.into()));
                map.entry(Value::String("tags".into()))
                    .or_insert_with(|| Value::Sequence(vec![]));
                map.entry(Value::String("status".into()))
                    .or_insert_with(|| Value::String("draft".into()));
                map.entry(Value::String("depends_on".into()))
                    .or_insert_with(|| Value::Sequence(vec![]));
                let yaml = serde_yaml::to_string(&Value::Mapping(map)).unwrap_or_default();
                let front = format!("---\n{}---\n", yaml);
                s = format!("{}{}", front, rest);
            }
        }
    }
    s
}

fn compute_next_id(prefix: &str, docs: &Vec<crate::model::AdrDoc>, padding: usize) -> String {
    // prefix is used verbatim before the numeric counter (e.g., "ADR-", "RFC-")
    let re = Regex::new(&format!(r"^{}(\d+)$", regex::escape(prefix))).unwrap();
    let mut max_n: usize = 0;
    for d in docs {
        if let Some(id) = &d.id {
            if let Some(caps) = re.captures(id) {
                if let Some(m) = caps.get(1) {
                    if let Ok(n) = m.as_str().parse::<usize>() {
                        if n > max_n {
                            max_n = n;
                        }
                    }
                }
            }
        }
    }
    format!("{}{:0width$}", prefix, max_n + 1, width = padding)
}

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
    let schema_cfg = cfg.schema.iter().find(|s| s.name == schema);
    let mut id = if let Some(scfg) = schema_cfg
        .and_then(|s| s.new.as_ref())
        .and_then(|n| n.id_generator.as_ref())
    {
        let strategy = scfg.strategy.as_str();
        match strategy {
            "increment" => {
                let prefix = scfg
                    .prefix
                    .clone()
                    .unwrap_or_else(|| format!("{}-", &schema));
                let padding = scfg.padding.unwrap_or(3);
                compute_next_id(&prefix, &docs, padding)
            }
            "datetime" => {
                let prefix = scfg
                    .prefix
                    .clone()
                    .unwrap_or_else(|| format!("{}-", &schema));
                let ts = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
                format!("{}{}", prefix, ts)
            }
            "uuid" => {
                let prefix = scfg
                    .prefix
                    .clone()
                    .unwrap_or_else(|| format!("{}-", &schema));
                let u = Uuid::new_v4().simple().to_string();
                format!("{}{}", prefix, u)
            }
            _ => compute_next_id(&format!("{}-", &schema), &docs, 3),
        }
    } else {
        // default increment with schema- prefix
        compute_next_id(&format!("{}-", &schema), &docs, 3)
    };
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
    // Compute target filename with optional template
    fn sanitize_filename_component(s: &str) -> String {
        let out = s.replace('/', "-").replace(['\n', '\r'], " ");
        out.trim().to_string()
    }
    fn render_filename_template(tpl: &str, id: &str, title: &str, schema: &str) -> String {
        use heck::{
            ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase,
        };
        let now = chrono::Local::now();
        let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_.]+)\s*(?:\|\s*([^}]+))?\s*\}\}").unwrap();
        let rendered = re
            .replace_all(tpl, |caps: &regex::Captures| {
                let var = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let modifier = caps.get(2).map(|m| m.as_str().trim());
                let raw = match var {
                    "id" => id.to_string(),
                    "title" => title.to_string(),
                    "now" => now.format("%Y-%m-%dT%H:%M:%S").to_string(),
                    "schema.name" => schema.to_string(),
                    _ => String::new(),
                };
                if let Some(mods) = modifier {
                    match mods {
                        "kebab-case" => raw.to_kebab_case(),
                        "snake_case" => raw.to_snake_case(),
                        "SCREAMING_SNAKE_CASE" => raw.to_shouty_snake_case(),
                        "camelCase" => raw.to_lower_camel_case(),
                        "PascalCase" => raw.to_upper_camel_case(),
                        m if m.starts_with("date:") => {
                            // Support date:"%Y-%m-%d" or date:'%Y-%m-%d'
                            let pat = m.splitn(2, ':').nth(1).unwrap_or("").trim();
                            let pat = pat.trim_matches('"').trim_matches('\'');
                            now.format(pat).to_string()
                        }
                        _ => raw,
                    }
                } else {
                    raw
                }
            })
            .to_string();
        let mut f = sanitize_filename_component(&rendered);
        if !f.ends_with(".md") {
            f.push_str(".md");
        }
        f
    }
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
            let prefix = schema;
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

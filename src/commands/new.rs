use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::discovery::docs_with_source;
use crate::util::try_open_editor;

fn render_template(mut s: String, id: &str, title: &str) -> String {
    s = s.replace("{{id}}", id);
    s = s.replace("{{title}}", title);
    // Minimal placeholders for date/time without adding chrono clock feature
    s = s.replace("{{date}}", "");
    s = s.replace("{{time}}", "");
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
    s
}

fn compute_next_id(prefix: &str, docs: &Vec<crate::model::AdrDoc>) -> String {
    let re = Regex::new(&format!(r"^{}-(\d+)$", regex::escape(prefix))).unwrap();
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
    format!("{}-{:03}", prefix, max_n + 1)
}

pub fn run(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    schema: String,
    title_opt: Option<String>,
    print_body: bool,
    dry_run: bool,
    edit: bool,
) -> Result<()> {
    if cfg.bases.is_empty() {
        return Err(anyhow!(
            "No bases configured; please run `cli-rag init` first"
        ));
    }
    let base = &cfg.bases[0];
    let (docs, _used_unified) = docs_with_source(cfg, cfg_path)?;
    let id = compute_next_id(&schema, &docs);
    let title = title_opt.unwrap_or_else(|| id.clone());

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
    let body = render_template(body_raw, &id, &title);
    let out_path = base.join(format!("{}.md", id));

    if print_body {
        print!("{}", body);
        return Ok(());
    }
    if dry_run {
        println!("Would write {}", out_path.display());
        println!("Id: {}", id);
        println!("Title: {}", title);
        return Ok(());
    }
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&out_path, body).with_context(|| format!("writing note {:?}", out_path))?;
    println!("Wrote {}", out_path.display());
    if edit {
        if let Err(e) = try_open_editor(&out_path) {
            eprintln!("Note: could not open editor automatically: {}", e);
        }
    }
    Ok(())
}

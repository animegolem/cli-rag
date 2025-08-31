use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::{find_config_upwards, write_template, TEMPLATE};
use crate::util::try_open_editor;

fn schema_block(name: &str) -> String {
    let upper = name.to_string();
    let patt = format!("{}-*.md", upper);
    format!(
        "# config_version = 1\n\n[[schema]]\nname = \"{upper}\"\nfile_patterns = [\"{patt}\"]\nrequired = [\"id\", \"tags\", \"status\", \"depends_on\"]\nunknown_policy = \"ignore\"\n# cycle_policy = \"warn\"   # warn | error | ignore\n\n[schema.rules.status]\nallowed = [\n  \"draft\", \"incomplete\", \"proposed\", \"accepted\",\n  \"complete\", \"design\", \"legacy-reference\", \"superseded\"\n]\nseverity = \"error\"\n"
    )
}

fn append_inline_schema(cfg_path: &Path, name: &str) -> Result<()> {
    let mut content =
        fs::read_to_string(cfg_path).with_context(|| format!("reading {:?}", cfg_path))?;
    if content.contains(&format!("name = \"{}\"", name)) {
        // Already present; no-op
        return Ok(());
    }
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push('\n');
    content.push_str(&schema_block(name));
    fs::write(cfg_path, content).with_context(|| format!("writing {:?}", cfg_path))?;
    Ok(())
}

fn write_separate_schema(cfg_path: &Path, name: &str, force: bool) -> Result<()> {
    let cfg_dir = cfg_path.parent().unwrap_or(Path::new("."));
    let templates_dir = cfg_dir.join(".cli-rag/templates");
    fs::create_dir_all(&templates_dir).ok();
    let file_path = templates_dir.join(format!("{}.toml", name));
    if !file_path.exists() || force {
        let mut body = String::new();
        // Comments ok; imports require only [[schema]] at top-level
        body.push_str(&schema_block(name));
        fs::write(&file_path, body).with_context(|| format!("writing template {:?}", file_path))?;
    }
    // Also write a minimal note body template alongside (non-config file)
    let body_path = templates_dir.join(format!("{}.md", name));
    if !body_path.exists() || force {
        let body =
            "---\nid: {{id}}\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# {{id}}: {{title}}\n\n";
        fs::write(&body_path, body)
            .with_context(|| format!("writing note stub {:?}", body_path))?;
    }
    // Update import list in the main config
    let rel = Path::new(".cli-rag/templates").join(format!("{}.toml", name));
    let rel_str = rel.to_string_lossy().to_string();
    let s = fs::read_to_string(cfg_path).with_context(|| format!("reading {:?}", cfg_path))?;
    let mut tv: toml::Value =
        toml::from_str(&s).with_context(|| format!("parsing TOML {:?}", cfg_path))?;
    let import_entry = tv
        .get_mut("import")
        .and_then(|v| v.as_array_mut())
        .map(|arr| {
            if !arr.iter().any(|x| x.as_str() == Some(&rel_str)) {
                arr.push(toml::Value::String(rel_str.clone()));
            }
        });
    if import_entry.is_none() {
        // Create import array
        if let Some(table) = tv.as_table_mut() {
            table.insert(
                "import".into(),
                toml::Value::Array(vec![toml::Value::String(rel_str.clone())]),
            );
        }
    }
    let out = toml::to_string_pretty(&tv).unwrap_or(s);
    fs::write(cfg_path, out).with_context(|| format!("updating {:?}", cfg_path))?;
    Ok(())
}

pub fn run(
    path: Option<PathBuf>,
    force: bool,
    print_template: bool,
    silent: bool,
    schema: Option<String>,
    separate: bool,
) -> Result<()> {
    let target = path.unwrap_or_else(|| PathBuf::from(".cli-rag.toml"));
    if print_template {
        print!("{}", TEMPLATE);
        return Ok(());
    }
    let existed = target.exists();
    if existed && !force {
        eprintln!(
            "Config exists: {} (not overwriting; use --force to rewrite)",
            target.display()
        );
    }
    if let Some(parent_cfg) = find_config_upwards(&None) {
        // Warn if creating a new config while another exists in an ancestor directory
        if parent_cfg != target {
            eprintln!(
                "Warning: a parent config exists at {} and may be shadowed by creating one here",
                parent_cfg.display()
            );
        }
    }
    if !existed || force {
        write_template(&target)?;
        println!("Wrote {}", target.display());
    }
    // Optional schema scaffolding
    if let Some(name) = schema {
        if separate {
            write_separate_schema(&target, &name, force)?;
            println!(
                "Scaffolded schema '{}' in .cli-rag/templates/{}.toml and added to import",
                name, name
            );
        } else {
            append_inline_schema(&target, &name)?;
            println!("Appended schema '{}' to {}", name, target.display());
        }
    }
    if !silent {
        if let Err(e) = try_open_editor(&target) {
            eprintln!("Note: could not open editor automatically: {}", e);
        }
    }
    Ok(())
}

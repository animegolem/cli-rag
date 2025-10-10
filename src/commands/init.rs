use anyhow::{bail, Context, Result};
use std::io::{self, IsTerminal, Write};
use std::path::PathBuf;

use crate::commands::init_support as sup;
use crate::config::find_config_upwards;
use crate::util::try_open_editor;
use serde_json::json;
use sup::InitOutcome;

const PRESET_ENV: &str = "CLI_RAG_INIT_CHOICE";

#[derive(Clone, Copy)]
enum Preset {
    Project,
    Generic,
}

#[allow(clippy::too_many_arguments)]
pub fn run(
    path: Option<PathBuf>,
    force: bool,
    print_template: bool,
    silent: bool,
    schema: Option<String>,
    separate: bool,
    preset: Option<String>,
    dry_run: bool,
    json: bool,
) -> Result<()> {
    let target = path.unwrap_or_else(|| PathBuf::from(".cli-rag.toml"));

    if print_template {
        print!("{}", sup::PROJECT_PRESET_CONFIG);
        return Ok(());
    }

    if let Some(parent_cfg) = find_config_upwards(&None) {
        if parent_cfg != target {
            eprintln!(
                "Warning: a parent config exists at {} and may be shadowed by creating one here",
                parent_cfg.display()
            );
        }
    }

    let config_exists = target.exists();
    let schema_only_mode = schema.is_some() && config_exists && preset.is_none() && !force;
    let mut should_open_editor = false;

    let mut outcome = if schema_only_mode {
        InitOutcome::new("schema", dry_run)
    } else {
        let preset_choice = resolve_preset(preset, force)?;
        let result = match preset_choice {
            Preset::Project => sup::init_project(&target, force, dry_run)?,
            Preset::Generic => {
                let msg = "Generic preset not implemented (coming in 1.0)";
                if json {
                    let payload = json!({
                        "protocolVersion": crate::protocol::PROTOCOL_VERSION,
                        "preset": "generic",
                        "created": [],
                        "updated": [],
                        "warnings": [msg],
                        "cancelled": true,
                    });
                    println!("{}", serde_json::to_string_pretty(&payload)?);
                    return Ok(());
                } else {
                    bail!("{msg}");
                }
            }
        };
        should_open_editor = true;
        result
    };

    if let Some(name) = schema.as_deref() {
        let config_changed =
            sup::add_schema(&target, name, separate, force, dry_run, &mut outcome)?;
        if config_changed {
            should_open_editor = true;
        }
    }

    if should_open_editor && !dry_run && !silent && !outcome.cancelled {
        if let Err(err) = try_open_editor(&target) {
            eprintln!("Note: could not open editor automatically: {}", err);
        }
    }

    if dry_run {
        outcome.add_warning("dry_run");
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&outcome.to_json())?);
        return Ok(());
    }

    if outcome.cancelled {
        println!("Init cancelled; existing config preserved.");
        return Ok(());
    }

    if dry_run {
        if outcome.created.is_empty() && outcome.updated.is_empty() {
            println!("Dry run complete; no changes required.");
        } else {
            println!("Dry run complete. Files that would be created:");
            for path in &outcome.created {
                println!("  create {}", path);
            }
            for path in &outcome.updated {
                println!("  update {}", path);
            }
        }
    } else {
        for path in &outcome.created {
            println!("Created {}", path);
        }
        for path in &outcome.updated {
            println!("Updated {}", path);
        }
    }

    Ok(())
}

// helper moved to init_support.rs: init_project

fn resolve_preset(preset: Option<String>, force: bool) -> Result<Preset> {
    if let Some(name) = preset {
        return parse_preset(&name).ok_or_else(|| anyhow::anyhow!("Unknown preset '{}'", name));
    }
    if let Ok(env_choice) = std::env::var(PRESET_ENV) {
        if let Some(p) = parse_preset(&env_choice) {
            return Ok(p);
        }
    }
    if force || !io::stdin().is_terminal() {
        return Ok(Preset::Project);
    }
    prompt_preset()
}

fn parse_preset(input: &str) -> Option<Preset> {
    match input.trim().to_lowercase().as_str() {
        "1" | "project" | "proj" => Some(Preset::Project),
        "2" | "generic" => Some(Preset::Generic),
        _ => None,
    }
}

fn prompt_preset() -> Result<Preset> {
    let mut input = String::new();
    loop {
        println!(
            "INIT: Select from the following:\n\
             1) Pre-defined notes for in-repo project management (recommended)\n\
             2) A broadly applicable preset as a starting point (coming in 1.0)\n\
             You may also provide your own TOML or Lua."
        );
        print!("> ");
        io::stdout().flush().ok();
        input.clear();
        io::stdin()
            .read_line(&mut input)
            .context("reading preset selection")?;
        if let Some(p) = parse_preset(&input) {
            return Ok(p);
        }
        println!("Please enter 1 or 2.");
    }
}

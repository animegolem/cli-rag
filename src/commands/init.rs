use anyhow::Result;
use std::path::PathBuf;

use crate::config::{find_config_upwards, write_template, TEMPLATE};
use crate::util::try_open_editor;

pub fn run(path: Option<PathBuf>, force: bool, print_template: bool, silent: bool) -> Result<()> {
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
    if !silent {
        if let Err(e) = try_open_editor(&target) {
            eprintln!("Note: could not open editor automatically: {}", e);
        }
    }
    Ok(())
}

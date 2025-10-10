use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn write_template(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    let mut f = fs::File::create(path).with_context(|| format!("creating {:?}", path))?;
    f.write_all(TEMPLATE.as_bytes())?;
    Ok(())
}

pub const TEMPLATE: &str = r#"# Repo-local CLI config (cli-rag)

[config]
#: =============================================================================
#:                            # --- Version --- #
#: =============================================================================
config_version = "0.1"

#: =============================================================================
#:                            # --- SCAN --- #
#: =============================================================================
[config.scan]
#: Project notes live under docs/RAG by default. Adjust as needed.
filepaths = ["docs/RAG"]
#: Index lives alongside the repo config.
index_path = ".cli-rag/index.json"
hash_mode = "mtime"
index_strategy = "content"
ignore_globs = ["**/node_modules/**", "**/dist/**"]
ignore_symlinks = true

#: =============================================================================
#:                            # --- AUTHORING --- #
#: =============================================================================
[config.authoring]
editor = "nvim"
background_watch = true

[config.authoring.destinations]
ADR = "docs/RAG/ADR"

#: =============================================================================
#:                             # --- GRAPH --- #
#: =============================================================================
[config.graph]
depth = 1
include_bidirectional = true

[config.graph.ai]
depth = 1
default_fanout = 5
include_bidirectional = true
neighbor_style = "metadata"
outline_lines = 2

#: =============================================================================
#:                        # --- TEMPLATE MANAGEMENT --- #
#: =============================================================================
[config.templates]
import = [".cli-rag/templates/ADR.toml"]
"#;

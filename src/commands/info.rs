use anyhow::Result;
use std::path::PathBuf;

use crate::cli::OutputFormat;
use crate::commands::output::print_json;
use crate::config::Config;

fn info_json(cfg: &Config, cfg_path: &Option<PathBuf>) -> serde_json::Value {
    let protocol_version = crate::protocol::PROTOCOL_VERSION;
    // Compute project root
    let project_root = cfg_path
        .as_ref()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    // Index path relative to project root (legacy index_relative in config)
    let index_path = project_root.join(&cfg.index_relative);
    let index_exists = index_path.exists();

    // AI index cache path
    let ai_index_path = project_root.join(".cli-rag/cache/ai-index.json");
    let ai_index_exists = ai_index_path.exists();

    // Config metadata (version/deprecated placeholders until versioning lands)
    let cfg_meta = serde_json::json!({
        "path": cfg_path
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<defaults>".into()),
        "version": "0.1",
        "deprecated": false,
    });

    let obj = serde_json::json!({
        "protocolVersion": protocol_version,
        "config": cfg_meta,
        "index": {
            "path": index_path.display().to_string(),
            "exists": index_exists,
        },
        "cache": {
            "aiIndexPath": ai_index_path.display().to_string(),
            "exists": ai_index_exists,
        },
        "capabilities": {
            "watchNdjson": true,
            "aiGet": { "retrievalVersion": 1 },
            "pathLocations": true,
            "aiIndex": true,
            "luaApiVersion": 1
        }
    });
    obj
}

pub fn run(cfg: &Config, cfg_path: &Option<PathBuf>, format: &OutputFormat) -> Result<()> {
    match format {
        OutputFormat::Json | OutputFormat::Ndjson | OutputFormat::Ai => {
            let report = info_json(cfg, cfg_path);
            print_json(&report)?;
        }
        OutputFormat::Plain => {
            let j = info_json(cfg, cfg_path);
            println!("cli-rag info");
            println!(
                "Config: {}",
                j.get("config")
                    .and_then(|c| c.get("path"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            );
            if let Some(index) = j.get("index").and_then(|v| v.as_object()) {
                println!(
                    "Index: {} (exists: {})",
                    index
                        .get("path")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    index
                        .get("exists")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                );
            }
        }
    }
    Ok(())
}

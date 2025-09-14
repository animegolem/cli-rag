use anyhow::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::time::Duration;

use crate::config::Config;
use crate::discovery::incremental_collect_docs;
use crate::index::write_indexes;
use crate::validate::validate_docs;

pub struct WatchArgs {
    pub full_rescan: bool,
    pub debounce_ms: u64,
    pub dry_run: bool,
    pub json: bool,
}

pub fn run_watch(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    args: WatchArgs,
) -> Result<()> {
    // Helper to emit NDJSON event envelopes
    let emit = |event: &str, payload: serde_json::Value| {
        let mut obj = serde_json::json!({
            "event": event,
            "protocolVersion": crate::protocol::PROTOCOL_VERSION,
        });
        if let Some(map) = obj.as_object_mut() {
            if let Some(add) = payload.as_object() {
                for (k, v) in add.iter() {
                    map.insert(k.clone(), v.clone());
                }
            }
        }
        println!("{}", obj);
    };

    if args.json {
        // NDJSON handshake first line
        emit("watch_start", serde_json::json!({}));
        // Ensure it's flushed right away
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }
    // Initial run
    {
        let docs = incremental_collect_docs(cfg, args.full_rescan)?;
        let report = validate_docs(cfg, &docs);
        if args.json {
            emit(
                "validated",
                serde_json::json!({"ok": report.ok, "docCount": docs.len()}),
            );
        }
        if report.ok && !args.dry_run {
            let cfg_dir = cfg_path
                .as_ref()
                .and_then(|p| p.parent())
                .map(|p| p as &std::path::Path);
            write_indexes(cfg, &docs, true, true, cfg_dir)?;
            if args.json {
                // Per-base index written events
                for base in &cfg.bases {
                    let count = docs.iter().filter(|d| d.file.starts_with(base)).count();
                    let path = base.join(&cfg.index_relative);
                    emit(
                        "index_written",
                        serde_json::json!({
                            "path": path.display().to_string(),
                            "count": count
                        }),
                    );
                }
                // Unified index event if cfg_dir is present
                if let Some(dir) = cfg_dir {
                    let path = dir.join(&cfg.index_relative);
                    emit(
                        "index_written",
                        serde_json::json!({
                            "path": path.display().to_string(),
                            "count": docs.len()
                        }),
                    );
                }
            }
        }
        if !report.errors.is_empty() {
            eprintln!("Validation failed:");
            for e in &report.errors {
                eprintln!(" - {}", e);
            }
        }
        if !report.warnings.is_empty() {
            eprintln!("Warnings:");
            for w in &report.warnings {
                eprintln!(" - {}", w);
            }
        }
    }
    // Set up watchers
    let (tx, rx) = std::sync::mpsc::channel::<notify::Result<Event>>();
    let mut _watchers: Vec<RecommendedWatcher> = Vec::new();
    for base in &cfg.bases {
        let txc = tx.clone();
        let mut w = notify::recommended_watcher(move |res| {
            let _ = txc.send(res);
        })?;
        w.watch(base, RecursiveMode::Recursive)?;
        _watchers.push(w);
    }
    let debounce = Duration::from_millis(args.debounce_ms);
    loop {
        // Wait for an event, then debounce
        let _ = rx.recv();
        // Drain burst
        while rx.try_recv().is_ok() {}
        std::thread::sleep(debounce);
        let docs = incremental_collect_docs(cfg, false)?;
        let report = validate_docs(cfg, &docs);
        if args.json {
            emit(
                "validated",
                serde_json::json!({"ok": report.ok, "docCount": docs.len()}),
            );
        }
        if report.ok && !args.dry_run {
            let cfg_dir = cfg_path
                .as_ref()
                .and_then(|p| p.parent())
                .map(|p| p as &std::path::Path);
            write_indexes(cfg, &docs, true, true, cfg_dir)?;
            if args.json {
                for base in &cfg.bases {
                    let count = docs.iter().filter(|d| d.file.starts_with(base)).count();
                    let path = base.join(&cfg.index_relative);
                    emit(
                        "index_written",
                        serde_json::json!({
                            "path": path.display().to_string(),
                            "count": count
                        }),
                    );
                }
                if let Some(dir) = cfg_dir {
                    let path = dir.join(&cfg.index_relative);
                    emit(
                        "index_written",
                        serde_json::json!({
                            "path": path.display().to_string(),
                            "count": docs.len()
                        }),
                    );
                }
            }
        }
        // groups removed per ADR-003d
        if !report.errors.is_empty() {
            eprintln!("Validation failed:");
            for e in &report.errors {
                eprintln!(" - {}", e);
            }
        }
        if !report.warnings.is_empty() {
            eprintln!("Warnings:");
            for w in &report.warnings {
                eprintln!(" - {}", w);
            }
        }
    }
}

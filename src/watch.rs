use anyhow::Result;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::time::Duration;

use crate::config::Config;
use crate::discovery::incremental_collect_docs;
use crate::index::{write_groups_config, write_indexes};
use crate::validate::validate_docs;

pub struct WatchArgs {
    pub full_rescan: bool,
    pub debounce_ms: u64,
    pub dry_run: bool,
    pub write_groups: bool,
    pub json: bool,
}

pub fn run_watch(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    args: WatchArgs,
) -> Result<()> {
    // Initial run
    {
        let docs = incremental_collect_docs(cfg, args.full_rescan)?;
        let report = validate_docs(cfg, &docs);
        if args.json {
            // Emit a validated event
            let ev = crate::protocol::SessionUpdate::Validated {
                ok: report.ok,
                doc_count: docs.len(),
            };
            println!("{}", serde_json::to_string(&ev)?);
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
                    let ev = crate::protocol::SessionUpdate::IndexWritten { path, count };
                    println!("{}", serde_json::to_string(&ev)?);
                }
                // Unified index event if cfg_dir is present
                if let Some(dir) = cfg_dir {
                    let path = dir.join(&cfg.index_relative);
                    let ev = crate::protocol::SessionUpdate::IndexWritten {
                        path,
                        count: docs.len(),
                    };
                    println!("{}", serde_json::to_string(&ev)?);
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
            let ev = crate::protocol::SessionUpdate::Validated {
                ok: report.ok,
                doc_count: docs.len(),
            };
            println!("{}", serde_json::to_string(&ev)?);
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
                    let ev = crate::protocol::SessionUpdate::IndexWritten { path, count };
                    println!("{}", serde_json::to_string(&ev)?);
                }
                if let Some(dir) = cfg_dir {
                    let path = dir.join(&cfg.index_relative);
                    let ev = crate::protocol::SessionUpdate::IndexWritten {
                        path,
                        count: docs.len(),
                    };
                    println!("{}", serde_json::to_string(&ev)?);
                }
            }
        }
        if args.write_groups && !args.dry_run {
            write_groups_config(cfg, &docs)?;
            if args.json {
                for base in &cfg.bases {
                    let path = base.join(&cfg.groups_relative);
                    // Count equals number of groups entries as a heuristic: number of group labels
                    let count = docs.iter().flat_map(|d| d.groups.iter()).count();
                    let ev = crate::protocol::SessionUpdate::GroupsWritten { path, count };
                    println!("{}", serde_json::to_string(&ev)?);
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
}

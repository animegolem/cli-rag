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
}

pub fn run_watch(cfg: &Config, args: WatchArgs) -> Result<()> {
    // Initial run
    {
        let docs = incremental_collect_docs(cfg, args.full_rescan)?;
        let report = validate_docs(cfg, &docs);
        if report.ok && !args.dry_run {
            write_indexes(cfg, &docs, true, true)?;
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
        if report.ok && !args.dry_run {
            write_indexes(cfg, &docs, true, true)?;
        }
        if args.write_groups && !args.dry_run {
            write_groups_config(cfg, &docs)?;
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

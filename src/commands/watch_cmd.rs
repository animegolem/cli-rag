use anyhow::Result;

use crate::config::Config;
use crate::watch::{run_watch, WatchArgs};

pub fn run(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    full_rescan: bool,
    debounce_ms: u64,
    dry_run: bool,
) -> Result<()> {
    run_watch(
        cfg,
        cfg_path,
        WatchArgs {
            full_rescan,
            debounce_ms,
            dry_run,
            write_groups: false,
        },
    )
}

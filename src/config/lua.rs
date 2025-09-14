use std::path::{Path, PathBuf};

use crate::config::schema::OverlayInfo;

fn home_dir() -> Option<PathBuf> {
    if let Ok(h) = std::env::var("HOME") {
        if !h.is_empty() {
            return Some(PathBuf::from(h));
        }
    }
    // Windows fallbacks
    if let Ok(p) = std::env::var("USERPROFILE") {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    None
}

/// Discover Lua overlays without executing them. Honors `no_lua_flag` as well as
/// `CLI_RAG_NO_LUA=1`. Looks for repo-level `.cli-rag.lua` next to the config file
/// and user-level `~/.config/cli-rag/config.lua`.
pub fn discover_overlays(cfg_path: &Option<PathBuf>, no_lua_flag: bool) -> OverlayInfo {
    // Effective disable flag
    let no_lua_env = std::env::var("CLI_RAG_NO_LUA")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if no_lua_flag || no_lua_env {
        return OverlayInfo {
            enabled: false,
            repo_path: None,
            user_path: None,
        };
    }
    let mut repo_path: Option<PathBuf> = None;
    let mut user_path: Option<PathBuf> = None;
    if let Some(cfg) = cfg_path {
        if let Some(dir) = cfg.parent() {
            let cand = dir.join(".cli-rag.lua");
            if cand.exists() && cand.is_file() {
                repo_path = Some(cand);
            }
        }
    }
    if let Some(home) = home_dir() {
        let cand = home.join(Path::new(".config/cli-rag/config.lua"));
        if cand.exists() && cand.is_file() {
            user_path = Some(cand);
        }
    }
    OverlayInfo {
        enabled: repo_path.is_some() || user_path.is_some(),
        repo_path,
        user_path,
    }
}

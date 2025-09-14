use std::path::{Path, PathBuf};

use crate::config::schema::OverlayInfo;
use mlua::{Lua, Table, Value as LuaValue};

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

fn read_file_if_exists(p: &Path) -> Option<String> {
    std::fs::read_to_string(p).ok()
}

fn merge_tables(_base_lua: &Lua, base: &Table, add: &Table) -> mlua::Result<()> {
    for pair in add.clone().pairs::<LuaValue, LuaValue>() {
        let (k, v) = pair?;
        base.set(k, v)?;
    }
    Ok(())
}

/// Load overlay Lua state and return a merged overlay table (repo overlaid by user).
pub fn load_overlay_state(_cfg_path: &Option<PathBuf>, overlays: &OverlayInfo) -> Option<Lua> {
    if !overlays.enabled {
        return None;
    }
    let lua = Lua::new();
    let overlay_tbl = lua.create_table().ok()?;
    // Load repo, then user (user overrides repo)
    if let Some(ref rp) = overlays.repo_path {
        if let Some(code) = read_file_if_exists(rp) {
            let chunk = lua.load(&code).set_name("repo_overlay");
            if let Ok(tbl) = chunk.eval::<Table>() {
                let _ = merge_tables(&lua, &overlay_tbl, &tbl);
            }
        }
    }
    if let Some(ref up) = overlays.user_path {
        if let Some(code) = read_file_if_exists(up) {
            let chunk = lua.load(&code).set_name("user_overlay");
            if let Ok(tbl) = chunk.eval::<Table>() {
                let _ = merge_tables(&lua, &overlay_tbl, &tbl);
            }
        }
    }
    // set as global 'overlay'
    {
        let globals = lua.globals();
        let _ = globals.set("overlay", overlay_tbl);
    }
    Some(lua)
}

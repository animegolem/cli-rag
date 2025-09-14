use crate::config::Config;
use crate::model::AdrDoc;
use mlua::{Function as LuaFunction, Lua, Table as LuaTable};
use std::collections::BTreeMap;

fn load_overlay(lua: &Lua) -> Option<LuaTable<'_>> {
    let globals = lua.globals();
    globals.get::<_, LuaTable>("overlay").ok()
}

pub fn lua_validate_augment(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    docs: &Vec<AdrDoc>,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    if !cfg.overlays.enabled {
        return;
    }
    if let Some(lua) = crate::config::lua::load_overlay_state(cfg_path, &cfg.overlays) {
        if let Some(overlay) = load_overlay(&lua) {
            if let Ok(func) = overlay.get::<_, LuaFunction>("validate") {
                let schema_sets = crate::config::build_schema_sets(cfg);
                let infer_schema = |path: &std::path::Path| -> String {
                    let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    for (sc, set) in &schema_sets {
                        if set.is_match(fname) {
                            return sc.name.clone();
                        }
                    }
                    "UNKNOWN".into()
                };
                for d in docs {
                    let note = lua.create_table().unwrap();
                    if let Some(ref id) = d.id {
                        let _ = note.set("id", id.clone());
                    }
                    let _ = note.set("title", d.title.clone());
                    let _ = note.set("schema", infer_schema(&d.file));
                    let _ = note.set("path", d.file.display().to_string());
                    if let Ok(body) = std::fs::read_to_string(&d.file) {
                        let _ = note.set("body", body);
                    }
                    let fm_tbl: LuaTable = lua.create_table().unwrap();
                    for (k, v) in &d.fm {
                        if let Some(s) = v.as_str() {
                            let _ = fm_tbl.set(k.as_str(), s.to_string());
                        } else if let Some(arr) = v.as_sequence() {
                            let seq = lua.create_table().unwrap();
                            let mut idx = 1;
                            for it in arr {
                                if let Some(ss) = it.as_str() {
                                    let _ = seq.set(idx, ss.to_string());
                                    idx += 1;
                                }
                            }
                            let _ = fm_tbl.set(k.as_str(), seq);
                        }
                    }
                    let _ = note.set("frontmatter", fm_tbl);
                    let ctx = lua.create_table().unwrap();
                    let _ = ctx.set("luaApiVersion", 1);
                    if let Ok(mlua::Value::Table(t)) = func.call::<_, mlua::Value>((note, ctx)) {
                        if let Ok(mlua::Value::Table(arr)) = t.get::<_, mlua::Value>("diagnostics")
                        {
                            for pair in arr.sequence_values::<mlua::Value>() {
                                if let Ok(mlua::Value::Table(dv)) = pair {
                                    let sev = dv
                                        .get::<_, String>("severity")
                                        .unwrap_or_else(|_| "warning".into());
                                    let code = dv
                                        .get::<_, String>("code")
                                        .unwrap_or_else(|_| "LUA".into());
                                    let msg =
                                        dv.get::<_, String>("msg").unwrap_or_else(|_| "".into());
                                    let line = dv.get::<_, i64>("line").ok();
                                    let text = if let Some(l) = line {
                                        format!("LUA[{}]: {} (line {})", code, msg, l)
                                    } else {
                                        format!("LUA[{}]: {}", code, msg)
                                    };
                                    if sev == "error" {
                                        errors.push(text);
                                    } else {
                                        warnings.push(text);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn lua_new_hooks(
    cfg: &Config,
    cfg_path: &Option<std::path::PathBuf>,
    schema: &str,
    title: &str,
    docs: &[AdrDoc],
) -> (Option<String>, Option<BTreeMap<String, serde_yaml::Value>>) {
    if !cfg.overlays.enabled {
        return (None, None);
    }
    if let Some(lua) = crate::config::lua::load_overlay_state(cfg_path, &cfg.overlays) {
        if let Some(overlay) = load_overlay(&lua) {
            let ctx = lua.create_table().unwrap();
            let req = lua.create_table().unwrap();
            let _ = req.set("title", title.to_string());
            let _ = ctx.set("request", req);
            let idx_tbl = lua.create_table().unwrap();
            let docs_clone = docs.to_owned();
            let func = lua
                .create_function(move |_, prefix: String| {
                    let re =
                        regex::Regex::new(&format!(r"^{}-(\d+)$", regex::escape(&prefix))).unwrap();
                    let mut max_n: i64 = 0;
                    for d in &docs_clone {
                        if let Some(ref oid) = d.id {
                            if let Some(caps) = re.captures(oid) {
                                if let Some(m) = caps.get(1) {
                                    if let Ok(n) = m.as_str().parse::<i64>() {
                                        if n > max_n {
                                            max_n = n;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Ok(max_n + 1)
                })
                .unwrap();
            let _ = idx_tbl.set("next_numeric_id", func);
            let _ = ctx.set("index", idx_tbl);

            // id_generator
            if let Ok(f) = overlay.get::<_, LuaFunction>("id_generator") {
                if let Ok(mlua::Value::Table(t)) =
                    f.call::<_, mlua::Value>((schema.to_string(), ctx.clone()))
                {
                    if let Ok(newid) = t.get::<_, String>("id") {
                        // optional fm next
                        let fm = render_fm(&overlay, &lua, schema, title, ctx);
                        return (Some(newid), fm);
                    }
                }
            }
            // fm only
            let fm = render_fm(&overlay, &lua, schema, title, ctx);
            return (None, fm);
        }
    }
    (None, None)
}

fn render_fm(
    overlay: &LuaTable,
    _lua: &Lua,
    schema: &str,
    title: &str,
    ctx: LuaTable,
) -> Option<BTreeMap<String, serde_yaml::Value>> {
    if let Ok(fm_func) = overlay.get::<_, LuaFunction>("render_frontmatter") {
        if let Ok(mlua::Value::Table(t)) =
            fm_func.call::<_, mlua::Value>((schema.to_string(), title.to_string(), ctx))
        {
            let mut map: BTreeMap<String, serde_yaml::Value> = BTreeMap::new();
            for (k, v) in t.pairs::<mlua::Value, mlua::Value>().flatten() {
                if let mlua::Value::String(ks) = k {
                    let key = ks.to_str().unwrap_or("").to_string();
                    if key.is_empty() {
                        continue;
                    }
                    let yaml_v = match v {
                        mlua::Value::String(sv) => {
                            serde_yaml::Value::String(sv.to_str().unwrap_or("").to_string())
                        }
                        mlua::Value::Table(tt) => {
                            let mut seq = Vec::new();
                            for item in tt.sequence_values::<mlua::Value>() {
                                if let Ok(mlua::Value::String(ss)) = item {
                                    seq.push(serde_yaml::Value::String(
                                        ss.to_str().unwrap_or("").to_string(),
                                    ));
                                }
                            }
                            serde_yaml::Value::Sequence(seq)
                        }
                        _ => serde_yaml::Value::Null,
                    };
                    map.insert(key, yaml_v);
                }
            }
            return Some(map);
        }
    }
    None
}

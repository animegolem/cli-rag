use anyhow::{anyhow, Context, Result};
use globset::{Glob, GlobSetBuilder};
use globwalk::GlobWalkerBuilder;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::defaults::*;
use super::schema::{Config, SchemaCfg};
use crate::config::lua::discover_overlays;

pub fn find_config_upwards(explicit: &Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = explicit {
        return Some(p.clone());
    }
    if let Ok(env_path) = env::var("CLI_RAG_CONFIG") {
        let p = PathBuf::from(env_path);
        if p.exists() {
            return Some(p);
        }
    }
    let mut dir = env::current_dir().ok()?;
    loop {
        let candidate = dir.join(".cli-rag.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        let parent = dir.parent();
        match parent {
            Some(p) => dir = p.to_path_buf(),
            None => return None,
        }
    }
}

fn find_all_configs_upwards_chain() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(mut dir) = env::current_dir() {
        loop {
            let candidate = dir.join(".cli-rag.toml");
            if candidate.exists() {
                out.push(candidate);
            }
            match dir.parent() {
                Some(p) => dir = p.to_path_buf(),
                None => break,
            }
        }
    }
    out
}

fn normalize_nested_user_config(mut tv: toml::Value) -> toml::Value {
    use toml::Value as V;
    let root = tv.as_table_mut();
    if root.is_none() {
        return tv;
    }
    let root = root.unwrap();
    // Extract and flatten [config] nested shape into flat keys our Config understands.
    if let Some(V::Table(cfg_tbl)) = root.remove("config") {
        // config.config_version -> config_version (nested takes precedence over flat)
        if let Some(V::String(v)) = cfg_tbl.get("config_version") {
            root.insert("config_version".into(), V::String(v.clone()));
        }
        // [config.scan]
        if let Some(V::Table(scan)) = cfg_tbl.get("scan") {
            // filepaths -> filepaths (alias to bases)
            if let Some(v) = scan.get("filepaths") {
                // Write to alias key understood by Config (serde alias will map to bases)
                root.insert("filepaths".into(), v.clone());
            }
            // index_path -> index_relative
            if let Some(V::String(v)) = scan.get("index_path") {
                root.insert("index_relative".into(), V::String(v.clone()));
            }
            // ignore_globs -> ignore_globs
            if let Some(v) = scan.get("ignore_globs") {
                root.insert("ignore_globs".into(), v.clone());
            }
        }
        // [config.graph]
        if let Some(V::Table(graph)) = cfg_tbl.get("graph") {
            // Map to [defaults] table fields
            let defaults_entry = root
                .entry("defaults")
                .or_insert(V::Table(Default::default()));
            if let V::Table(def_tbl) = defaults_entry {
                if let Some(V::Integer(depth)) = graph.get("depth") {
                    def_tbl.insert("depth".into(), V::Integer(*depth));
                }
                if let Some(V::Boolean(b)) = graph.get("include_bidirectional") {
                    def_tbl.insert("include_bidirectional".into(), V::Boolean(*b));
                }
            }
        }
        // [config.templates]
        if let Some(V::Table(templates)) = cfg_tbl.get("templates") {
            if let Some(v) = templates.get("import") {
                root.insert("import".into(), v.clone());
            }
        }
        // We've consumed the nested table; not re-inserting keeps the normalized shape.
    }
    tv
}

pub fn load_config(
    path_opt: &Option<PathBuf>,
    base_override: &Option<Vec<PathBuf>>,
    no_lua: bool,
) -> Result<(Config, Option<PathBuf>)> {
    // Detect multiple configs in scope (unless an explicit path is provided or CLI_RAG_CONFIG is set)
    let env_cfg = env::var("CLI_RAG_CONFIG").ok().map(PathBuf::from);
    let path = find_config_upwards(path_opt);
    if path_opt.is_none() && env_cfg.is_none() {
        let chain = find_all_configs_upwards_chain();
        if chain.len() > 1 {
            let list = chain
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(anyhow!(
                "E100: Multiple project configs detected. Only one .cli-rag.toml is allowed. Found: {}",
                list
            ));
        }
    }
    let mut cfg: Config = if let Some(ref p) = path {
        let s = fs::read_to_string(p).with_context(|| format!("reading config {:?}", p))?;
        // Parse as TOML value first to support nested [config.*] shape, then normalize
        let tv: toml::Value =
            toml::from_str(&s).with_context(|| format!("parsing TOML config {:?}", p))?;
        let flat = normalize_nested_user_config(tv);
        let flat_str = toml::to_string(&flat).unwrap_or_else(|_| s.clone());
        toml::from_str(&flat_str).with_context(|| format!("mapping normalized config {:?}", p))?
    } else {
        Config {
            config_version: None,
            import: Vec::new(),
            bases: default_bases(),
            index_relative: default_index_rel(),
            groups_relative: default_groups_rel(),
            file_patterns: default_file_patterns(),
            ignore_globs: default_ignore_globs(),
            allowed_statuses: default_allowed_statuses(),
            defaults: default_defaults(),
            schema: Vec::new(),
            overlays: super::schema::OverlayInfo::default(),
        }
    };
    // Default config_version if not provided
    if cfg.config_version.is_none() {
        cfg.config_version = Some(super::defaults::default_config_version());
    }
    // Discover overlays (repo + user), honoring CLI flag and env. Merge is deferred to hook wiring.
    let overlays = discover_overlays(&path, no_lua);
    cfg.overlays = overlays;
    // Env override for bases/filepaths (comma-separated)
    if let Ok(env_bases) = env::var("CLI_RAG_FILEPATHS") {
        let list: Vec<PathBuf> = env_bases
            .split(',')
            .map(|s| PathBuf::from(s.trim()))
            .filter(|p| !p.as_os_str().is_empty())
            .collect();
        if !list.is_empty() {
            cfg.bases = list;
        }
    }
    if let Some(override_bases) = base_override {
        if !override_bases.is_empty() {
            cfg.bases = override_bases.clone();
        }
    }
    // Import external schema files (schemas only)
    if let Some(ref cfg_path) = path {
        if !cfg.import.is_empty() {
            let cfg_dir = cfg_path.parent().unwrap_or(Path::new("."));
            let mut imported: Vec<SchemaCfg> = Vec::new();
            // Track schema name -> sources for better E120 reporting across imports
            use std::collections::BTreeMap as _BTreeMap;
            let mut name_sources: _BTreeMap<String, Vec<String>> = _BTreeMap::new();
            for sc in &cfg.schema {
                name_sources
                    .entry(sc.name.clone())
                    .or_default()
                    .push(cfg_path.display().to_string());
            }
            for patt in &cfg.import {
                let patt_path = cfg_dir.join(patt);
                let mut files: Vec<PathBuf> = Vec::new();
                // Expand globs if any; if no matches, try as a direct file path.
                let walk_res = GlobWalkerBuilder::from_patterns(cfg_dir, &[patt.as_str()])
                    .max_depth(10)
                    .follow_links(true)
                    .build();
                if let Ok(walker) = walk_res {
                    for entry in walker.filter_map(Result::ok) {
                        if entry.path().is_file() {
                            files.push(entry.path().to_path_buf());
                        }
                    }
                }
                if files.is_empty() {
                    if patt_path.exists() && patt_path.is_file() {
                        files.push(patt_path);
                    } else {
                        let abs = PathBuf::from(patt);
                        if abs.is_absolute() && abs.exists() && abs.is_file() {
                            files.push(abs);
                        }
                    }
                }
                for fpath in files {
                    let s = fs::read_to_string(&fpath)
                        .with_context(|| format!("reading import {:?}", fpath))?;
                    let tv: toml::Value = toml::from_str(&s)
                        .with_context(|| format!("parsing import {:?}", fpath))?;
                    // Validate only [[schema]] allowed at top-level
                    let illegal_keys: Vec<String> = tv
                        .as_table()
                        .map(|t| {
                            t.keys()
                                .filter(|k| k.as_str() != "schema")
                                .cloned()
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if !illegal_keys.is_empty() {
                        return Err(anyhow!(
                            "E110: Illegal top-level key(s) [{}] in import {}. Imports may define schemas only.",
                            illegal_keys.join(", "),
                            fpath.display()
                        ));
                    }
                    #[derive(Deserialize)]
                    struct ImportSchemas {
                        #[serde(default)]
                        schema: Vec<SchemaCfg>,
                    }
                    let imp: ImportSchemas = toml::from_str(&s)
                        .with_context(|| format!("parsing schemas in import {:?}", fpath))?;
                    // Detect duplicate names against prior and current imports with source paths
                    for sc in imp.schema {
                        if let Some(srcs) = name_sources.get(&sc.name).cloned() {
                            let mut all = srcs;
                            all.push(fpath.display().to_string());
                            return Err(anyhow!(
                                "E120: Duplicate schema name(s) detected: {}\nConflicting schema sources:\n  - {}",
                                sc.name,
                                all.join("\n  - ")
                            ));
                        }
                        name_sources
                            .entry(sc.name.clone())
                            .or_default()
                            .push(fpath.display().to_string());
                        imported.push(sc);
                    }
                }
            }
            cfg.schema.extend(imported);
        }
    }
    // Invariant: unique schema names across the effective config
    if !cfg.schema.is_empty() {
        use std::collections::BTreeMap;
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        for sc in &cfg.schema {
            *seen.entry(sc.name.clone()).or_insert(0) += 1;
        }
        let dups: Vec<String> = seen
            .into_iter()
            .filter_map(|(k, v)| if v > 1 { Some(k) } else { None })
            .collect();
        if !dups.is_empty() {
            return Err(anyhow!(
                "E120: Duplicate schema name(s) detected: {}",
                dups.join(", ")
            ));
        }
    }
    Ok((cfg, path))
}

// Helper: compile schema globsets once for reuse across modules.
pub fn build_schema_sets(cfg: &Config) -> Vec<(SchemaCfg, globset::GlobSet)> {
    let mut out = Vec::new();
    for sc in &cfg.schema {
        let mut b = GlobSetBuilder::new();
        for p in &sc.file_patterns {
            if let Ok(g) = Glob::new(p) {
                b.add(g);
            }
        }
        if let Ok(set) = b.build() {
            out.push((sc.clone(), set));
        }
    }
    out
}

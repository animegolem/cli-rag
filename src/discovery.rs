use anyhow::{Context, Result};
use globwalk::GlobWalkerBuilder;
use globset::{Glob, GlobSetBuilder};
use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::model::{AdrDoc, parse_front_matter_and_title, file_mtime, file_size};

pub fn scan_docs(cfg: &Config) -> Result<Vec<AdrDoc>> {
    let mut docs = Vec::new();
    let mut ig_builder = GlobSetBuilder::new();
    for pat in &cfg.ignore_globs { ig_builder.add(Glob::new(pat)?); }
    let ignore_set = ig_builder.build()?;
    for base in &cfg.bases {
        for pattern in &cfg.file_patterns {
            let builder = GlobWalkerBuilder::from_patterns(base, &[pattern.as_str()]);
            let walker = builder.build()?;
            for entry in walker.filter_map(Result::ok) {
                let path = entry.path().to_path_buf();
                if path.is_file() {
                    if ignore_set.is_match(&path) { continue; }
                    let content = fs::read_to_string(&path).with_context(|| format!("reading {:?}", path))?;
                    let doc = parse_front_matter_and_title(&content, &path);
                    docs.push(doc);
                }
            }
        }
    }
    Ok(docs)
}

pub fn scan_docs_in_base(base: &Path, cfg: &Config) -> Result<Vec<AdrDoc>> {
    let mut docs = Vec::new();
    let mut ig_builder = GlobSetBuilder::new();
    for pat in &cfg.ignore_globs { ig_builder.add(Glob::new(pat)?); }
    let ignore_set = ig_builder.build()?;
    for pattern in &cfg.file_patterns {
        let builder = GlobWalkerBuilder::from_patterns(base, &[pattern.as_str()]);
        let walker = builder.build()?;
        for entry in walker.filter_map(Result::ok) {
            let path = entry.path().to_path_buf();
            if path.is_file() {
                if ignore_set.is_match(&path) { continue; }
                let content = fs::read_to_string(&path).with_context(|| format!("reading {:?}", path))?;
                let doc = parse_front_matter_and_title(&content, &path);
                docs.push(doc);
            }
        }
    }
    Ok(docs)
}

pub fn load_docs_from_index(base: &Path, cfg: &Config) -> Result<Vec<AdrDoc>> {
    let index_path = base.join(&cfg.index_relative);
    let data = fs::read_to_string(&index_path).with_context(|| format!("reading index {:?}", index_path))?;
    let root: Value = serde_json::from_str(&data).with_context(|| format!("parsing index {:?}", index_path))?;
    let mut docs = Vec::new();
    let items_opt: Option<&Vec<Value>> = if let Some(items) = root.as_array() {
        Some(items)
    } else if let Some(items) = root.get("items").and_then(|v| v.as_array()) {
        Some(items)
    } else { None };
    if let Some(items) = items_opt {
        for item in items {
            let file_rel = item.get("file").and_then(|v| v.as_str()).unwrap_or("");
            let file = base.join(file_rel);
            let id = item.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let tags = item.get("tags").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_else(|| Vec::new());
            let status = item.get("status").and_then(|v| v.as_str()).map(|s| s.to_string());
            let groups = item.get("groups").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_else(|| Vec::new());
            let depends_on = item.get("depends_on").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect()).unwrap_or_else(|| Vec::new());
            let supersedes = match item.get("supersedes") {
                Some(Value::Array(a)) => a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect(),
                Some(Value::String(s)) => vec![s.to_string()],
                _ => Vec::new(),
            };
            let superseded_by = match item.get("superseded_by") {
                Some(Value::Array(a)) => a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect(),
                Some(Value::String(s)) => vec![s.to_string()],
                _ => Vec::new(),
            };
            let mtime = item.get("mtime").and_then(|v| v.as_u64());
            let size = item.get("size").and_then(|v| v.as_u64());
            docs.push(AdrDoc { file, id, title, tags, status, groups, depends_on, supersedes, superseded_by, fm: std::collections::BTreeMap::new(), mtime, size });
        }
    }
    Ok(docs)
}

pub fn load_docs(cfg: &Config) -> Result<Vec<AdrDoc>> {
    let mut combined: Vec<AdrDoc> = Vec::new();
    for base in &cfg.bases {
        let index_path = base.join(&cfg.index_relative);
        let use_index = index_path.exists();
        let mut docs = if use_index { load_docs_from_index(base, cfg)? } else { scan_docs_in_base(base, cfg)? };
        combined.append(&mut docs);
    }
    Ok(dedupe_by_id(combined))
}

pub fn incremental_collect_docs(cfg: &Config, full_rescan: bool) -> Result<Vec<AdrDoc>> {
    let mut ig_builder = GlobSetBuilder::new();
    for pat in &cfg.ignore_globs { ig_builder.add(Glob::new(pat)?); }
    let ignore_set = ig_builder.build()?;
    let mut combined: Vec<AdrDoc> = Vec::new();
    for base in &cfg.bases {
        let mut prior = std::collections::HashMap::<String, AdrDoc>::new();
        let index_path = base.join(&cfg.index_relative);
        if index_path.exists() {
            let idx_docs = load_docs_from_index(base, cfg)?;
            for d in idx_docs {
                if let Ok(rel) = d.file.strip_prefix(base) { prior.insert(rel.to_string_lossy().to_string(), d); }
            }
        }
        let mut seen_rel: std::collections::HashSet<String> = std::collections::HashSet::new();
        for pattern in &cfg.file_patterns {
            let walker = GlobWalkerBuilder::from_patterns(base, &[pattern.as_str()]).build()?;
            for entry in walker.filter_map(Result::ok) {
                let path = entry.path().to_path_buf();
                if !path.is_file() { continue; }
                if ignore_set.is_match(&path) { continue; }
                let rel = path.strip_prefix(base).unwrap_or(&path).to_string_lossy().to_string();
                seen_rel.insert(rel.clone());
                let cur_mtime = file_mtime(&path).unwrap_or(0);
                let cur_size = file_size(&path).unwrap_or(0);
                let need_parse = full_rescan || match prior.get(&rel) {
                    None => true,
                    Some(old) => old.mtime.unwrap_or(0) != cur_mtime || old.size.unwrap_or(0) != cur_size,
                };
                if need_parse {
                    let content = fs::read_to_string(&path).with_context(|| format!("reading {:?}", path))?;
                    let mut doc = parse_front_matter_and_title(&content, &path);
                    doc.mtime = Some(cur_mtime);
                    doc.size = Some(cur_size);
                    combined.push(doc);
                } else {
                    if let Some(mut d_old) = prior.get(&rel).cloned() {
                        d_old.mtime = Some(cur_mtime);
                        d_old.size = Some(cur_size);
                        combined.push(d_old);
                    }
                }
            }
        }
        // Removed files implicitly dropped
    }
    Ok(dedupe_by_id(combined))
}

fn dedupe_by_id(mut docs: Vec<AdrDoc>) -> Vec<AdrDoc> {
    use std::collections::HashMap;
    let mut by_id: HashMap<String, AdrDoc> = HashMap::new();
    let mut no_id: Vec<AdrDoc> = Vec::new();
    for d in docs.drain(..) {
        if let Some(id) = &d.id {
            let replace = match by_id.get(id) {
                Some(existing) => {
                    let a = existing.mtime;
                    let b = d.mtime;
                    match (a, b) { (Some(a), Some(b)) => b > a, (None, Some(_)) => true, _ => false }
                }
                None => true,
            };
            if replace { by_id.insert(id.clone(), d); }
        } else {
            no_id.push(d);
        }
    }
    let mut out: Vec<AdrDoc> = by_id.into_values().collect();
    out.extend(no_id);
    out
}

use anyhow::{Context, Result};
use globset::{Glob, GlobSetBuilder};
use globwalk::GlobWalkerBuilder;
use std::fs;
use std::path::Path;

use crate::config::Config;
use crate::model::{parse_front_matter_and_title, AdrDoc};

pub fn scan_docs(cfg: &Config) -> Result<Vec<AdrDoc>> {
    let mut docs = Vec::new();
    let mut ig_builder = GlobSetBuilder::new();
    for pat in &cfg.ignore_globs {
        ig_builder.add(Glob::new(pat)?);
    }
    let ignore_set = ig_builder.build()?;
    for base in &cfg.bases {
        for pattern in &cfg.file_patterns {
            let builder = GlobWalkerBuilder::from_patterns(base, &[pattern.as_str()]);
            let walker = builder.build()?;
            for entry in walker.filter_map(Result::ok) {
                let path = entry.path().to_path_buf();
                if path.is_file() {
                    if ignore_set.is_match(&path) {
                        continue;
                    }
                    let content =
                        fs::read_to_string(&path).with_context(|| format!("reading {:?}", path))?;
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
    for pat in &cfg.ignore_globs {
        ig_builder.add(Glob::new(pat)?);
    }
    let ignore_set = ig_builder.build()?;
    for pattern in &cfg.file_patterns {
        let builder = GlobWalkerBuilder::from_patterns(base, &[pattern.as_str()]);
        let walker = builder.build()?;
        for entry in walker.filter_map(Result::ok) {
            let path = entry.path().to_path_buf();
            if path.is_file() {
                if ignore_set.is_match(&path) {
                    continue;
                }
                let content =
                    fs::read_to_string(&path).with_context(|| format!("reading {:?}", path))?;
                let doc = parse_front_matter_and_title(&content, &path);
                docs.push(doc);
            }
        }
    }
    Ok(docs)
}

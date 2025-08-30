use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

use crate::cli::OutputFormat;
use crate::commands::output::print_json;
use crate::config::{Config, SchemaCfg};
use crate::discovery::load_docs;

pub(crate) fn build_report(
    cfg: &Config,
    cfg_path: &Option<PathBuf>,
    docs: &Vec<crate::model::AdrDoc>,
) -> serde_json::Value {
    let mut id_to_docs: HashMap<String, Vec<&crate::model::AdrDoc>> = HashMap::new();
    for d in docs {
        if let Some(ref id) = d.id {
            id_to_docs.entry(id.clone()).or_default().push(d);
        }
    }
    let mut conflicts = Vec::new();
    for (id, lst) in &id_to_docs {
        if lst.len() > 1 {
            let mut titles: BTreeSet<String> = BTreeSet::new();
            let mut statuses: BTreeSet<String> = BTreeSet::new();
            for d in lst.iter() {
                let doc = *d;
                titles.insert(doc.title.clone());
                if let Some(ref s) = doc.status {
                    statuses.insert(s.clone());
                }
            }
            if titles.len() > 1 || statuses.len() > 1 {
                conflicts.push(id.clone());
            }
        }
    }
    let group_count: usize = docs.iter().flat_map(|d| d.groups.iter()).count();
    // Per-type counts and unknown-key stats
    let mut schema_sets: Vec<(SchemaCfg, globset::GlobSet)> = Vec::new();
    for sc in &cfg.schema {
        let mut b = globset::GlobSetBuilder::new();
        for p in &sc.file_patterns {
            if let Ok(g) = globset::Glob::new(p) {
                b.add(g);
            }
        }
        if let Ok(set) = b.build() {
            schema_sets.push((sc.clone(), set));
        }
    }
    let mut type_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut unknown_stats: BTreeMap<String, (usize, usize)> = BTreeMap::new(); // schema -> (docs_with_unknowns, total_unknown_keys)
    let reserved: BTreeSet<String> = [
        "id",
        "tags",
        "status",
        "groups",
        "depends_on",
        "supersedes",
        "superseded_by",
    ]
    .into_iter()
    .map(|s| s.to_string())
    .collect();
    for d in docs {
        let fname = d.file.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let mut sname: Option<String> = None;
        for (sc, set) in &schema_sets {
            if set.is_match(fname) {
                sname = Some(sc.name.clone());
                break;
            }
        }
        if let Some(sname) = sname {
            *type_counts.entry(sname.clone()).or_insert(0) += 1;
            let present: BTreeSet<String> = d.fm.keys().cloned().collect();
            let rule_keys: BTreeSet<String> = cfg
                .schema
                .iter()
                .find(|s| s.name == sname)
                .map(|sc| sc.rules.keys().cloned().collect())
                .unwrap_or_default();
            let mut known: BTreeSet<String> = reserved.union(&rule_keys).cloned().collect();
            let req: BTreeSet<String> = cfg
                .schema
                .iter()
                .find(|s| s.name == sname)
                .map(|sc| sc.required.iter().cloned().collect())
                .unwrap_or_default();
            known = known.union(&req).cloned().collect();
            let allow: BTreeSet<String> = cfg
                .schema
                .iter()
                .find(|s| s.name == sname)
                .map(|sc| sc.allowed_keys.iter().cloned().collect())
                .unwrap_or_default();
            known = known.union(&allow).cloned().collect();
            let unknown: Vec<String> = present.difference(&known).cloned().collect();
            if !unknown.is_empty() {
                let e = unknown_stats.entry(sname).or_insert((0, 0));
                e.0 += 1;
                e.1 += unknown.len();
            }
        }
    }

    let per_base: Vec<serde_json::Value> = cfg
        .bases
        .iter()
        .map(|b| {
            let idx = b.join(&cfg.index_relative);
            let mode = if idx.exists() { "index" } else { "scan" };
            serde_json::json!({
                "base": b,
                "mode": mode
            })
        })
        .collect();

    // Invariants summary (post-load)
    let mut invariants_ok = true;
    let mut invariants_errors: Vec<String> = Vec::new();
    // Duplicate schema names (should be enforced at load time, but re-check for visibility)
    use std::collections::BTreeMap as _BTreeMap;
    let mut seen: _BTreeMap<String, usize> = _BTreeMap::new();
    for sc in &cfg.schema {
        *seen.entry(sc.name.clone()).or_insert(0) += 1;
    }
    for (name, count) in seen {
        if count > 1 {
            invariants_ok = false;
            invariants_errors.push(format!("E120 duplicate schema name: {} ({}x)", name, count));
        }
    }

    serde_json::json!({
        "config": cfg_path.as_ref().map(|p| p.display().to_string()).unwrap_or("<defaults>".into()),
        "bases": cfg.bases,
        "per_base": per_base,
        "counts": {"docs": docs.len(), "group_entries": group_count},
        "conflicts": conflicts,
        "types": type_counts,
        "unknown_stats": unknown_stats,
        "invariants_ok": invariants_ok,
        "invariants_errors": invariants_errors,
    })
}

pub fn run(cfg: &Config, cfg_path: &Option<PathBuf>, format: &OutputFormat) -> Result<()> {
    let docs = load_docs(cfg)?;
    match format {
        OutputFormat::Json | OutputFormat::Ndjson => {
            let report = build_report(cfg, cfg_path, &docs);
            print_json(&report)?;
        }
        OutputFormat::Plain => {
            // Plain text output
            let report = build_report(cfg, cfg_path, &docs);
            let config_path = report.get("config").and_then(|v| v.as_str()).unwrap_or("");
            println!("Config: {}", config_path);
            println!("Bases:");
            for b in &cfg.bases {
                println!("  - {}", b.display());
            }
            println!("index_relative: {}", cfg.index_relative);
            println!("groups_relative: {}", cfg.groups_relative);
            for item in report
                .get("per_base")
                .and_then(|v| v.as_array())
                .unwrap_or(&Vec::new())
            {
                let base = item.get("base").and_then(|v| v.as_str()).unwrap_or("");
                let mode = item.get("mode").and_then(|v| v.as_str()).unwrap_or("");
                println!("Base {} → {}", base, mode);
            }
            let counts = report
                .get("counts")
                .and_then(|v| v.as_object())
                .cloned()
                .unwrap_or_default();
            let docs_count = counts.get("docs").and_then(|v| v.as_u64()).unwrap_or(0);
            let group_entries = counts
                .get("group_entries")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            println!(
                "Found {} ADR-like files; group entries: {}",
                docs_count, group_entries
            );
            if let Some(arr) = report.get("conflicts").and_then(|v| v.as_array()) {
                if !arr.is_empty() {
                    let list: Vec<String> = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    println!(
                        "Conflicts (ids with differing title/status): {}",
                        list.join(", ")
                    );
                }
            }
            if let Some(types) = report.get("types").and_then(|v| v.as_object()) {
                if !types.is_empty() {
                    println!("Types:");
                    for (k, v) in types {
                        println!("  - {}: {} notes", k, v.as_u64().unwrap_or(0));
                    }
                }
            }
            if let Some(unknown) = report.get("unknown_stats").and_then(|v| v.as_object()) {
                if !unknown.is_empty() {
                    println!("Unknown key stats:");
                    for (k, v) in unknown {
                        if let Some(arr) = v.as_array() {
                            if arr.len() == 2 {
                                let docs = arr[0].as_u64().unwrap_or(0);
                                let total = arr[1].as_u64().unwrap_or(0);
                                println!(
                                    "  - {}: {} notes with unknowns ({} keys)",
                                    k, docs, total
                                );
                            }
                        }
                    }
                }
            }
            let invariants_ok = report.get("invariants_ok").and_then(|v| v.as_bool()).unwrap_or(true);
            if invariants_ok {
                println!("Invariants: OK");
            } else if let Some(errs) = report.get("invariants_errors").and_then(|v| v.as_array()) {
                println!("Invariants: FAILED");
                for e in errs {
                    if let Some(s) = e.as_str() {
                        println!("  - {}", s);
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        default_allowed_statuses, default_defaults, default_file_patterns, default_groups_rel,
        default_ignore_globs, default_index_rel, Config, SchemaCfg,
    };
    use std::collections::BTreeMap;

    #[test]
    fn test_build_report_invariants_duplicates() {
        let cfg = Config {
            import: Vec::new(),
            bases: vec![],
            index_relative: default_index_rel(),
            groups_relative: default_groups_rel(),
            file_patterns: default_file_patterns(),
            ignore_globs: default_ignore_globs(),
            allowed_statuses: default_allowed_statuses(),
            defaults: default_defaults(),
            schema: vec![
                SchemaCfg {
                    name: "ADR".into(),
                    file_patterns: vec!["ADR-*.md".into()],
                    required: vec!["id".into()],
                    unknown_policy: Some("ignore".into()),
                    allowed_keys: vec![],
                    rules: BTreeMap::new(),
                },
                SchemaCfg {
                    name: "ADR".into(),
                    file_patterns: vec!["ADR-DB-*.md".into()],
                    required: vec!["id".into()],
                    unknown_policy: Some("ignore".into()),
                    allowed_keys: vec![],
                    rules: BTreeMap::new(),
                },
            ],
        };
        let docs = Vec::new();
        let report = build_report(&cfg, &None, &docs);
        let ok = report
            .get("invariants_ok")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        assert!(!ok, "expected invariants_ok=false with duplicate schema names");
        let errs = report
            .get("invariants_errors")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        assert!(
            errs.iter()
                .filter_map(|v| v.as_str())
                .any(|s| s.contains("E120")),
            "expected E120 in invariants_errors"
        );
    }
}

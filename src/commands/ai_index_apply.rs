use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::PathBuf;

use crate::config::Config;
use crate::discovery::unified::load_docs_unified;

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    let mut s = String::with_capacity(out.len() * 2);
    for b in out {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

pub fn run(
    cfg: &Config,
    cfg_path: &Option<PathBuf>,
    plan_path: PathBuf,
    write_cache: bool,
    write_frontmatter: bool,
    dry_run: bool,
) -> Result<()> {
    let cfg_dir = cfg_path.as_ref().and_then(|p| p.parent()).ok_or_else(|| {
        anyhow!("Cannot locate config directory; pass --config and run `validate` first")
    })?;

    // Load plan
    let plan_data = std::fs::read(&plan_path)
        .with_context(|| format!("reading plan {}", plan_path.display()))?;
    let plan: Value = serde_json::from_slice(&plan_data)
        .with_context(|| format!("parsing plan {}", plan_path.display()))?;

    // Check sourceIndexHash match by hashing current unified index
    let unified_path = cfg_dir.join(&cfg.index_relative);
    if !unified_path.exists() {
        return Err(anyhow!(
            "Unified index not found at {}. Run `cli-rag validate` first.",
            unified_path.display()
        ));
    }
    let idx_bytes = std::fs::read(&unified_path)
        .with_context(|| format!("reading unified index {}", unified_path.display()))?;
    let idx_hash = format!("sha256:{}", sha256_hex(&idx_bytes));
    let plan_hash = plan
        .get("sourceIndexHash")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if !plan_hash.is_empty() && plan_hash != idx_hash {
        eprintln!(
            "Plan sourceIndexHash does not match current index (plan: {}, current: {})",
            plan_hash, idx_hash
        );
        // Exit code 2 per conventions (validation/contract failure)
        std::process::exit(2);
    }

    // Build cache JSON
    let mut cache_clusters: Vec<Value> = Vec::new();
    let mut apply_members: BTreeSet<String> = BTreeSet::new();
    if let Some(arr) = plan.get("clusters").and_then(|v| v.as_array()) {
        for c in arr {
            let cluster_id = c.get("clusterId").and_then(|v| v.as_str()).unwrap_or("");
            let label = c.get("label").and_then(|v| v.as_str()).unwrap_or("");
            let summary = c.get("summary").and_then(|v| v.as_str()).unwrap_or("");
            let tags = c
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_else(Vec::new);
            let members = c
                .get("members")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect::<Vec<String>>()
                })
                .unwrap_or_else(Vec::new);
            for m in &members {
                apply_members.insert(m.clone());
            }
            let mut obj = serde_json::Map::new();
            obj.insert("clusterId".into(), Value::String(cluster_id.to_string()));
            obj.insert("label".into(), Value::String(label.to_string()));
            obj.insert("summary".into(), Value::String(summary.to_string()));
            obj.insert(
                "members".into(),
                Value::Array(members.into_iter().map(Value::String).collect()),
            );
            obj.insert(
                "tags".into(),
                Value::Array(tags.into_iter().map(Value::String).collect()),
            );
            cache_clusters.push(Value::Object(obj));
        }
    }
    let cache = serde_json::json!({
        "version": 1,
        "clusters": cache_clusters,
    });

    // Write cache if requested
    let cache_path = cfg_dir.join(".cli-rag/cache/ai-index.json");
    if write_cache && !dry_run {
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(&cache_path, serde_json::to_string_pretty(&cache)?)
            .with_context(|| format!("writing cache {}", cache_path.display()))?;
    }

    // Frontmatter writes (optional, additive)
    let mut members_tagged = 0usize;
    let mut warnings: Vec<String> = Vec::new();
    if write_frontmatter && !apply_members.is_empty() {
        // Build ID -> path map via unified docs
        let docs = load_docs_unified(cfg, &Some(cfg_dir.join(".cli-rag.toml"))).unwrap_or(None);
        let mut id_to_path: HashMap<String, PathBuf> = HashMap::new();
        let mut id_to_fm_tags_present: HashMap<String, bool> = HashMap::new();
        if let Some(docs) = docs {
            for d in docs {
                if let Some(id) = d.id {
                    id_to_path.insert(id.clone(), d.file.clone());
                    // Heuristic: mark whether tags field exists in FM from unified index frontmatter
                    let has_tags = d.fm.contains_key("tags");
                    id_to_fm_tags_present.insert(id, has_tags);
                }
            }
        }
        // Build cluster tag map from plan
        let mut cluster_tags: BTreeMap<String, Vec<String>> = BTreeMap::new();
        if let Some(arr) = plan.get("clusters").and_then(|v| v.as_array()) {
            for c in arr {
                let members = c
                    .get("members")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let tags: Vec<String> = c
                    .get("tags")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_else(|| {
                        // Derive from label if tags missing
                        let label = c.get("label").and_then(|v| v.as_str()).unwrap_or("");
                        if label.is_empty() {
                            Vec::new()
                        } else {
                            vec![label.to_string().to_kebab_case()]
                        }
                    });
                for m in members {
                    if let Some(mid) = m.as_str() {
                        cluster_tags.insert(mid.to_string(), tags.clone());
                    }
                }
            }
        }

        for id in apply_members {
            let Some(path) = id_to_path.get(&id) else {
                warnings.push(format!(
                    "ID {} not found in unified index; skipping frontmatter write",
                    id
                ));
                continue;
            };
            // Require tags to be present in FM to comply with ADR (we don't know schema here)
            if !id_to_fm_tags_present.get(&id).cloned().unwrap_or(false) {
                eprintln!(
                    "Schema for {} may not define tags; refusing to write tags.",
                    id
                );
                std::process::exit(4);
            }
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("reading {}", path.display()))?;
            // Extract frontmatter block
            if !content.starts_with("---\n") {
                eprintln!(
                    "{} missing YAML frontmatter; refusing to tag.",
                    path.display()
                );
                std::process::exit(4);
            }
            let Some(end) = content.find("\n---\n") else {
                eprintln!(
                    "{} malformed YAML frontmatter; refusing to tag.",
                    path.display()
                );
                std::process::exit(4);
            };
            let fm_str = &content[4..end];
            let rest = content[end + 5..].to_string();
            let mut fm_val: serde_yaml::Value =
                serde_yaml::from_str(fm_str).unwrap_or(serde_yaml::Value::Null);
            // Ensure mapping
            if let serde_yaml::Value::Mapping(ref mut map) = fm_val {
                use serde_yaml::Value as YV;
                let tags_key = YV::String("tags".into());
                let mut current: Vec<String> = Vec::new();
                if let Some(existing) = map.get(&tags_key) {
                    if let YV::Sequence(seq) = existing {
                        for it in seq {
                            if let YV::String(s) = it {
                                current.push(s.to_string());
                            }
                        }
                    } else {
                        eprintln!(
                            "{}: tags is not a sequence; refusing to tag.",
                            path.display()
                        );
                        std::process::exit(4);
                    }
                } else {
                    // Missing tags; per ADR, treat as schema not supporting tags
                    eprintln!("{}: no tags field; refusing to tag.", path.display());
                    std::process::exit(4);
                }
                // Merge (additive, de-dup)
                let add = cluster_tags.get(&id).cloned().unwrap_or_default();
                let mut set: BTreeSet<String> = current.into_iter().collect();
                for t in add {
                    set.insert(t);
                }
                let new_seq = YV::Sequence(set.into_iter().map(YV::String).collect());
                map.insert(tags_key, new_seq);
                // Write back
                if !dry_run {
                    let fm_new = serde_yaml::to_string(&fm_val).unwrap_or_default();
                    let new_content = format!("---\n{}---\n{}", fm_new, rest);
                    std::fs::write(path, new_content)
                        .with_context(|| format!("writing {}", path.display()))?;
                }
                members_tagged += 1;
            } else {
                eprintln!(
                    "{}: frontmatter is not a mapping; refusing to tag.",
                    path.display()
                );
                std::process::exit(4);
            }
        }
    }

    // Emit apply report JSON
    let report = serde_json::json!({
        "ok": true,
        "written": { "cache": write_cache, "frontmatter": write_frontmatter },
        "clustersApplied": cache["clusters"].as_array().map(|a| a.len()).unwrap_or(0),
        "membersTagged": members_tagged,
        "warnings": warnings,
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

// For kebab-case derivation
use heck::ToKebabCase;

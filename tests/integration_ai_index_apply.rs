use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::str::contains;
use std::process::Command;

#[test]
fn ai_index_apply_writes_cache_and_reports() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    // Minimal config
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // Create a small connected trio with tags field in FM to allow tagging later
    base.child("ADR-001.md").write_str(
        "---\nid: ADR-001\ntags: []\nstatus: draft\ndepends_on: [ADR-002]\n---\n\n# ADR-001\n\n",
    ).unwrap();
    base.child("ADR-002.md").write_str(
        "---\nid: ADR-002\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# ADR-002\nSee [[ADR-003]].\n",
    ).unwrap();
    base.child("ADR-003.md")
        .write_str(
            "---\nid: ADR-003\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# ADR-003\n\n",
        )
        .unwrap();

    // Validate to produce unified index
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // Plan
    let plan_path = temp.child("plan.json");
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("ai")
        .arg("index")
        .arg("plan")
        .arg("--min-cluster-size")
        .arg("2")
        .arg("--output")
        .arg(plan_path.path())
        .assert()
        .success();

    // Apply (dry-run, no writes to files)
    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("ai")
        .arg("index")
        .arg("apply")
        .arg("--from")
        .arg(plan_path.path())
        .arg("--dry-run")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(v["ok"].as_bool().unwrap());
    assert!(v["written"]["cache"].as_bool().unwrap());
    assert!(!v["written"]["frontmatter"].as_bool().unwrap());

    temp.close().unwrap();
}

#[test]
fn ai_index_apply_writes_frontmatter_tags_and_cache() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    // Config
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // Notes with tags field
    base.child("ADR-010.md").write_str(
        "---\nid: ADR-010\ntags: []\nstatus: draft\ndepends_on: [ADR-011]\n---\n\n# ADR-010\n\n",
    ).unwrap();
    base.child("ADR-011.md")
        .write_str(
            "---\nid: ADR-011\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# ADR-011\n\n",
        )
        .unwrap();

    // Build index
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // Plan
    let plan_path = temp.child("plan.json");
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("ai")
        .arg("index")
        .arg("plan")
        .arg("--min-cluster-size")
        .arg("2")
        .arg("--output")
        .arg(plan_path.path())
        .assert()
        .success();

    // Inject a label (no tags) so apply derives tag from label
    let mut plan_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(plan_path.path()).unwrap()).unwrap();
    if let Some(arr) = plan_json.get_mut("clusters").and_then(|v| v.as_array_mut()) {
        if let Some(c0) = arr.get_mut(0) {
            let obj = c0.as_object_mut().unwrap();
            obj.insert(
                "label".into(),
                serde_json::Value::String("retrieval systems".into()),
            );
            // Remove tags key to trigger derivation from label
            obj.remove("tags");
        }
    }
    std::fs::write(
        plan_path.path(),
        serde_json::to_string_pretty(&plan_json).unwrap(),
    )
    .unwrap();

    // Apply with frontmatter writes (and cache default true)
    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("ai")
        .arg("index")
        .arg("apply")
        .arg("--from")
        .arg(plan_path.path())
        .arg("--write-frontmatter")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let report: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(report["ok"].as_bool().unwrap());
    assert!(report["written"]["cache"].as_bool().unwrap());
    assert!(report["written"]["frontmatter"].as_bool().unwrap());
    assert!(report["membersTagged"].as_i64().unwrap() >= 1);

    // Cache exists
    let cache = temp.child(".cli-rag/cache/ai-index.json");
    assert!(cache.path().exists());
    let _cache_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(cache.path()).unwrap()).unwrap();

    // At least one file received derived tag "retrieval-systems"
    let body = std::fs::read_to_string(base.child("ADR-010.md").path()).unwrap();
    assert!(body.contains("retrieval-systems"));

    temp.close().unwrap();
}

#[test]
fn ai_index_apply_hash_mismatch_exits_2() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();
    base.child("ADR-050.md")
        .write_str(
            "---\nid: ADR-050\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# ADR-050\n\n",
        )
        .unwrap();

    // Build index and plan
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
    let plan_path = temp.child("plan.json");
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("ai")
        .arg("index")
        .arg("plan")
        .arg("--min-cluster-size")
        .arg("1")
        .arg("--output")
        .arg(plan_path.path())
        .assert()
        .success();

    // Corrupt the sourceIndexHash in the plan
    let mut plan_text = std::fs::read_to_string(plan_path.path()).unwrap();
    plan_text = plan_text.replace("sha256:", "sha256:deadbeef");
    std::fs::write(plan_path.path(), plan_text).unwrap();

    // Apply should exit with code 2
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("ai")
        .arg("index")
        .arg("apply")
        .arg("--from")
        .arg(plan_path.path())
        .assert()
        .failure()
        .code(2);

    temp.close().unwrap();
}

#[test]
fn ai_index_apply_alias_prints_deprecation() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();
    base.child("ADR-200.md")
        .write_str(
            "---\nid: ADR-200\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# ADR-200\n\n",
        )
        .unwrap();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    let plan_path = temp.child("plan_alias.json");
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("ai-index-plan")
        .arg("--min-cluster-size")
        .arg("1")
        .arg("--output")
        .arg(plan_path.path())
        .assert()
        .success();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("ai-index-apply")
        .arg("--from")
        .arg(plan_path.path())
        .arg("--dry-run")
        .assert()
        .success()
        .stderr(contains("Deprecated: use `cli-rag ai index apply`"));

    temp.close().unwrap();
}

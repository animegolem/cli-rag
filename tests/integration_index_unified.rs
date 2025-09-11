use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;

fn write_adr(file: &assert_fs::fixture::ChildPath, id: &str, dep: Option<&str>, body_extra: &str) {
    let mut fm = format!("---\nid: {id}\ntags: [x]\nstatus: draft\n");
    if let Some(d) = dep {
        fm.push_str(&format!("depends_on: [\"{d}\"]\n"));
    } else {
        fm.push_str("depends_on: []\n");
    }
    fm.push_str("---\n\n");
    let content = format!("{fm}# {id}: Title\n\nBody\n{body_extra}\n");
    file.write_str(&content).unwrap();
}

#[test]
fn unified_index_includes_kinded_edges_and_locations() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    // ADR-002 target (no deps)
    write_adr(&base.child("ADR-002.md"), "ADR-002", None, "");
    // ADR-001 depends_on ADR-002 and mentions it in body on a known line
    write_adr(
        &base.child("ADR-001.md"),
        "ADR-001",
        Some("ADR-002"),
        "Here we reference [[ADR-002]] for context.",
    );

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = \"ADR\"\nfile_patterns = [\"ADR-*.md\"]\nunknown_policy = \"ignore\"\n",
        base.path().display()
    ))
    .unwrap();

    // Run validate (non-dry) to write unified index
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // Load unified index
    // Index path defaults to config.index_relative
    let idx_path = temp.child("index/adr-index.json");
    idx_path.assert(predicate::path::exists());
    let data = std::fs::read_to_string(idx_path.path()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&data).unwrap();
    // Required top-level keys
    assert!(v.get("version").is_some());
    assert!(v.get("generatedAt").is_some());
    assert!(v.get("nodes").and_then(|n| n.as_array()).is_some());
    assert!(v.get("edges").and_then(|n| n.as_array()).is_some());

    let edges = v.get("edges").unwrap().as_array().unwrap();
    // Has depends_on edge ADR-001 -> ADR-002
    assert!(edges.iter().any(|e| {
        e.get("from") == Some(&serde_json::Value::String("ADR-001".into()))
            && e.get("to") == Some(&serde_json::Value::String("ADR-002".into()))
            && e.get("kind") == Some(&serde_json::Value::String("depends_on".into()))
    }));
    // Has mentions edge with locations
    assert!(edges.iter().any(|e| {
        e.get("from") == Some(&serde_json::Value::String("ADR-001".into()))
            && e.get("to") == Some(&serde_json::Value::String("ADR-002".into()))
            && e.get("kind") == Some(&serde_json::Value::String("mentions".into()))
            && e.get("locations")
                .and_then(|l| l.as_array())
                .map(|a| !a.is_empty())
                .unwrap_or(false)
    }));

    temp.close().unwrap();
}

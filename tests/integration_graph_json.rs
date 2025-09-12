use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn write_adr(file: &assert_fs::fixture::ChildPath, id: &str, dep: Option<&str>) {
    let mut fm = format!("---\nid: {id}\ntags: [x]\nstatus: draft\n");
    if let Some(d) = dep {
        fm.push_str(&format!("depends_on: [\"{d}\"]\n"));
    } else {
        fm.push_str("depends_on: []\n");
    }
    fm.push_str("---\n\n");
    let content = format!("{fm}# {id}: Title\n\nBody\n");
    file.write_str(&content).unwrap();
}

#[test]
fn graph_json_contract_shape() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write_adr(&base.child("ADR-002.md"), "ADR-002", None);
    write_adr(&base.child("ADR-001.md"), "ADR-001", Some("ADR-002"));

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = \"ADR\"\nfile_patterns = [\"ADR-*.md\"]\nunknown_policy = \"ignore\"\n",
        base.path().display()
    ))
    .unwrap();

    // Build index
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .assert()
        .success();

    // Graph JSON
    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("graph")
        .arg("--id")
        .arg("ADR-001")
        .arg("--graph-format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["protocolVersion"].as_i64(), Some(1));
    assert!(v.get("root").is_some());
    assert_eq!(v["root"]["id"].as_str(), Some("ADR-001"));
    assert!(v.get("nodes").and_then(|n| n.as_array()).is_some());
    assert!(v.get("edges").and_then(|e| e.as_array()).is_some());
    // Has depends_on kinded edge
    let edges = v["edges"].as_array().unwrap();
    assert!(edges
        .iter()
        .any(|e| { e["from"] == "ADR-001" && e["to"] == "ADR-002" && e["kind"] == "depends_on" }));

    temp.close().unwrap();
}

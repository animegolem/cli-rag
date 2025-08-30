use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn write_simple_adr(file: &assert_fs::fixture::ChildPath, id: &str, group: &str) {
    let content = format!(
        "---\nid: {id}\ntags: [x]\nstatus: draft\ngroups: [\"{group}\"]\ndepends_on: []\n---\n\n# {id}: Title\n\nBody\n"
    );
    file.write_str(&content).unwrap();
}

#[test]
fn search_returns_protocol_with_groups() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let adr = base.child("ADR-001.md");
    write_simple_adr(&adr, "ADR-001", "G1");

    let cfg = temp.child(".adr-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    let mut cmd = Command::cargo_bin("adr-rag").unwrap();
    let output = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("search")
        .arg("-q")
        .arg("ADR-001")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"].as_str().unwrap(), "ADR-001");
    assert_eq!(
        arr[0]["groups"].as_array().unwrap()[0].as_str().unwrap(),
        "G1"
    );

    temp.close().unwrap();
}

#[test]
fn topics_counts_groups_from_docs() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write_simple_adr(&base.child("ADR-001.md"), "ADR-001", "Tools");

    let cfg = temp.child(".adr-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    let mut cmd = Command::cargo_bin("adr-rag").unwrap();
    let output = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("topics")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let arr = v.as_array().unwrap();
    assert!(arr.iter().any(|e| e["topic"] == "Tools" && e["count"] == 1));

    temp.close().unwrap();
}

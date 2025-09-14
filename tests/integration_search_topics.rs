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

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
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
    let arr = v["results"].as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["kind"].as_str().unwrap(), "note");
    assert_eq!(arr[0]["id"].as_str().unwrap(), "ADR-001");

    temp.close().unwrap();
}

// topics command removed per ADR-003d

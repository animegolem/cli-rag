use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn write_adr(file: &assert_fs::fixture::ChildPath, id: &str) {
    let content = format!(
        "---\nid: {id}\ntags: [x]\nstatus: draft\ngroups: [\"G\"]\ndepends_on: []\n---\n\n# {id}: Title\n\nBody\n"
    );
    file.write_str(&content).unwrap();
}

#[test]
fn get_ai_returns_content_blocks() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let adr = base.child("ADR-123.md");
    write_adr(&adr, "ADR-123");

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // Validate to generate unified index (optional for get, but aligns with other commands)
    let mut v = Command::cargo_bin("cli-rag").unwrap();
    v.arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    let out = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("get")
        .arg("--id")
        .arg("ADR-123")
        .arg("--format")
        .arg("ai")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["id"], "ADR-123");
    assert!(v["content"].is_array());
    let arr = v["content"].as_array().unwrap();
    assert!(arr.len() >= 2);
    assert_eq!(arr[0]["type"], "resource_link");
    assert_eq!(arr[1]["type"], "text");

    temp.close().unwrap();
}

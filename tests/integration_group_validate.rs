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
fn group_json_wraps_members() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write_simple_adr(&base.child("ADR-010.md"), "ADR-010", "Tools");

    let cfg = temp.child(".adr-rag.toml");
    cfg.write_str(&format!("bases = [\n  \"{}\"\n]\n", base.path().display()))
        .unwrap();

    let mut cmd = Command::cargo_bin("adr-rag").unwrap();
    let out = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("group")
        .arg("--topic")
        .arg("Tools")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["topic"], "Tools");
    assert_eq!(v["count"], 1);
    assert_eq!(v["adrs"].as_array().unwrap()[0]["id"], "ADR-010");

    temp.close().unwrap();
}

#[test]
fn validate_ndjson_header_only_on_empty() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    let cfg = temp.child(".adr-rag.toml");
    cfg.write_str(&format!("bases = [\n  \"{}\"\n]\n", base.path().display()))
        .unwrap();

    let mut cmd = Command::cargo_bin("adr-rag").unwrap();
    let output = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("ndjson")
        .arg("--dry-run")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let s = String::from_utf8(output).unwrap();
    // Should contain a single header line with ok true and doc_count 0
    let mut lines = s.lines();
    let header = lines.next().unwrap();
    assert!(header.contains("\"ok\":true"));
    assert!(header.contains("\"doc_count\":0"));
    assert!(lines.next().is_none());

    temp.close().unwrap();
}

#[test]
fn group_ndjson_emits_header_then_members() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write_simple_adr(&base.child("ADR-020.md"), "ADR-020", "Tools");

    let cfg = temp.child(".adr-rag.toml");
    cfg.write_str(&format!("bases = [\n  \"{}\"\n]\n", base.path().display()))
        .unwrap();

    let mut cmd = Command::cargo_bin("adr-rag").unwrap();
    let out = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("group")
        .arg("--topic")
        .arg("Tools")
        .arg("--format")
        .arg("ndjson")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    let mut lines = s.lines();
    let header = lines.next().unwrap();
    assert!(header.contains("\"topic\":\"Tools\""));
    assert!(header.contains("\"count\":1"));
    let first = lines.next().unwrap();
    assert!(first.contains("\"id\":\"ADR-020\""));
    assert!(lines.next().is_none());

    temp.close().unwrap();
}

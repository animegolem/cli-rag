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
fn unified_index_reader_works_without_per_base() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write_adr(&base.child("ADR-900.md"), "ADR-900");

    // Config and validate (writes unified + per-base)
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    let mut v = Command::cargo_bin("cli-rag").unwrap();
    v.arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // Remove per-base index but keep unified
    let per_base_idx = base.path().join("index/adr-index.json");
    if per_base_idx.exists() {
        std::fs::remove_file(&per_base_idx).unwrap();
    }

    // Search should still succeed (reads unified)
    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    let out = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("search")
        .arg("-q")
        .arg("ADR-900")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);

    temp.close().unwrap();
}

#[test]
fn fallback_to_scan_when_no_indexes() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write_adr(&base.child("ADR-901.md"), "ADR-901");

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // No validate run → no indexes present. Search should fallback to scan and succeed.
    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    let out = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("search")
        .arg("-q")
        .arg("ADR-901")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);

    temp.close().unwrap();
}

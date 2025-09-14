use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn validate_json_shape_and_writes_groups() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    // Minimal config pointing to empty base
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // Run validate in JSON mode and request writing groups
    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    let out = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(v["ok"].as_bool().unwrap());
    assert_eq!(v["docCount"].as_u64().unwrap(), 0);
    assert!(v["diagnostics"].as_array().unwrap().is_empty());

    // Groups file removed per ADR-003d; validate focuses on index/diagnostics only

    temp.close().unwrap();
}

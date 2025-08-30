use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn validate_json_shape_and_writes_groups() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    // Minimal config pointing to empty base
    let cfg = temp.child(".adr-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // Run validate in JSON mode and request writing groups
    let mut cmd = Command::cargo_bin("adr-rag").unwrap();
    let out = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .arg("--write-groups")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(v["ok"].as_bool().unwrap());
    assert_eq!(v["doc_count"].as_u64().unwrap(), 0);
    assert!(v["errors"].as_array().unwrap().is_empty());
    assert!(v["warnings"].as_array().unwrap().is_empty());

    // Groups file should exist (empty sections array)
    let groups_path = base.path().join("index/semantic-groups.json");
    assert!(groups_path.exists());
    let groups: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(groups_path).unwrap()).unwrap();
    assert!(groups["sections"].is_array());

    temp.close().unwrap();
}

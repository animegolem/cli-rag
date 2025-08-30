use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn doctor_json_on_empty_base_reports_structure() {
    // Setup isolated temp repo with a minimal config and empty base
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    let cfg = temp.child(".adr-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // Run `adr-rag doctor --format json --config <cfg>`
    let mut cmd = Command::cargo_bin("adr-rag").unwrap();
    let assert = cmd
        .arg("doctor")
        .arg("--format")
        .arg("json")
        .arg("--config")
        .arg(cfg.path())
        .assert()
        .success();

    // Parse JSON and assert a few stable fields
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(v
        .get("config")
        .and_then(|x| x.as_str())
        .unwrap()
        .ends_with(".adr-rag.toml"));
    assert_eq!(v["counts"]["docs"].as_u64().unwrap_or(999), 0);
    let per_base = v["per_base"].as_array().unwrap();
    assert_eq!(per_base.len(), 1);
    assert_eq!(per_base[0]["mode"].as_str().unwrap(), "scan");

    temp.close().unwrap();
}

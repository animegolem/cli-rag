use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn info_json_on_empty_base_reports_structure() {
    // Setup isolated temp repo with a minimal config and empty base
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // Run `cli-rag info --format json --config <cfg>`
    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    let assert = cmd
        .arg("info")
        .arg("--format")
        .arg("json")
        .arg("--config")
        .arg(cfg.path())
        .assert()
        .success();

    // Parse JSON and assert a few stable fields aligned to contracts
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(v["protocolVersion"].as_u64().unwrap() >= 1);
    assert!(v["config"]["path"]
        .as_str()
        .unwrap()
        .ends_with(".cli-rag.toml"));
    let idx_path = v["index"]["path"].as_str().unwrap();
    assert!(
        !idx_path.is_empty(),
        "index.path should be a non-empty string"
    );
    assert!(
        v["index"]["exists"].is_boolean(),
        "index.exists should be a boolean"
    );
    assert!(v["cache"]["aiIndexPath"]
        .as_str()
        .unwrap()
        .contains(".cli-rag"));
    assert!(
        v["capabilities"]["aiGet"]["retrievalVersion"]
            .as_u64()
            .unwrap()
            >= 1
    );

    temp.close().unwrap();
}

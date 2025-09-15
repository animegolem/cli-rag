use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn config_version_flows_when_present() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "config_version = \"1.0\"\n
bases = [\n  '{}'\n]\n",
        base.path().display()
    ))
    .unwrap();

    // info should expose config.version
    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("info")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["config"]["version"].as_str().unwrap(), "1.0");

    // validate should write resolved.json with configVersion
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
    let resolved_path = temp.child(".cli-rag/resolved.json");
    assert!(resolved_path.path().exists());
    let r: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(resolved_path.path()).unwrap()).unwrap();
    assert_eq!(r["configVersion"].as_str().unwrap(), "1.0");

    temp.close().unwrap();
}

#[test]
fn config_version_defaults_when_absent() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("info")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["config"]["version"].as_str().unwrap(), "0.1");

    // Resolved snapshot also defaults
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
    let resolved_path = temp.child(".cli-rag/resolved.json");
    let r: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(resolved_path.path()).unwrap()).unwrap();
    assert_eq!(r["configVersion"].as_str().unwrap(), "0.1");

    temp.close().unwrap();
}

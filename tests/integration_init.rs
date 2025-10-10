use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use serde_json::Value;
use std::process::Command;

#[test]
fn init_writes_config_silently() {
    let temp = assert_fs::TempDir::new().unwrap();
    // Run `cli-rag init --silent --force --preset project` in a temp dir
    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .arg("--preset")
        .arg("project")
        .arg("--silent")
        .arg("--force")
        .assert()
        .success();

    // Verify file exists and contains recognizable header
    let cfg = temp.child(".cli-rag.toml");
    cfg.assert(predicates::path::exists());
    cfg.assert(predicates::str::contains("Repo-local CLI config (cli-rag)"));
    cfg.assert(predicates::str::contains("docs/RAG"));
    cfg.assert(predicates::str::contains("ADR = \"docs/RAG/ADR\""));

    let template = temp.child(".cli-rag/templates/ADR.toml");
    template.assert(predicates::path::exists());

    temp.close().unwrap();
}

#[test]
fn init_project_emits_json() {
    let temp = assert_fs::TempDir::new().unwrap();

    let output = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .arg("--preset")
        .arg("project")
        .arg("--silent")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let payload: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(payload["preset"], "project");
    assert_eq!(payload["dryRun"], false);
    assert!(payload["created"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v.as_str().unwrap().ends_with(".cli-rag.toml")));
    assert!(payload["created"]
        .as_array()
        .unwrap()
        .iter()
        .any(|v| v.as_str().unwrap().ends_with(".cli-rag/templates/ADR.toml")));

    temp.close().unwrap();
}

#[test]
fn init_not_implemented_generic() {
    let temp = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .arg("--preset")
        .arg("generic")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Generic preset not implemented"));

    temp.close().unwrap();
}

#[test]
fn init_backup_creates_bak() {
    let temp = assert_fs::TempDir::new().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str("old-config").unwrap();

    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .arg("--preset")
        .arg("project")
        .arg("--silent")
        .env("CLI_RAG_INIT_OVERWRITE", "backup")
        .assert()
        .success();

    cfg.assert(predicates::str::contains("Repo-local CLI config (cli-rag)"));
    let bak = temp.child(".cli-rag.toml.bak");
    bak.assert(predicates::path::exists());
    bak.assert(predicates::str::contains("old-config"));

    temp.close().unwrap();
}

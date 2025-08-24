use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn init_writes_config_silently() {
    let temp = assert_fs::TempDir::new().unwrap();
    // Run `adr-rag init --silent --force` in a temp dir
    let mut cmd = Command::cargo_bin("adr-rag").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .arg("--silent")
        .arg("--force")
        .assert()
        .success();

    // Verify file exists and contains recognizable header
    let cfg = temp.child(".adr-rag.toml");
    cfg.assert(predicates::path::exists());
    cfg.assert(predicates::str::contains("Repo-local ADR CLI config"));

    temp.close().unwrap();
}

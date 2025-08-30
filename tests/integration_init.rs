use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn init_writes_config_silently() {
    let temp = assert_fs::TempDir::new().unwrap();
    // Run `cli-rag init --silent --force` in a temp dir
    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .arg("--silent")
        .arg("--force")
        .assert()
        .success();

    // Verify file exists and contains recognizable header
    let cfg = temp.child(".cli-rag.toml");
    cfg.assert(predicates::path::exists());
    cfg.assert(predicates::str::contains("Repo-local CLI config (cli-rag)"));

    temp.close().unwrap();
}

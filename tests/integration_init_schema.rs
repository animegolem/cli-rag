use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn init_appends_inline_schema() {
    let temp = assert_fs::TempDir::new().unwrap();
    // Write base config
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str("bases = [\n  'notes'\n]\n").unwrap();

    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .arg("--silent")
        .arg("--schema")
        .arg("ADR")
        .assert()
        .success();

    cfg.assert(predicates::str::contains("[[schema]]"));
    cfg.assert(predicates::str::contains("name = \"ADR\""));
}

#[test]
fn init_writes_separate_schema_and_imports() {
    let temp = assert_fs::TempDir::new().unwrap();
    // Write base config
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str("bases = [\n  'notes'\n]\n").unwrap();

    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .arg("--silent")
        .arg("--schema")
        .arg("IMP")
        .arg("--separate")
        .assert()
        .success();

    // Template file exists
    let tmpl = temp.child(".cli-rag/templates/IMP.toml");
    tmpl.assert(predicates::path::exists());
    tmpl.assert(predicates::str::contains("[[schema]]"));
    tmpl.assert(predicates::str::contains("name = \"IMP\""));

    // Body template stub exists
    let body = temp.child(".cli-rag/templates/IMP.md");
    body.assert(predicates::path::exists());
    body.assert(predicates::str::contains("id: {{id}}"));

    // Config has import list
    cfg.assert(predicates::str::contains("import = ["));
    cfg.assert(predicates::str::contains(".cli-rag/templates/IMP.toml"));
}

#[test]
fn init_inline_idempotent() {
    let temp = assert_fs::TempDir::new().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str("bases = [\n  'notes'\n]\n").unwrap();

    // First run
    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .arg("--silent")
        .arg("--schema")
        .arg("ADR")
        .assert()
        .success();
    // Second run — should not duplicate the schema block
    let mut cmd2 = Command::cargo_bin("cli-rag").unwrap();
    cmd2.current_dir(temp.path())
        .arg("init")
        .arg("--silent")
        .arg("--schema")
        .arg("ADR")
        .assert()
        .success();

    let body = std::fs::read_to_string(cfg.path()).unwrap();
    let count = body.matches("name = \"ADR\"").count();
    assert_eq!(count, 1, "should not duplicate schema block for ADR");
}

#[test]
fn init_separate_idempotent_import() {
    let temp = assert_fs::TempDir::new().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str("bases = [\n  'notes'\n]\n").unwrap();

    // First run
    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    cmd.current_dir(temp.path())
        .arg("init")
        .arg("--silent")
        .arg("--schema")
        .arg("IMP")
        .arg("--separate")
        .assert()
        .success();
    // Second run — should not duplicate import entry
    let mut cmd2 = Command::cargo_bin("cli-rag").unwrap();
    cmd2.current_dir(temp.path())
        .arg("init")
        .arg("--silent")
        .arg("--schema")
        .arg("IMP")
        .arg("--separate")
        .assert()
        .success();

    let body = std::fs::read_to_string(cfg.path()).unwrap();
    let needle = ".cli-rag/templates/IMP.toml";
    let count = body.matches(needle).count();
    assert_eq!(count, 1, "should not duplicate import entry");
}

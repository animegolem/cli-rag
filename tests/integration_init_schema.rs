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

    // Config has import list
    cfg.assert(predicates::str::contains("import = ["));
    cfg.assert(predicates::str::contains(".cli-rag/templates/IMP.toml"));
}

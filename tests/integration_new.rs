use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn write_base_cfg(dir: &assert_fs::TempDir, base_rel: &str) -> assert_fs::fixture::ChildPath {
    let cfg = dir.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n",
        dir.child(base_rel).path().display()
    ))
    .unwrap();
    cfg
}

#[test]
fn new_creates_note_from_template() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = write_base_cfg(&temp, "notes");

    // Scaffold schema and body template
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .arg("--silent")
        .arg("--schema")
        .arg("ADR")
        .arg("--separate")
        .assert()
        .success();

    // Create note
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--title")
        .arg("Hello World")
        .assert()
        .success();

    // Verify file exists and content replaced
    let f = base.child("ADR-001.md");
    f.assert(predicates::path::exists());
    let s = std::fs::read_to_string(f.path()).unwrap();
    assert!(s.contains("id: ADR-001"));
    assert!(s.contains("# ADR-001: Hello World"));
    drop(cfg);
    temp.close().unwrap();
}

#[test]
fn new_dry_run_does_not_write() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let _cfg = write_base_cfg(&temp, "notes");

    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("IMP")
        .arg("--dry-run")
        .assert()
        .success();

    base.child("IMP-001.md").assert(predicates::path::missing());
    temp.close().unwrap();
}

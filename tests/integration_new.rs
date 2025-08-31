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

#[test]
fn new_twice_increments_id() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let _cfg = write_base_cfg(&temp, "notes");

    // First note
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .assert()
        .success();
    // Second note
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .assert()
        .success();

    base.child("ADR-001.md").assert(predicates::path::exists());
    base.child("ADR-002.md").assert(predicates::path::exists());
    temp.close().unwrap();
}

#[test]
fn new_print_body_prints() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let _cfg = write_base_cfg(&temp, "notes");

    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--print-body")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("id: ADR-001"));
    assert!(s.contains("# ADR-001:"));
    temp.close().unwrap();
}

#[test]
fn new_filename_template_creates_expected_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let _cfg = write_base_cfg(&temp, "notes");

    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--title")
        .arg("hello")
        .arg("--filename-template")
        .arg("{{id}}-{{title}}.md")
        .assert()
        .success();

    base.child("ADR-001-hello.md")
        .assert(predicates::path::exists());
    temp.close().unwrap();
}

#[test]
fn new_writes_to_specified_dest_base() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base1 = temp.child("notes1");
    let base2 = temp.child("notes2");
    base1.create_dir_all().unwrap();
    base2.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}',\n  '{}'\n]\n",
        base1.path().display(),
        base2.path().display()
    ))
    .unwrap();

    // Write into base2 via --dest-base
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--dest-base")
        .arg(base2.path())
        .assert()
        .success();

    base1
        .child("ADR-001.md")
        .assert(predicates::path::missing());
    base2.child("ADR-001.md").assert(predicates::path::exists());
    temp.close().unwrap();
}

#[test]
fn new_injects_frontmatter_when_token_present() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = write_base_cfg(&temp, "notes");

    // Write a template that uses ((frontmatter)) token
    let tmpl_dir = temp.child(".cli-rag/templates");
    tmpl_dir.create_dir_all().unwrap();
    let tmpl = tmpl_dir.child("ADR.md");
    tmpl.write_str("---\n((frontmatter))\n---\n\n# {{id}}: {{title}}\n")
        .unwrap();

    // Create note
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--title")
        .arg("Injected")
        .assert()
        .success();

    let f = base.child("ADR-001.md");
    f.assert(predicates::path::exists());
    let s = std::fs::read_to_string(f.path()).unwrap();
    assert!(s.contains("id: ADR-001"));
    assert!(s.contains("tags: []"));
    assert!(s.contains("status: draft"));
    assert!(s.contains("depends_on: []"));
    assert!(s.contains("# ADR-001: Injected"));
    drop(cfg);
    temp.close().unwrap();
}

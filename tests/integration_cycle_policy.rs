use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn write_adr(file: &assert_fs::fixture::ChildPath, id: &str, dep: &str) {
    let content = format!(
        "---\nid: {id}\ntags: [x]\nstatus: draft\ngroups: []\ndepends_on: [\"{dep}\"]\n---\n\n# {id}: Title\n\nBody\n"
    );
    file.write_str(&content).unwrap();
}

#[test]
fn cycles_warn_by_default_or_warn_policy() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write_adr(&base.child("ADR-001.md"), "ADR-001", "ADR-002");
    write_adr(&base.child("ADR-002.md"), "ADR-002", "ADR-001");

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = \"ADR\"\nfile_patterns = [\"ADR-*.md\"]\nunknown_policy = \"ignore\"\ncycle_policy = \"warn\"\n",
        base.path().display()
    ))
    .unwrap();

    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    let out = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(v["ok"].as_bool().unwrap(), "should be ok (warning only)");
    let diags = v["diagnostics"].as_array().unwrap();
    assert!(diags.iter().any(|d| d["severity"] == "warning"
        && (d["code"] == "W240" || d["msg"].as_str().unwrap_or("").contains("cycle detected"))));

    temp.close().unwrap();
}

#[test]
fn cycles_error_with_error_policy() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write_adr(&base.child("ADR-101.md"), "ADR-101", "ADR-102");
    write_adr(&base.child("ADR-102.md"), "ADR-102", "ADR-101");

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = \"ADR\"\nfile_patterns = [\"ADR-*.md\"]\nunknown_policy = \"ignore\"\ncycle_policy = \"error\"\n",
        base.path().display()
    ))
    .unwrap();

    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    let out = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .failure() // exit 1 on not ok
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(!v["ok"].as_bool().unwrap(), "should fail (error on cycle)");
    let diags = v["diagnostics"].as_array().unwrap();
    assert!(diags.iter().any(|d| d["severity"] == "error"
        && (d["code"] == "E240" || d["msg"].as_str().unwrap_or("").contains("cycle detected"))));

    temp.close().unwrap();
}

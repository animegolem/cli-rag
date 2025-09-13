use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn write(file: &assert_fs::fixture::ChildPath, id: &str, body: &str, extra: &str) {
    let content = format!(
        "---\nid: {id}\ntags: [x]\nstatus: draft\n{extra}---\n\n# {id}: Title\n\n{body}\n"
    );
    file.write_str(&content).unwrap();
}

#[test]
fn get_json_neighbors_metadata_and_outline() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let a = base.child("ADR-300.md");
    let b = base.child("ADR-301.md");
    write(&a, "ADR-300", "# H1\nLine1\nLine2\n## H2\nL3\n", "depends_on: [\"ADR-301\"]\n");
    write(&b, "ADR-301", "# H\nX\nY\n", "");

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = \"ADR\"\nfile_patterns = [\"ADR-*.md\"]\nunknown_policy = \"ignore\"\n",
        base.path().display()
    ))
    .unwrap();

    // validate to build unified index (ensures stable scanning)
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // metadata (default)
    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("get")
        .arg("--id")
        .arg("ADR-300")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let ns = v["neighbors"].as_array().unwrap();
    assert!(ns.iter().any(|n| n["id"] == "ADR-301"));
    assert!(ns.iter().all(|n| n.get("content").is_none() && n.get("contentOutline").is_none()));

    // outline
    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("get")
        .arg("--id")
        .arg("ADR-300")
        .arg("--format")
        .arg("json")
        .arg("--neighbor-style")
        .arg("outline")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let ns = v["neighbors"].as_array().unwrap();
    let first = ns.iter().find(|n| n["id"] == "ADR-301").unwrap();
    assert!(first.get("contentOutline").is_some());
    assert!(first.get("content").is_none());
}

#[test]
fn get_json_policy_violation_full_depth_gt1() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write(&base.child("ADR-400.md"), "ADR-400", "", "depends_on: [\"ADR-401\"]\n");
    write(&base.child("ADR-401.md"), "ADR-401", "", "");

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = \"ADR\"\nfile_patterns = [\"ADR-*.md\"]\nunknown_policy = \"ignore\"\n",
        base.path().display()
    ))
    .unwrap();

    // Build unified index before running
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    let assert = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("get")
        .arg("--id")
        .arg("ADR-400")
        .arg("--format")
        .arg("json")
        .arg("--neighbor-style")
        .arg("full")
        .arg("--depth")
        .arg("2")
        .assert();
    assert.failure().code(2);
}

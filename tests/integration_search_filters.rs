use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;
fn write_note(file: &assert_fs::fixture::ChildPath, id: &str, body: &str) {
    let content = format!(
        "---\nid: {id}\ntags: [x]\nstatus: draft\ndepends_on: []\n---\n\n# {id}: Title\n\n{body}\n"
    );
    file.write_str(&content).unwrap();
}

#[test]
fn search_kind_filter_limits_items() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let body = "- [ ] Task one";
    write_note(&base.child("ADR-200.md"), "ADR-200", body);

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = \"ADR\"\nfile_patterns = [\"ADR-*.md\"]\nunknown_policy = \"ignore\"\n",
        base.path().display()
    ))
    .unwrap();

    // Only notes
    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("search")
        .arg("-q")
        .arg("ADR-200")
        .arg("--kind")
        .arg("note")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let res = v["results"].as_array().unwrap();
    assert!(res.iter().all(|e| e["kind"] == "note"));

    // Only todos
    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("search")
        .arg("-q")
        .arg("ADR-200")
        .arg("--kind")
        .arg("todo")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let res = v["results"].as_array().unwrap();
    assert!(res.iter().all(|e| e["kind"] == "todo"));

    temp.close().unwrap();
}

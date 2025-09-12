use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn write_note(file: &assert_fs::fixture::ChildPath, id: &str, body: &str, fm_extra: &str) {
    let content = format!(
        "---\nid: {id}\ntags: [x]\nstatus: draft\ndepends_on: []\n{fm_extra}---\n\n# {id}: Title\n\n{body}\n"
    );
    file.write_str(&content).unwrap();
}

#[test]
fn search_emits_todo_items_from_body() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let body = "- [ ] First task\nSome text\n- [x] Done task";
    write_note(&base.child("ADR-100.md"), "ADR-100", body, "");

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = \"ADR\"\nfile_patterns = [\"ADR-*.md\"]\nunknown_policy = \"ignore\"\n",
        base.path().display()
    ))
    .unwrap();

    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("search")
        .arg("-q")
        .arg("ADR-100")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let results = v["results"].as_array().unwrap();
    assert!(results.iter().any(|e| e["kind"] == "todo"));

    temp.close().unwrap();
}

#[test]
fn search_emits_kanban_item_from_frontmatter() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let fm_extra = "kanban_status: doing\nkanban_statusline: In progress\ndue_date: 2025-12-31\n";
    write_note(&base.child("ADR-101.md"), "ADR-101", "No tasks", fm_extra);

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = \"ADR\"\nfile_patterns = [\"ADR-*.md\"]\nunknown_policy = \"ignore\"\n",
        base.path().display()
    ))
    .unwrap();

    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("search")
        .arg("-q")
        .arg("ADR-101")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let results = v["results"].as_array().unwrap();
    assert!(results
        .iter()
        .any(|e| e["kind"] == "kanban" && e["kanbanStatus"] == "doing"));

    temp.close().unwrap();
}

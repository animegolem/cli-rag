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
        .arg("--kind")
        .arg("kanban,todo")
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

#[test]
fn search_emits_gtd_box_with_rank_and_due_and_span() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let body = "[@TODO:rank=high:due=2025-09-01] Review deployment";
    write_note(&base.child("ADR-102.md"), "ADR-102", body, "");

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
        .arg("ADR-102")
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
    let items = v["results"].as_array().unwrap();
    let t = items.iter().find(|e| e["kind"] == "todo").unwrap();
    assert_eq!(t["priorityScore"].as_i64(), Some(8));
    assert_eq!(t["dueDate"].as_str(), Some("2025-09-01"));
    assert_eq!(t["source"].as_str(), Some("body"));
    let span = t["span"].as_array().unwrap();
    assert_eq!(span.len(), 2);
    assert!(span[0].as_i64().unwrap() <= span[1].as_i64().unwrap());

    temp.close().unwrap();
}

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn write_adr(file: &assert_fs::fixture::ChildPath, id: &str, dep: Option<&str>, body_extra: &str) {
    let mut fm = format!("---\nid: {id}\ntags: [x]\nstatus: draft\n");
    if let Some(d) = dep {
        fm.push_str(&format!("depends_on: [\"{d}\"]\n"));
    } else {
        fm.push_str("depends_on: []\n");
    }
    fm.push_str("---\n\n");
    let content = format!("{fm}# {id}: Title\n\nBody\n{body_extra}\n");
    file.write_str(&content).unwrap();
}

#[test]
fn path_json_contract_shape_depends_on() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write_adr(&base.child("ADR-002.md"), "ADR-002", None, "");
    write_adr(&base.child("ADR-001.md"), "ADR-001", Some("ADR-002"), "");

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = \"ADR\"\nfile_patterns = [\"ADR-*.md\"]\nunknown_policy = \"ignore\"\n",
        base.path().display()
    ))
    .unwrap();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .assert()
        .success();

    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("path")
        .arg("--from")
        .arg("ADR-001")
        .arg("--to")
        .arg("ADR-002")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["protocolVersion"].as_i64(), Some(1));
    assert_eq!(v["ok"].as_bool(), Some(true));
    assert!(v["path"].as_array().map(|a| a.len()).unwrap_or(0) >= 2);
    let edges = v["edges"].as_array().unwrap();
    assert!(edges
        .iter()
        .any(|e| { e["from"] == "ADR-001" && e["to"] == "ADR-002" && e["kind"] == "depends_on" }));

    temp.close().unwrap();
}

#[test]
fn path_json_contract_shape_mentions_location() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write_adr(&base.child("ADR-029.md"), "ADR-029", None, "");
    write_adr(
        &base.child("IMP-006.md"),
        "IMP-006",
        None,
        "This mentions [[ADR-029]] on this line.",
    );
    write_adr(&base.child("ADR-024.md"), "ADR-024", Some("IMP-006"), "");

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = \"ADR\"\nfile_patterns = [\"ADR-*.md\"]\nunknown_policy = \"ignore\"\n\n[[schema]]\nname = \"IMP\"\nfile_patterns = [\"IMP-*.md\"]\nunknown_policy = \"ignore\"\n",
        base.path().display()
    ))
    .unwrap();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .assert()
        .success();

    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("path")
        .arg("--from")
        .arg("ADR-024")
        .arg("--to")
        .arg("ADR-029")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["protocolVersion"].as_i64(), Some(1));
    assert_eq!(v["ok"].as_bool(), Some(true));
    // Expect a mentions edge with locations
    let edges = v["edges"].as_array().unwrap();
    assert!(edges.iter().any(|e| {
        e["from"] == "IMP-006"
            && e["to"] == "ADR-029"
            && e["kind"] == "mentions"
            && e["locations"]
                .as_array()
                .map(|a| !a.is_empty())
                .unwrap_or(false)
    }));

    temp.close().unwrap();
}

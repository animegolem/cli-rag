use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serde_json::{Map as JsonMap, Value};
use std::io::Write;
use std::process::Command;

fn write_basic_config(temp: &assert_fs::TempDir) {
    let notes = temp.child("notes");
    notes.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(
        r#"bases = [
  "notes"
]

[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md"]
unknown_policy = "ignore"
"#,
    )
    .unwrap();
}

#[test]
fn ai_new_start_submit_flow() {
    let temp = assert_fs::TempDir::new().unwrap();
    write_basic_config(&temp);
    let cfg_path = temp.child(".cli-rag.toml");

    // Start draft
    let output = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "--config",
            cfg_path.path().to_str().unwrap(),
            "ai",
            "new",
            "start",
            "--schema",
            "ADR",
            "--title",
            "Circuit Breaker",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let start_json: Value = serde_json::from_slice(&output).unwrap();
    let draft_id = start_json["draftId"].as_str().unwrap();
    let filename = start_json["filename"].as_str().unwrap();
    let constraints = start_json["constraints"]["headings"].as_array().unwrap();

    // Build section payload based on headings
    let mut sections = JsonMap::new();
    for h in constraints {
        let name = h["name"].as_str().unwrap();
        sections.insert(
            name.to_string(),
            Value::String(format!("Content for {}", name)),
        );
    }
    let mut frontmatter = JsonMap::new();
    frontmatter.insert(
        "tags".into(),
        Value::Array(vec![Value::String("ai".to_string())]),
    );
    let mut payload_obj = JsonMap::new();
    payload_obj.insert("frontmatter".into(), Value::Object(frontmatter));
    payload_obj.insert("sections".into(), Value::Object(sections));
    let payload = Value::Object(payload_obj);

    // Submit draft via stdin
    let mut submit = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "--config",
            cfg_path.path().to_str().unwrap(),
            "ai",
            "new",
            "submit",
            "--draft",
            draft_id,
            "--stdin",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(mut stdin) = submit.stdin.take() {
        serde_json::to_writer(&mut stdin, &payload).unwrap();
        stdin.write_all(b"\n").unwrap();
    }
    let output = submit.wait_with_output().unwrap();
    assert!(output.status.success());
    let submit_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(submit_json["ok"].as_bool().unwrap());
    assert_eq!(submit_json["id"], start_json["id"]);

    // Final note exists
    temp.child(format!("notes/{}", filename))
        .assert(predicate::path::exists());

    temp.close().unwrap();
}

#[test]
fn ai_new_cancel_and_list() {
    let temp = assert_fs::TempDir::new().unwrap();
    write_basic_config(&temp);
    let cfg_path = temp.child(".cli-rag.toml");

    let mut draft_ids = Vec::new();
    for title in ["Async Queue", "Cache Policy"] {
        let out = Command::cargo_bin("cli-rag")
            .unwrap()
            .current_dir(temp.path())
            .args([
                "--config",
                cfg_path.path().to_str().unwrap(),
                "ai",
                "new",
                "start",
                "--schema",
                "ADR",
                "--title",
                title,
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let v: Value = serde_json::from_slice(&out).unwrap();
        draft_ids.push(v["draftId"].as_str().unwrap().to_string());
    }

    // List drafts
    let list_out = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "--config",
            cfg_path.path().to_str().unwrap(),
            "ai",
            "new",
            "list",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list_json: Value = serde_json::from_slice(&list_out).unwrap();
    assert_eq!(list_json["drafts"].as_array().unwrap().len(), 2);

    // Cancel first draft
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "--config",
            cfg_path.path().to_str().unwrap(),
            "ai",
            "new",
            "cancel",
            "--draft",
            &draft_ids[0],
        ])
        .assert()
        .success();

    let list_out = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "--config",
            cfg_path.path().to_str().unwrap(),
            "ai",
            "new",
            "list",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let list_json: Value = serde_json::from_slice(&list_out).unwrap();
    assert_eq!(list_json["drafts"].as_array().unwrap().len(), 1);

    temp.close().unwrap();
}

#[test]
fn ai_new_destination_mapping_routes_notes() {
    let temp = assert_fs::TempDir::new().unwrap();
    let notes = temp.child("notes");
    notes.create_dir_all().unwrap();
    let cfg_path = temp.child(".cli-rag.toml");
    cfg_path
        .write_str(
            r#"bases = ["notes"]

[authoring.destinations]
ADR = "notes/adr"

[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md"]

[schema.new]
filename_template = "{{id}}.md"
"#,
        )
        .unwrap();

    let start_out = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "--config",
            cfg_path.path().to_str().unwrap(),
            "ai",
            "new",
            "start",
            "--schema",
            "ADR",
            "--title",
            "Destination Mapping",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let start_json: Value = serde_json::from_slice(&start_out).unwrap();
    let draft_id = start_json["draftId"].as_str().unwrap();
    let filename = start_json["filename"].as_str().unwrap();

    // submit using same payload strategy as base test
    let mut sections = JsonMap::new();
    if let Some(headings) = start_json["constraints"]["headings"].as_array() {
        for h in headings {
            let name = h["name"].as_str().unwrap();
            sections.insert(name.to_string(), Value::String("Body".into()));
        }
    }
    let mut payload_obj = JsonMap::new();
    payload_obj.insert("frontmatter".into(), Value::Object(JsonMap::new()));
    payload_obj.insert("sections".into(), Value::Object(sections));
    let payload = Value::Object(payload_obj);

    let mut submit = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "--config",
            cfg_path.path().to_str().unwrap(),
            "ai",
            "new",
            "submit",
            "--draft",
            draft_id,
            "--stdin",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(mut stdin) = submit.stdin.take() {
        serde_json::to_writer(&mut stdin, &payload).unwrap();
        stdin.write_all(b"\n").unwrap();
    }
    let output = submit.wait_with_output().unwrap();
    assert!(output.status.success());

    notes
        .child(format!("adr/{}", filename))
        .assert(predicate::path::exists());
    temp.close().unwrap();
}

#[test]
fn ai_new_schema_output_path_overrides_global() {
    let temp = assert_fs::TempDir::new().unwrap();
    let notes = temp.child("notes");
    notes.create_dir_all().unwrap();
    let cfg_path = temp.child(".cli-rag.toml");
    cfg_path
        .write_str(
            r#"bases = ["notes"]

[authoring.destinations]
ADR = "notes/adr"

[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md"]

[schema.new]
filename_template = "{{id}}.md"
output_path = ["notes/custom"]
"#,
        )
        .unwrap();

    let start_out = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "--config",
            cfg_path.path().to_str().unwrap(),
            "ai",
            "new",
            "start",
            "--schema",
            "ADR",
            "--title",
            "Schema Override",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let start_json: Value = serde_json::from_slice(&start_out).unwrap();
    let draft_id = start_json["draftId"].as_str().unwrap();
    let filename = start_json["filename"].as_str().unwrap();

    let mut sections = JsonMap::new();
    if let Some(headings) = start_json["constraints"]["headings"].as_array() {
        for h in headings {
            let name = h["name"].as_str().unwrap();
            sections.insert(name.to_string(), Value::String("Body".into()));
        }
    }
    let mut payload_obj = JsonMap::new();
    payload_obj.insert("frontmatter".into(), Value::Object(JsonMap::new()));
    payload_obj.insert("sections".into(), Value::Object(sections));
    let payload = Value::Object(payload_obj);

    let mut submit = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "--config",
            cfg_path.path().to_str().unwrap(),
            "ai",
            "new",
            "submit",
            "--draft",
            draft_id,
            "--stdin",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(mut stdin) = submit.stdin.take() {
        serde_json::to_writer(&mut stdin, &payload).unwrap();
        stdin.write_all(b"\n").unwrap();
    }
    let output = submit.wait_with_output().unwrap();
    assert!(output.status.success());

    notes
        .child(format!("custom/{}", filename))
        .assert(predicate::path::exists());
    temp.close().unwrap();
}

#[test]
fn ai_new_rejects_destination_outside_base() {
    let temp = assert_fs::TempDir::new().unwrap();
    let notes = temp.child("notes");
    notes.create_dir_all().unwrap();
    let cfg_path = temp.child(".cli-rag.toml");
    cfg_path
        .write_str(
            r#"bases = ["notes"]

[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md"]

[schema.new]
output_path = ["../escape"]
"#,
        )
        .unwrap();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "--config",
            cfg_path.path().to_str().unwrap(),
            "ai",
            "new",
            "start",
            "--schema",
            "ADR",
            "--title",
            "Escape Hatch",
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("outside configured bases"));

    notes.assert(predicate::path::is_dir());
    temp.close().unwrap();
}

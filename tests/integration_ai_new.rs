use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serde_json::{json, Map as JsonMap, Value};
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

// Legacy repo templates have been removed; tests seed schema templates via TOML
// in the temporary config instead of copying Markdown from the repository.

#[test]
fn ai_new_start_submit_flow() {
    let temp = assert_fs::TempDir::new().unwrap();
    write_basic_config(&temp);
    let cfg_path = temp.child(".cli-rag.toml");

    // Start draft
    let stdout = Command::cargo_bin("cli-rag")
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
    let start_json: Value = serde_json::from_slice(&stdout).unwrap();
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
fn ai_new_cancel_without_id_auto_cancels_when_single_draft() {
    let temp = assert_fs::TempDir::new().unwrap();
    write_basic_config(&temp);
    let cfg_path = temp.child(".cli-rag.toml");

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
            "Auto Cancel",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let start_json: Value = serde_json::from_slice(&start_out).unwrap();
    let draft_id = start_json["draftId"].as_str().unwrap();

    let cancel_out = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "--config",
            cfg_path.path().to_str().unwrap(),
            "ai",
            "new",
            "cancel",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let cancel_json: Value = serde_json::from_slice(&cancel_out).unwrap();
    assert!(cancel_json["ok"].as_bool().unwrap());
    assert_eq!(cancel_json["draftId"].as_str().unwrap(), draft_id);

    // Ensure no drafts remain
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
    assert_eq!(list_json["drafts"].as_array().unwrap().len(), 0);

    temp.close().unwrap();
}

#[test]
fn ai_new_cancel_without_id_requires_choice_when_multiple_drafts_exist() {
    let temp = assert_fs::TempDir::new().unwrap();
    write_basic_config(&temp);
    let cfg_path = temp.child(".cli-rag.toml");

    for title in ["One", "Two"] {
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
                title,
            ])
            .assert()
            .success();
    }

    let stdout = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .args([
            "--config",
            cfg_path.path().to_str().unwrap(),
            "ai",
            "new",
            "cancel",
        ])
        .assert()
        .failure()
        .code(3)
        .get_output()
        .stdout
        .clone();
    let failure_json: Value = serde_json::from_slice(&stdout).unwrap();
    assert!(!failure_json["ok"].as_bool().unwrap());
    let diag = &failure_json["diagnostics"][0];
    assert_eq!(diag["code"], "MULTIPLE_DRAFTS");
    assert!(diag["message"].as_str().unwrap().contains("--draft"));

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

#[test]
fn ai_new_note_template_includes_contract_guidance() {
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
[schema.new.template]
[schema.new.template.prompt]
template = "Contract-guided authoring for ADRs."
[schema.new.template.note]
template = "---\n{{frontmatter}}---\n\n<!-- Keep this record concise and professional. Use short paragraphs. -->\n\n# {{title}}\n\n## Objective\n{{LOC|1}}\n"

[[schema]]
name = "IMP"
file_patterns = ["AI-IMP-*.md"]
[schema.new.template]
[schema.new.template.prompt]
template = "IMP guidance."
[schema.new.template.note]
template = "---\n{{frontmatter}}---\n\n<!-- Fill out the YAML frontmatter in full before drafting sections. -->\n# {{title}}\n\n## Summary of Issue #1\n{{LOC|1}}\n"

[[schema]]
name = "EPIC"
file_patterns = ["AI-EPIC-*.md"]
[schema.new.template]
[schema.new.template.prompt]
template = "EPIC guidance."
[schema.new.template.note]
template = "---\n{{frontmatter}}\ntags:\n  - EPIC\n---\n\n<!-- Fill out the YAML frontmatter above. Keep this epic focused. -->\n\n# {{title}}\n\n## Problem Statement/Feature Scope\n{{LOC|1}}\n"
"#,
        )
        .unwrap();

    let expectations = [
        (
            "ADR",
            "<!-- Keep this record concise and professional.",
            "## Objective",
        ),
        (
            "IMP",
            "<!-- Fill out the YAML frontmatter in full before drafting sections.",
            "## Summary of Issue #1",
        ),
        (
            "EPIC",
            "<!-- Fill out the YAML frontmatter above.",
            "## Problem Statement/Feature Scope",
        ),
    ];

    for (schema, guidance_snippet, heading) in expectations {
        let stdout = Command::cargo_bin("cli-rag")
            .unwrap()
            .current_dir(temp.path())
            .args([
                "--config",
                cfg_path.path().to_str().unwrap(),
                "ai",
                "new",
                "start",
                "--schema",
                schema,
                "--title",
                "Template Check",
                "--format",
                "json",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let start_json: Value = serde_json::from_slice(&stdout).unwrap();
        let note_template = start_json["noteTemplate"].as_str().unwrap();

        assert!(
            note_template.starts_with(
                "---
"
            ),
            "missing frontmatter block for {}",
            schema
        );
        assert!(
            note_template.contains(&format!("id: {}-", schema)),
            "expected id insertion for {} template",
            schema
        );
        if schema == "EPIC" {
            assert!(
                note_template.contains("tags:\n  - EPIC"),
                "expected EPIC template to seed tags list"
            );
        } else {
            assert!(
                note_template.contains("tags: []"),
                "expected default tags for {} template",
                schema
            );
        }
        assert!(
            note_template.contains("depends_on: []"),
            "expected default depends_on for {} template",
            schema
        );
        assert!(
            note_template.contains(guidance_snippet),
            "missing guidance comment '{}' for {} template",
            guidance_snippet,
            schema
        );
        assert!(
            note_template.contains(heading),
            "missing heading '{}' for {} template",
            heading,
            schema
        );
        assert!(
            !note_template.contains("{{frontmatter}}"),
            "frontmatter token not expanded for {} template",
            schema
        );
    }

    temp.close().unwrap();
}

#[test]
fn ai_new_template_precedence_prefers_lua() {
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
"#,
        )
        .unwrap();

    temp.child(".cli-rag.lua")
        .write_str(
            r#"overlay = {}

function overlay.template_prompt(ctx)
  return "Lua prompt for " .. ctx.schema.name
end

function overlay.template_note(ctx)
  return [[---
{{frontmatter}}
---

# Lua Body
]]
end

return overlay
"#,
        )
        .unwrap();

    let stdout = Command::cargo_bin("cli-rag")
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
            "Lua Wins",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let start_json: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(
        start_json["instructions"].as_str().unwrap(),
        "Lua prompt for ADR"
    );
    let note_template = start_json["noteTemplate"].as_str().unwrap();
    assert!(note_template.contains("# Lua Body"));
    assert!(note_template.contains("id: ADR-"));
    assert!(!note_template.contains("{{frontmatter}}"));

    temp.close().unwrap();
}

#[test]
fn ai_new_template_precedence_prefers_toml_when_no_lua() {
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

[schema.new.template.prompt]
template = "TOML prompt for {{schema.name}} writing {{filename}}"

[schema.new.template.note]
template = """---
{{frontmatter}}
---

# TOML Body
Filename: {{filename}}
"""
"#,
        )
        .unwrap();
    let stdout = Command::cargo_bin("cli-rag")
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
            "Toml Wins",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let start_json: Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(
        start_json["instructions"].as_str().unwrap(),
        "TOML prompt for ADR writing ADR-001.md"
    );
    let note_template = start_json["noteTemplate"].as_str().unwrap();
    assert!(note_template.contains("# TOML Body"));
    assert!(note_template.contains("id: ADR-"));
    assert!(!note_template.contains("{{frontmatter}}"));
    assert!(note_template.contains("Filename: ADR-001.md"));

    temp.close().unwrap();
}

#[test]
fn ai_new_start_includes_frontmatter_constraints_metadata() {
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
required = ["status"]
unknown_policy = "ignore"

[schema.rules.status]
enum = ["draft", "accepted"]

[schema.rules.related_files]
type = "array"
globs = ["*.md"]

[schema.rules.priority]
integer = { min = 0, max = 100 }

[schema.rules.confidence_score]
float = { min = 0.0, max = 1.0 }
"#,
        )
        .unwrap();

    let stdout = Command::cargo_bin("cli-rag")
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
            "Constraint Check",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let start_json: Value = serde_json::from_slice(&stdout).unwrap();
    let frontmatter = start_json["constraints"]["frontmatter"]
        .as_object()
        .unwrap();

    let allowed = frontmatter["allowed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(allowed.contains(&"priority"));
    assert!(allowed.contains(&"confidence_score"));
    assert!(allowed.contains(&"related_files"));

    let readonly = frontmatter["readonly"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(readonly.contains(&"id"));
    assert!(readonly.contains(&"created_date"));
    assert!(readonly.contains(&"last_modified"));

    let enums = frontmatter["enums"].as_object().unwrap();
    let status_values = enums["status"].as_array().unwrap();
    assert_eq!(
        status_values
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["draft", "accepted"]
    );

    let globs = frontmatter["globs"].as_object().unwrap();
    let related_patterns = globs["related_files"].as_array().unwrap();
    assert_eq!(related_patterns[0].as_str().unwrap(), "*.md");

    let integers = frontmatter["integers"].as_object().unwrap();
    let priority_range = integers["priority"].as_object().unwrap();
    assert_eq!(priority_range["min"].as_i64().unwrap(), 0);
    assert_eq!(priority_range["max"].as_i64().unwrap(), 100);

    let floats = frontmatter["floats"].as_object().unwrap();
    let confidence_range = floats["confidence_score"].as_object().unwrap();
    assert!((confidence_range["min"].as_f64().unwrap() - 0.0).abs() < f64::EPSILON);
    assert!((confidence_range["max"].as_f64().unwrap() - 1.0).abs() < f64::EPSILON);

    temp.close().unwrap();
}

#[test]
fn ai_new_submit_blocks_readonly_frontmatter_override() {
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
unknown_policy = "ignore"
[schema.new.template]
[schema.new.template.prompt]
template = "ADR guidance."
[schema.new.template.note]
template = "---\n{{frontmatter}}---\n\n<!-- Keep this record concise and professional. Use short paragraphs. -->\n\n# {{title}}\n\n## Objective\n{{LOC|1}}\n"
"#,
        )
        .unwrap();

    let stdout = Command::cargo_bin("cli-rag")
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
            "Readonly Test",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let start_json: Value = serde_json::from_slice(&stdout).unwrap();
    let draft_id = start_json["draftId"].as_str().unwrap();

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
        .spawn()
        .unwrap();
    if let Some(mut stdin) = submit.stdin.take() {
        let payload = json!({
            "frontmatter": {
                "id": "ADR-999"
            },
            "sections": {}
        });
        serde_json::to_writer(&mut stdin, &payload).unwrap();
        stdin.write_all(b"\n").unwrap();
    }
    let output = submit.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    let failure_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(!failure_json["ok"].as_bool().unwrap());
    let diag = &failure_json["diagnostics"][0];
    assert_eq!(diag["code"], "READONLY_FIELD");
    assert!(diag["message"].as_str().unwrap().contains("readonly"));

    temp.close().unwrap();
}

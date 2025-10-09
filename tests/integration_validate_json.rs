use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use cli_rag::protocol::PROTOCOL_VERSION;
use std::process::Command;

#[test]
fn validate_json_shape_and_writes_groups() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    // Minimal config pointing to empty base
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // Run validate in JSON mode and request writing groups
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
    assert_eq!(
        v["protocolVersion"].as_u64().unwrap(),
        PROTOCOL_VERSION as u64
    );
    assert!(v["ok"].as_bool().unwrap());
    assert_eq!(v["docCount"].as_u64().unwrap(), 0);
    assert!(v["diagnostics"].as_array().unwrap().is_empty());

    // Groups file removed per ADR-003d; validate focuses on index/diagnostics only

    temp.close().unwrap();
}

#[test]
fn validate_enforces_enum_globs_and_numeric_bounds() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        r#"bases = ["{base}"]

[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md"]
unknown_policy = "ignore"

[schema.rules.status]
enum = ["draft", "accepted"]

[schema.rules.related_files]
type = "array"
globs = ["*.md"]

[schema.rules.priority]
integer = {{ min = 0, max = 100 }}

[schema.rules.confidence_score]
float = {{ min = 0.0, max = 1.0 }}
"#,
        base = base.path().display()
    ))
    .unwrap();

    base.child("ADR-001.md")
        .write_str(
            r#"---
id: ADR-001
status: legacy
related_files:
  - src/main.rs
priority: 200
confidence_score: 1.5
---

# ADR-001
"#,
        )
        .unwrap();

    let output = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .code(2)
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert!(!v["ok"].as_bool().unwrap());
    let diagnostics = v["diagnostics"].as_array().unwrap();
    assert!(diagnostics.len() >= 4);
    let messages: Vec<String> = diagnostics
        .iter()
        .map(|d| d["msg"].as_str().unwrap().to_string())
        .collect();
    assert!(messages
        .iter()
        .any(|m| m.contains("status") && m.contains("legacy")));
    assert!(messages
        .iter()
        .any(|m| m.contains("related_files") && m.contains("src/main.rs")));
    assert!(messages
        .iter()
        .any(|m| m.contains("priority") && m.contains("200")));
    assert!(messages
        .iter()
        .any(|m| m.contains("confidence_score") && m.contains("1.5")));

    temp.close().unwrap();
}

#[test]
fn validate_enforces_exact_heading_policy() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        r#"bases = ["{base}"]

[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md"]
unknown_policy = "ignore"

[schema.new.template.note]
template = "---\n{{{{frontmatter}}}}\n---\n\n# {{{{title}}}}\n\n## First\n{{{{LOC|2}}}}\n\n## Second\n{{{{LOC|2}}}}\n"

[schema.validate.body.headings]
heading_check = "exact"
"#,
        base = base.path().display()
    ))
    .unwrap();

    base.child("ADR-001.md")
        .write_str("---\nid: ADR-001\n---\n\n# ADR-001\n\n## Second\nLine A\n\n## First\nLine B\n")
        .unwrap();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicates::str::contains("headings do not match template"));

    temp.close().unwrap();
}

#[test]
fn validate_enforces_missing_only_heading_policy() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        r#"bases = ["{base}"]

[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md"]
unknown_policy = "ignore"

[schema.new.template.note]
template = "---\n{{{{frontmatter}}}}\n---\n\n# {{{{title}}}}\n\n## First\n{{{{LOC|2}}}}\n\n## Second\n{{{{LOC|2}}}}\n"

[schema.validate.body.headings]
heading_check = "missing_only"
"#,
        base = base.path().display()
    ))
    .unwrap();

    base.child("ADR-002.md")
        .write_str("---\nid: ADR-002\n---\n\n# ADR-002\n\n## First\nLine\n")
        .unwrap();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicates::str::contains("missing required headings"));

    temp.close().unwrap();
}

#[test]
fn validate_enforces_max_count_and_loc_on_validate() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        r#"bases = ["{base}"]

[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md"]
unknown_policy = "ignore"

[schema.new.template.note]
template = "---\n{{{{frontmatter}}}}\n---\n\n# {{{{title}}}}\n\n## First\n{{{{LOC|1}}}}\n\n## Second\n{{{{LOC|1}}}}\n"

[schema.validate.body.headings]
heading_check = "missing_only"
max_count = 2

[schema.validate.body.line_count]
scan_policy = "on_validate"
"#,
        base = base.path().display()
    ))
    .unwrap();

    base.child("ADR-003.md").write_str(
        "---\nid: ADR-003\n---\n\n# ADR-003\n\n## First\nLine one\nLine two\n\n## Second\nLine\n\n## Extra\nLine\n",
    )
    .unwrap();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .stdout(predicates::str::contains("heading count"))
        .stdout(predicates::str::contains("exceeds max lines"));

    temp.close().unwrap();
}

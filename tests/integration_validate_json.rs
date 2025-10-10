use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use cli_rag::protocol::PROTOCOL_VERSION;
use serde_json::Value;
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

#[test]
fn init_project_preset_and_validate_single_adr() {
    let temp = assert_fs::TempDir::new().unwrap();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .arg("--preset")
        .arg("project")
        .arg("--silent")
        .assert()
        .success();

    temp.child(".cli-rag/templates/ADR.toml")
        .assert(predicates::path::exists())
        .assert(predicates::str::contains("This is a HUMAN note Schema"));

    let adr_dir = temp.child("docs/RAG/ADR");
    adr_dir.create_dir_all().unwrap();
    adr_dir.child("ADR-001.md").write_str(concat!(
        "---\n",
        "node_id: ADR-001\n",
        "id: ADR-001\n",
        "tags: [architecture]\n",
        "status: draft\n",
        "depends_on: []\n",
        "created_date: 2025-10-10\n",
        "---\n",
        "\n",
        "# ADR-001: Validate ADR MVP\n",
        "\n",
        "## Objective\n",
        "Demonstrate that a single ADR validates successfully using the project preset.\n",
        "\n",
        "## Context\n",
        "This is a minimal note created by the integration test to exercise validate and indexing.\n",
        "\n",
        "## Decision\n",
        "Adopt the contract-aligned ADR template and ensure validate writes the unified index without errors.\n",
        "\n",
        "## Consequences\n",
        "Warnings may still surface for isolated notes; this is acceptable for MVP.\n",
        "\n",
        "## Updates\n",
        "None yet.\n",
    )).unwrap();

    let output = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .failure()
        .code(2)
        .get_output()
        .stdout
        .clone();

    let payload: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(
        payload["protocolVersion"],
        Value::from(cli_rag::protocol::PROTOCOL_VERSION)
    );
    assert!(!payload["ok"].as_bool().unwrap());
    assert_eq!(payload["docCount"].as_u64().unwrap(), 1);

    let diagnostics = payload["diagnostics"].as_array().unwrap();
    assert!(diagnostics
        .iter()
        .any(|d| d["code"] == "EDGE_REQUIRED_MISSING"));
    assert!(diagnostics.iter().any(|d| d["code"] == "W250"));
    for diag in diagnostics {
        let path = diag["path"].as_str().unwrap();
        assert!(
            path.ends_with("docs/RAG/ADR/ADR-001.md"),
            "unexpected diagnostic path {path}"
        );
    }

    temp.close().unwrap();
}

#[test]
fn validate_emits_stable_codes_for_edges_and_wikilinks() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        r#"bases = ["{base}"]

[[schema]]
name = "IMP"
file_patterns = ["IMP-*.md"]
unknown_policy = "ignore"
allowed_keys = ["implements"]

[schema.validate]
severity = "error"

[schema.validate.edges.wikilinks]
min_outgoing = 1
min_incoming = 1
severity = "warning"

[schema.validate.edges.cross_schema]
allowed_targets = ["IMP"]

[schema.validate.edges.implements]
required = "error"

[schema.validate.edges.depends_on]
required = "ignore"

[[schema]]
name = "ADR"
file_patterns = ["ADR-*.md"]
unknown_policy = "ignore"
"#,
        base = base.path().display()
    ))
    .unwrap();

    base.child("IMP-001.md")
        .write_str(
            r#"---
id: IMP-001
implements:
  - IMP-404
depends_on:
  - ADR-001
---

# IMP-001
"#,
        )
        .unwrap();

    base.child("IMP-002.md")
        .write_str(
            r#"---
id: IMP-002
---

# IMP-002
"#,
        )
        .unwrap();

    base.child("ADR-001.md")
        .write_str(
            r#"---
id: ADR-001
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
    let diagnostics = v["diagnostics"].as_array().unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|d| d["code"] == "EDGE_REQUIRED_MISSING" && d["severity"] == "error"),
        "EDGE_REQUIRED_MISSING code missing"
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d["code"] == "EDGE_ID_NOT_FOUND" && d["severity"] == "error"),
        "EDGE_ID_NOT_FOUND code missing"
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d["code"] == "EDGE_CROSS_SCHEMA_DISALLOWED" && d["severity"] == "error"),
        "EDGE_CROSS_SCHEMA_DISALLOWED code missing"
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d["code"] == "LINK_MIN_OUT" && d["severity"] == "warning"),
        "LINK_MIN_OUT code missing"
    );
    assert!(
        diagnostics
            .iter()
            .any(|d| d["code"] == "LINK_MIN_IN" && d["severity"] == "warning"),
        "LINK_MIN_IN code missing"
    );

    temp.close().unwrap();
}

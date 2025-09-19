use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use serde_json::{Map as JsonMap, Value};
use std::io::Write;
use std::process::Command;

#[test]
fn lua_new_overrides_id_and_frontmatter() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // overlay new hooks
    let overlay = temp.child(".cli-rag.lua");
    overlay
        .write_str(
            r#"return {
  id_generator = function(schema, ctx)
    return { id = schema .. "-999" }
  end,
  render_frontmatter = function(schema, title, ctx)
    return { status = "design", tags = {"lua","hook"} }
  end
}
"#,
        )
        .unwrap();

    // Start draft through AI flow
    let start_out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .args([
            "ai", "new", "start", "--schema", "ADR", "--title", "Lua Test", "--format", "json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let start_json: serde_json::Value = serde_json::from_slice(&start_out).unwrap();
    assert_eq!(start_json["id"].as_str().unwrap(), "ADR-999");
    let draft_id = start_json["draftId"].as_str().unwrap();
    let filename = start_json["filename"].as_str().unwrap();

    // Build payload from template headings
    let mut sections = JsonMap::new();
    if let Some(headings) = start_json["constraints"]["headings"].as_array() {
        for h in headings {
            let name = h["name"].as_str().unwrap();
            sections.insert(
                name.to_string(),
                serde_json::Value::String("Lua body".into()),
            );
        }
    }
    let mut payload_obj = JsonMap::new();
    payload_obj.insert("frontmatter".into(), Value::Object(JsonMap::new()));
    payload_obj.insert("sections".into(), Value::Object(sections));
    let payload = Value::Object(payload_obj);

    let mut submit = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .args(["ai", "new", "submit", "--draft", draft_id, "--stdin"])
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

    // Check file exists with id ADR-999 and Lua overrides
    let created = base.child(filename);
    assert!(created.path().exists());
    let content = std::fs::read_to_string(created.path()).unwrap();
    assert!(content.contains("id: ADR-999"));
    assert!(content.contains("status: design"));
    assert!(content.contains("- lua"));
    assert!(content.contains("- hook"));

    temp.close().unwrap();
}

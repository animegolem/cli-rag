use assert_cmd::prelude::*;
use assert_fs::prelude::*;
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

    // Create new note
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--title")
        .arg("Lua Test")
        .assert()
        .success();

    // Check file exists with id ADR-999
    let created = base.child("ADR-999.md");
    assert!(created.path().exists());
    let content = std::fs::read_to_string(created.path()).unwrap();
    assert!(content.contains("id: ADR-999"));
    assert!(content.contains("status: design"));
    assert!(content.contains("- lua"));
    assert!(content.contains("- hook"));

    temp.close().unwrap();
}

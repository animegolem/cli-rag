use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn lua_validate_adds_warning() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    // minimal note
    let note = base.child("ADR-001.md");
    note.write_str(
        "---\nid: ADR-001\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# ADR-001: Title\n",
    )
    .unwrap();
    // overlay validate returns one warning
    let overlay = temp.child(".cli-rag.lua");
    overlay
        .write_str(
            r#"return {
  validate = function(note, ctx)
    return { diagnostics = { { severity = "warning", code = "LUA_TEST", msg = "hello from lua" } } }
  end
}
"#,
        )
        .unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .arg("--dry-run")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(v["ok"].as_bool().unwrap());
    let diags = v["diagnostics"].as_array().unwrap();
    assert!(diags.iter().any(|d| d["code"] == "LUA_TEST"));
    assert!(diags
        .iter()
        .any(|d| d["msg"].as_str().unwrap().contains("hello from lua")));

    temp.close().unwrap();
}

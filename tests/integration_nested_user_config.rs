use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn nested_only_config_is_accepted_and_writes_resolved() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "[config]\nconfig_version = \"1.2\"\n\n[config.scan]\nfilepaths = [\n  '{}'\n]\nindex_path = 'alt/index.json'\nignore_globs = [\"**/tmp/**\"]\n\n[config.graph]\ndepth = 3\ninclude_bidirectional = false\n\n[config.templates]\nimport = [\".cli-rag/templates/ADR.toml\"]\n",
        base.path().display()
    ))
    .unwrap();

    // Run validate (non-dry) to write resolved.json
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    let resolved_path = temp.child(".cli-rag/resolved.json");
    assert!(resolved_path.path().exists());
    let r: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(resolved_path.path()).unwrap()).unwrap();
    assert_eq!(r["configVersion"].as_str().unwrap(), "1.2");
    assert_eq!(r["graph"]["depth"].as_i64().unwrap(), 3);
    assert_eq!(r["graph"]["includeBidirectional"].as_bool().unwrap(), false);
    // indexPath should reflect nested index_path
    let idx = r["scan"]["indexPath"].as_str().unwrap();
    assert!(idx.ends_with("alt/index.json"), "indexPath was {}", idx);
}

#[test]
fn nested_overrides_flat_when_both_present() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "# flat keys present too, but nested should win\nindex_relative = 'WRONG/index.json'\nfilepaths = [\n  '{}'\n]\n\n[config]\nconfig_version = \"0.9\"\n\n[config.scan]\nindex_path = 'RIGHT/index.json'\nfilepaths = [\n  '{}'\n]\n\n[config.graph]\ndepth = 4\ninclude_bidirectional = true\n",
        base.path().display(),
        base.path().display()
    ))
    .unwrap();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
    let resolved_path = temp.child(".cli-rag/resolved.json");
    let r: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(resolved_path.path()).unwrap()).unwrap();
    let idx = r["scan"]["indexPath"].as_str().unwrap();
    assert!(
        idx.ends_with("RIGHT/index.json"),
        "nested should win: {}",
        idx
    );
    assert_eq!(r["graph"]["depth"].as_i64().unwrap(), 4);
}

#[test]
fn env_cli_rag_filepaths_overrides_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base_cfg = temp.child("notesA");
    base_cfg.create_dir_all().unwrap();
    let base_env = temp.child("notesB");
    base_env.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "[config]\nconfig_version = \"0.1\"\n[config.scan]\nfilepaths = [\n  '{}'\n]\nindex_path = 'idx.json'\n",
        base_cfg.path().display()
    ))
    .unwrap();

    // Run validate with env override
    let assert = Command::cargo_bin("cli-rag")
        .unwrap()
        .env("CLI_RAG_FILEPATHS", base_env.path().display().to_string())
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert();
    assert.success();

    // Resolved should list env base path first and only (since override replaces)
    let resolved_path = temp.child(".cli-rag/resolved.json");
    let r: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(resolved_path.path()).unwrap()).unwrap();
    let fps = r["scan"]["filepaths"].as_array().unwrap();
    assert_eq!(fps.len(), 1);
    assert_eq!(
        fps[0].as_str().unwrap(),
        base_env.path().display().to_string()
    );
}

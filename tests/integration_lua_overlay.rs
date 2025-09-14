use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn overlays_enabled_when_repo_lua_present() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    // Repo overlay next to config
    let overlay = temp.child(".cli-rag.lua");
    overlay.write_str("return {}\n").unwrap();

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // info --format json reflects overlaysEnabled=true
    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("info")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(v["capabilities"]["overlaysEnabled"].as_bool().unwrap());

    // validate writes resolved.json with overlays metadata
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
    let resolved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(resolved_path.path()).unwrap()).unwrap();
    assert!(resolved["overlays"]["enabled"].as_bool().unwrap());
    let repo_path = resolved["overlays"]["repoPath"].as_str().unwrap_or("");
    assert!(repo_path.ends_with(".cli-rag.lua"));

    temp.close().unwrap();
}

#[test]
fn overlays_disabled_with_flag_or_env() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let _overlay = temp.child(".cli-rag.lua");
    _overlay.write_str("return {}\n").unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // Case A: --no-lua disables overlays
    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--no-lua")
        .arg("--config")
        .arg(cfg.path())
        .arg("info")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(!v["capabilities"]["overlaysEnabled"].as_bool().unwrap());

    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--no-lua")
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
    let resolved_path = temp.child(".cli-rag/resolved.json");
    assert!(resolved_path.path().exists());
    let resolved: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(resolved_path.path()).unwrap()).unwrap();
    assert!(!resolved["overlays"]["enabled"].as_bool().unwrap());

    // Case B: env CLI_RAG_NO_LUA=1 disables overlays
    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .env("CLI_RAG_NO_LUA", "1")
        .arg("--config")
        .arg(cfg.path())
        .arg("info")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v2: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(!v2["capabilities"]["overlaysEnabled"].as_bool().unwrap());

    temp.close().unwrap();
}

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn write_adr(file: &assert_fs::fixture::ChildPath, id: &str) {
    let content = format!(
        "---\nid: {id}\ntags: [x]\nstatus: draft\ngroups: [\"G\"]\ndepends_on: []\n---\n\n# {id}: Title\n\nBody\n"
    );
    file.write_str(&content).unwrap();
}

#[test]
fn env_overrides_config_and_cli_overrides_env() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base1 = temp.child("notes1");
    let base2 = temp.child("notes2");
    base1.create_dir_all().unwrap();
    base2.create_dir_all().unwrap();
    write_adr(&base1.child("ADR-001.md"), "ADR-001");
    write_adr(&base2.child("ADR-002.md"), "ADR-002");

    // Config points to base1
    let cfg = temp.child(".adr-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base1.path().display()))
        .unwrap();

    // Case A: env overrides config to base2
    let mut cmd = Command::cargo_bin("adr-rag").unwrap();
    let out = cmd
        .env("ADR_RAG_BASES", base2.path())
        .arg("--config")
        .arg(cfg.path())
        .arg("search")
        .arg("-q")
        .arg("ADR-002")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);
    assert_eq!(v.as_array().unwrap()[0]["id"], "ADR-002");

    // Case B: CLI --base overrides env back to base1
    let mut cmd = Command::cargo_bin("adr-rag").unwrap();
    let out = cmd
        .env("ADR_RAG_BASES", base2.path())
        .arg("--base")
        .arg(base1.path())
        .arg("--config")
        .arg(cfg.path())
        .arg("search")
        .arg("-q")
        .arg("ADR-001")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);
    assert_eq!(v.as_array().unwrap()[0]["id"], "ADR-001");

    temp.close().unwrap();
}

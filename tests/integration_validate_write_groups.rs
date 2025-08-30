use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn write_adr(file: &assert_fs::fixture::ChildPath, id: &str, group: &str) {
    let content = format!(
        "---\nid: {id}\ntags: [x]\nstatus: draft\ngroups: [\"{group}\"]\ndepends_on: []\n---\n\n# {id}: Title\n\nBody\n"
    );
    file.write_str(&content).unwrap();
}

#[test]
fn validate_writes_groups_with_ids() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write_adr(&base.child("ADR-030.md"), "ADR-030", "A");
    write_adr(&base.child("ADR-031.md"), "ADR-031", "B");

    let cfg = temp.child(".adr-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    let mut cmd = Command::cargo_bin("adr-rag").unwrap();
    cmd.arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .arg("--write-groups")
        .assert()
        .success();

    let groups_path = base.path().join("index/semantic-groups.json");
    assert!(groups_path.exists());
    let groups: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(groups_path).unwrap()).unwrap();
    let sections = groups["sections"].as_array().unwrap();
    // Titles include A and B, and ids include ADR-030 and ADR-031
    assert!(sections.iter().any(|s| s["title"] == "A"
        && s["selectors"][0]["anyIds"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == "ADR-030")));
    assert!(sections.iter().any(|s| s["title"] == "B"
        && s["selectors"][0]["anyIds"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == "ADR-031")));

    temp.close().unwrap();
}

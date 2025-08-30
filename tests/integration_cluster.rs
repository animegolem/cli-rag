use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

fn write_adr(file: &assert_fs::fixture::ChildPath, id: &str, title: &str, depends: &[&str]) {
    let deps = if depends.is_empty() {
        String::from("[]")
    } else {
        format!(
            "[{}]",
            depends
                .iter()
                .map(|d| format!("\"{}\"", d))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let content = format!(
        "---\nid: {id}\ntags: [x]\nstatus: draft\ngroups: [\"G\"]\ndepends_on: {deps}\n---\n\n# {id}: {title}\n\nBody\n"
    );
    file.write_str(&content).unwrap();
}

#[test]
fn cluster_json_includes_members() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    write_adr(&base.child("ADR-100.md"), "ADR-100", "Root", &[]);
    write_adr(&base.child("ADR-101.md"), "ADR-101", "Child", &["ADR-100"]);

    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    let out = cmd
        .arg("--config")
        .arg(cfg.path())
        .arg("cluster")
        .arg("--id")
        .arg("ADR-101")
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["root"], "ADR-101");
    assert!(v["members"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["id"] == "ADR-100"));

    temp.close().unwrap();
}

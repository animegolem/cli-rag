use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::process::Command;

#[test]
fn ai_index_plan_writes_schema_compliant_shape() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();

    // Minimal config
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!("bases = [\n  '{}'\n]\n", base.path().display()))
        .unwrap();

    // Create a small connected component of 3 ADRs via depends_on and mentions
    base.child("ADR-001.md").write_str(
        "---\nid: ADR-001\ntags: []\nstatus: draft\ndepends_on: [ADR-002]\n---\n\n# ADR-001\n\n",
    ).unwrap();
    base.child("ADR-002.md").write_str(
        "---\nid: ADR-002\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# ADR-002\nSee [[ADR-003]].\n",
    ).unwrap();
    base.child("ADR-003.md")
        .write_str(
            "---\nid: ADR-003\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# ADR-003\n\n",
        )
        .unwrap();

    // And another pair (IMP schema) to ensure multiple clusters are possible
    base.child("IMP-001.md").write_str(
        "---\nid: IMP-001\ntags: []\nstatus: draft\ndepends_on: [IMP-002]\n---\n\n# IMP-001\n\n",
    ).unwrap();
    base.child("IMP-002.md")
        .write_str(
            "---\nid: IMP-002\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# IMP-002\n\n",
        )
        .unwrap();

    // Validate to produce unified index
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    // Run ai index plan
    let out_path = temp.child("plan.json");
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("ai-index-plan")
        .arg("--min-cluster-size")
        .arg("2")
        .arg("--output")
        .arg(out_path.path())
        .assert()
        .success();

    // Verify plan contents
    let plan: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out_path.path()).unwrap()).unwrap();
    assert_eq!(plan["version"].as_i64().unwrap(), 1);
    assert!(plan["generatedAt"].as_str().unwrap().contains("T"));
    let src = plan["sourceIndexHash"].as_str().unwrap();
    assert!(src.starts_with("sha256:"));
    assert_eq!(plan["params"]["minClusterSize"].as_i64().unwrap(), 2);
    let edges = plan["params"]["edges"].as_array().unwrap();
    assert!(edges.iter().any(|e| e.as_str() == Some("depends_on")));
    let clusters = plan["clusters"].as_array().unwrap();
    assert!(!clusters.is_empty());
    // Basic shape checks on first cluster
    let c0 = &clusters[0];
    let cid = c0["clusterId"].as_str().unwrap();
    assert!(cid.starts_with("c_"));
    let members = c0["members"].as_array().unwrap();
    // Members sorted
    let mut sorted = members
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    let original = sorted.clone();
    sorted.sort();
    assert_eq!(sorted, original);

    temp.close().unwrap();
}

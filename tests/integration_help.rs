use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn cli_help_prints_usage() {
    let mut cmd = Command::cargo_bin("adr-rag").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicates::str::contains("Usage"))
        .stdout(predicates::str::contains("validate"))
        .stdout(predicates::str::contains("doctor"));
}

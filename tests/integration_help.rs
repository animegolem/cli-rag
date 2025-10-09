use assert_cmd::prelude::*;
use predicates::str::contains;
use std::process::Command;

#[test]
fn cli_help_prints_usage() {
    let mut cmd = Command::cargo_bin("cli-rag").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("Usage"))
        .stdout(contains("validate"))
        .stdout(contains("info"))
        .stdout(contains("AI-oriented workflows"));
}

#[test]
fn cli_ai_help_lists_subcommands() {
    Command::cargo_bin("cli-rag")
        .unwrap()
        .args(["ai", "--help"])
        .assert()
        .success()
        .stdout(contains("new"))
        .stdout(contains("index"))
        .stdout(contains("start"))
        .stdout(contains("plan"));
}

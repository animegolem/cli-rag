use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[test]
fn watch_emits_handshake_first() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    // Minimal config
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = \"ADR\"\nfile_patterns = [\"ADR-*.md\"]\nunknown_policy = \"ignore\"\n",
        base.path().display()
    ))
    .unwrap();

    // spawn watch --json with small debounce and dry_run
    let mut child = Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("watch")
        .arg("--debounce-ms")
        .arg("200")
        .arg("--dry-run")
        .arg("--json")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn watch");

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    // read first line (handshake)
    reader.read_line(&mut line).unwrap();
    let v: serde_json::Value = serde_json::from_str(&line).unwrap();
    assert_eq!(v["event"], "watch_start");
    assert_eq!(v["protocolVersion"].as_i64(), Some(1));

    // terminate process and wait to avoid zombie
    let _ = child.kill();
    let _ = child.wait();
    temp.close().unwrap();
}

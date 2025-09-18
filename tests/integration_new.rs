use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use std::path::PathBuf;
use std::process::Command;

fn write_base_cfg(dir: &assert_fs::TempDir, base_rel: &str) -> assert_fs::fixture::ChildPath {
    let cfg = dir.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n",
        dir.child(base_rel).path().display()
    ))
    .unwrap();
    cfg
}

#[test]
fn new_creates_note_from_template() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = write_base_cfg(&temp, "notes");

    // Scaffold schema and body template
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("init")
        .arg("--silent")
        .arg("--schema")
        .arg("ADR")
        .arg("--separate")
        .assert()
        .success();

    // Create note
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--title")
        .arg("Hello World")
        .assert()
        .success();

    // Verify file exists and content replaced
    let f = base.child("ADR-001.md");
    f.assert(predicates::path::exists());
    let s = std::fs::read_to_string(f.path()).unwrap();
    assert!(s.contains("id: ADR-001"));
    assert!(s.contains("# ADR-001: Hello World"));
    drop(cfg);
    temp.close().unwrap();
}

#[test]
fn new_dry_run_does_not_write() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let _cfg = write_base_cfg(&temp, "notes");

    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("IMP")
        .arg("--dry-run")
        .assert()
        .success();

    base.child("IMP-001.md").assert(predicates::path::missing());
    temp.close().unwrap();
}

#[test]
fn new_twice_increments_id() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let _cfg = write_base_cfg(&temp, "notes");

    // First note
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .assert()
        .success();
    // Second note
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .assert()
        .success();

    base.child("ADR-001.md").assert(predicates::path::exists());
    base.child("ADR-002.md").assert(predicates::path::exists());
    temp.close().unwrap();
}

#[test]
fn new_print_body_prints() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let _cfg = write_base_cfg(&temp, "notes");

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let tmpl_src = manifest_dir.join(".cli-rag/templates/ADR.md");
    let tmpl_dir = temp.child(".cli-rag/templates");
    tmpl_dir.create_dir_all().unwrap();
    tmpl_dir
        .child("ADR.md")
        .write_str(&std::fs::read_to_string(tmpl_src).unwrap())
        .unwrap();

    let out = Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--print-body")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(s.contains("id: ADR-001"));
    assert!(s.contains("created_date:"));
    assert!(s.contains("<!-- A concise statement explaining the goal of this decision. -->"));
    assert!(s.contains("## Consequences"));
    assert!(s.contains("## Updates"));
    temp.close().unwrap();
}

#[test]
fn new_filename_template_creates_expected_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let _cfg = write_base_cfg(&temp, "notes");

    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--title")
        .arg("hello")
        .arg("--filename-template")
        .arg("{{id}}-{{title}}.md")
        .assert()
        .success();

    base.child("ADR-001-hello.md")
        .assert(predicates::path::exists());
    temp.close().unwrap();
}

#[test]
fn new_normalize_title_changes_filename_when_template_used() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let _cfg = write_base_cfg(&temp, "notes");

    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--title")
        .arg("hello world")
        .arg("--filename-template")
        .arg("{{id}}-{{title}}.md")
        .arg("--normalize-title")
        .assert()
        .success();

    base.child("ADR-001-Hello World.md")
        .assert(predicates::path::exists());
    temp.close().unwrap();
}

#[test]
fn new_writes_to_specified_dest_base() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base1 = temp.child("notes1");
    let base2 = temp.child("notes2");
    base1.create_dir_all().unwrap();
    base2.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}',\n  '{}'\n]\n",
        base1.path().display(),
        base2.path().display()
    ))
    .unwrap();

    // Write into base2 via --dest-base
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--dest-base")
        .arg(base2.path())
        .assert()
        .success();

    base1
        .child("ADR-001.md")
        .assert(predicates::path::missing());
    base2.child("ADR-001.md").assert(predicates::path::exists());
    temp.close().unwrap();
}

#[test]
fn new_injects_frontmatter_when_token_present() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = write_base_cfg(&temp, "notes");

    // Write a template that uses ((frontmatter)) token
    let tmpl_dir = temp.child(".cli-rag/templates");
    tmpl_dir.create_dir_all().unwrap();
    let tmpl = tmpl_dir.child("ADR.md");
    tmpl.write_str("---\n((frontmatter))\n---\n\n# {{id}}: {{title}}\n")
        .unwrap();

    // Create note
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--title")
        .arg("Injected")
        .assert()
        .success();

    let f = base.child("ADR-001.md");
    f.assert(predicates::path::exists());
    let s = std::fs::read_to_string(f.path()).unwrap();
    assert!(s.contains("id: ADR-001"));
    assert!(s.contains("tags: []"));
    assert!(s.contains("status: draft"));
    assert!(s.contains("depends_on: []"));
    assert!(s.contains("# ADR-001: Injected"));
    drop(cfg);
    temp.close().unwrap();
}
#[test]
fn new_id_generator_increment_with_prefix_and_padding() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = 'RFC'\nfile_patterns = ['RFC-*.md']\n[schema.new]\nid_generator = {{ strategy = 'increment', prefix = 'RFC-', padding = 4 }}\n",
        base.path().display()
    ))
    .unwrap();

    // Create note
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("new")
        .arg("--schema")
        .arg("RFC")
        .arg("--title")
        .arg("Spec One")
        .assert()
        .success();

    base.child("RFC-0001.md").assert(predicates::path::exists());
    // Validate repo stays healthy
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
    temp.close().unwrap();
}

#[test]
fn new_id_generator_datetime_and_uuid() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = 'IMP'\nfile_patterns = ['IMP-*.md']\n[schema.new]\nid_generator = {{ strategy = 'datetime', prefix = 'IMP-' }}\n\n[[schema]]\nname = 'UID'\nfile_patterns = ['UID-*.md']\n[schema.new]\nid_generator = {{ strategy = 'uuid', prefix = 'UID-' }}\n",
        base.path().display()
    ))
    .unwrap();

    // Create datetime note
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("new")
        .arg("--schema")
        .arg("IMP")
        .assert()
        .success();
    // File with IMP-YYYYMMDDHHMMSS.md exists
    let mut found_dt = false;
    for entry in std::fs::read_dir(base.path()).unwrap() {
        let p = entry.unwrap().path();
        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
            if name.starts_with("IMP-")
                && name.ends_with(".md")
                && name.len() >= "IMP-YYYYMMDDHHMMSS.md".len()
            {
                found_dt = true;
            }
        }
    }
    assert!(found_dt, "IMP datetime file not found");

    // Create uuid note
    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("new")
        .arg("--schema")
        .arg("UID")
        .assert()
        .success();
    // File with UID-<uuid>.md exists
    let mut found_uuid = false;
    for entry in std::fs::read_dir(base.path()).unwrap() {
        let p = entry.unwrap().path();
        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
            if name.starts_with("UID-") && name.ends_with(".md") && name.len() > "UID-.md".len() {
                found_uuid = true;
            }
        }
    }
    assert!(found_uuid, "UID uuid file not found");

    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();

    temp.close().unwrap();
}

#[test]
fn new_cli_filename_template_filters_work() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let _cfg = write_base_cfg(&temp, "notes");

    let now = chrono::Local::now().format("%Y-%m").to_string();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--title")
        .arg("Hello Filters")
        .arg("--filename-template")
        .arg("{{now|date:\"%Y-%m\"}}-{{title|snake_case}}.md")
        .assert()
        .success();

    base.child(format!("{}-hello_filters.md", now))
        .assert(predicates::path::exists());
    Command::cargo_bin("cli-rag")
        .unwrap()
        .current_dir(temp.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
    temp.close().unwrap();
}

#[test]
fn new_schema_filename_template_filters() {
    let temp = assert_fs::TempDir::new().unwrap();
    let base = temp.child("notes");
    base.create_dir_all().unwrap();
    let cfg = temp.child(".cli-rag.toml");
    cfg.write_str(&format!(
        "bases = [\n  '{}'\n]\n\n[[schema]]\nname = 'ADR'\nfile_patterns = ['ADR-*.md']\n[schema.new]\nid_generator = {{ strategy = 'increment', prefix = 'ADR-', padding = 3 }}\nfilename_template = \"{{{{schema.name|kebab-case}}}}-{{{{title|PascalCase}}}}-{{{{id}}}}.md\"\n",
        base.path().display()
    ))
    .unwrap();

    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("new")
        .arg("--schema")
        .arg("ADR")
        .arg("--title")
        .arg("hello world")
        .assert()
        .success();

    let names: Vec<String> = std::fs::read_dir(base.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter_map(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect();
    assert!(
        names.iter().any(|n| n == "adr-HelloWorld-ADR-001.md"),
        "expected adr-HelloWorld-ADR-001.md; got {:?}",
        names
    );

    Command::cargo_bin("cli-rag")
        .unwrap()
        .arg("--config")
        .arg(cfg.path())
        .arg("validate")
        .arg("--format")
        .arg("json")
        .assert()
        .success();
    temp.close().unwrap();
}

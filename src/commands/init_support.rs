use anyhow::{anyhow, Context, Result};
use serde_json::json;
use std::fs;
// use std::io; // reserved for future interactive I/O helpers
use std::path::{Path, PathBuf};
use toml::{map::Map as TomlMap, Value as TomlValue};

pub const PROJECT_PRESET_CONFIG: &str = r#"# Repo-local CLI config (cli-rag)

[config]
#: =============================================================================
#:                            # --- Version --- #
#: =============================================================================
#: Sets the config version for all top level rules.
config_version = "0.1"

#: =============================================================================
#:                            # --- SCAN --- #
#: =============================================================================
[config.scan]
#: The root directories that cli-rag will scan for notes.
#: All paths must be relative to the location of this `.cli-rag.toml` file.
filepaths = ["docs/RAG"]
#: By default, an index will be created at `.cli-rag/index.json`.
#: File paths are relative to the location of this `.cli-rag.toml` file.
index_path = ".cli-rag/index.json"
#: `hash_mode` controls how file changes are detected. `mtime` is fast; `content` is exact.
hash_mode = "mtime"
#: `index_strategy` controls what is stored in the index (metadata-only vs full content).
index_strategy = "content"
#: Remove directories or patterns from scanning to improve speed.
ignore_globs = ["**/node_modules/**", "**/dist/**"]
#: Default true; set to false if your repo relies on symlinks.
ignore_symlinks = true

#: =============================================================================
#:                            # --- AUTHORING --- #
#: =============================================================================
[config.authoring]
#: Editor invoked by `cli-rag new` and friends. Falls back to $EDITOR/$VISUAL.
editor = "nvim"
#: When true, `cli-rag watch` will run in the background to keep indexes fresh.
background_watch = true

[config.authoring.destinations]
#: Map schema names to write paths relative to this config (created if missing).
ADR = "docs/RAG/ADR"

#: =============================================================================
#:                             # --- GRAPH --- #
#: =============================================================================
[config.graph]
#: Default depth for graph/path commands. 1 = note + immediate neighbors.
depth = 1
#: Whether to include dependents (backlinks) in traversals. default = true.
include_bidirectional = true

[config.graph.ai]
#: Defaults for `cli-rag ai get` style commands.
depth = 1
#: Maximum neighbors shown per node (metadata mode).
default_fanout = 5
#: Include backlinks when walking neighbors.
include_bidirectional = true
#: Output style for neighbors (metadata|outline|full). Default metadata.
neighbor_style = "metadata"
#: Number of lines per heading when neighbor_style = outline.
outline_lines = 2

#: =============================================================================
#:                        # --- TEMPLATE MANAGEMENT --- #
#: =============================================================================
[config.templates]
#: Load schema definitions from external files. Add more entries as needed.
import = [".cli-rag/templates/ADR.toml"]
"#;

pub const ADR_TEMPLATE: &str =
    include_str!("../../contracts/v1/config/user_config/templates/ADR.toml");

#[derive(Clone, Copy)]
pub enum FileStatus {
    Created,
    Updated,
    Unchanged,
}

pub struct InitOutcome {
    pub preset: &'static str,
    pub created: Vec<String>,
    pub updated: Vec<String>,
    pub warnings: Vec<String>,
    pub cancelled: bool,
    pub dry_run: bool,
}

impl InitOutcome {
    pub fn new(preset: &'static str, dry_run: bool) -> Self {
        Self {
            preset,
            created: vec![],
            updated: vec![],
            warnings: vec![],
            cancelled: false,
            dry_run,
        }
    }
    pub fn record_created(&mut self, path: &Path) {
        let display = path_display(path);
        if !self.created.contains(&display) {
            self.created.push(display);
        }
    }
    pub fn record_updated(&mut self, path: &Path) {
        let display = path_display(path);
        if self.created.contains(&display) {
            return;
        }
        if !self.updated.contains(&display) {
            self.updated.push(display);
        }
    }
    pub fn add_warning<S: Into<String>>(&mut self, warning: S) {
        self.warnings.push(warning.into());
    }
    pub fn to_json(&self) -> serde_json::Value {
        json!({
          "protocolVersion": crate::protocol::PROTOCOL_VERSION,
          "preset": self.preset,
          "dryRun": self.dry_run,
          "created": self.created,
          "updated": self.updated,
          "warnings": self.warnings,
          "cancelled": self.cancelled,
        })
    }
}

pub fn init_project(target: &Path, force: bool, dry_run: bool) -> Result<InitOutcome> {
    let mut outcome = InitOutcome::new("project", dry_run);

    let config_dir = target
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let templates_dir = config_dir.join(".cli-rag").join("templates");
    let template_path = templates_dir.join("ADR.toml");

    let existed = target.exists();
    let mut backup_requested = false;
    if existed && !force {
        // caller must decide overwrite via UI; kept simple here
        backup_requested = true;
    }
    if backup_requested && !dry_run {
        let backup = create_backup(target)?;
        outcome.record_created(&backup);
    } else if backup_requested && dry_run {
        outcome.record_created(&backup_path(target));
    }

    match stage_write(target, PROJECT_PRESET_CONFIG, dry_run)? {
        FileStatus::Created => outcome.record_created(target),
        FileStatus::Updated => outcome.record_updated(target),
        FileStatus::Unchanged => {}
    }
    match stage_write(&template_path, ADR_TEMPLATE, dry_run)? {
        FileStatus::Created => outcome.record_created(&template_path),
        FileStatus::Updated => outcome.record_updated(&template_path),
        FileStatus::Unchanged => {}
    }
    Ok(outcome)
}

pub fn create_backup(path: &Path) -> Result<PathBuf> {
    let backup = backup_path(path);
    if let Some(parent) = backup.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::copy(path, &backup).with_context(|| format!("backing up {:?} to {:?}", path, backup))?;
    Ok(backup)
}
pub fn backup_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|s| s.to_os_string())
        .unwrap_or_default();
    name.push(".bak");
    path.with_file_name(name)
}
pub fn stage_write(path: &Path, contents: &str, dry_run: bool) -> Result<FileStatus> {
    let exists = path.exists();
    if exists {
        if let Ok(existing) = fs::read_to_string(path) {
            if existing == contents {
                return Ok(FileStatus::Unchanged);
            }
        }
    }
    if !dry_run {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating directory {:?}", parent))?;
        }
        fs::write(path, contents).with_context(|| format!("writing preset config {:?}", path))?;
    }
    Ok(if exists {
        FileStatus::Updated
    } else {
        FileStatus::Created
    })
}
pub fn path_display(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

pub fn add_schema(
    cfg_path: &Path,
    name: &str,
    separate: bool,
    force: bool,
    dry_run: bool,
    outcome: &mut InitOutcome,
) -> Result<bool> {
    if separate {
        let (template_status, body_status, config_updated) =
            write_separate_schema(cfg_path, name, force, dry_run)?;
        let cfg_dir = cfg_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        let templates_dir = cfg_dir.join(".cli-rag").join("templates");
        let template_path = templates_dir.join(format!("{}.toml", name));
        let body_path = templates_dir.join(format!("{}.md", name));
        if let FileStatus::Created = template_status {
            outcome.record_created(&template_path)
        } else if let FileStatus::Updated = template_status {
            outcome.record_updated(&template_path)
        }
        if let FileStatus::Created = body_status {
            outcome.record_created(&body_path)
        } else if let FileStatus::Updated = body_status {
            outcome.record_updated(&body_path)
        }
        if config_updated {
            outcome.record_updated(cfg_path);
        }
        Ok(config_updated)
    } else {
        let appended = append_inline_schema(cfg_path, name, dry_run)?;
        if appended {
            outcome.record_updated(cfg_path);
        }
        Ok(appended)
    }
}

pub fn schema_block(name: &str) -> String {
    let upper = name.to_uppercase();
    let pattern = format!("{}-*.md", upper);
    format!("[[schema]]\nname = \"{upper}\"\nfile_patterns = [\"{pattern}\"]\nrequired = [\"id\", \"tags\", \"status\", \"depends_on\"]\nunknown_policy = \"ignore\"\n\n[schema.rules.status]\nallowed = [\n  \"draft\", \"incomplete\", \"proposed\", \"accepted\",\n  \"complete\", \"design\", \"legacy-reference\", \"superseded\"\n]\nseverity = \"error\"\n")
}
pub fn note_stub(_name: &str) -> String {
    "---\nid: {{id}}\ntags: []\nstatus: draft\ndepends_on: []\n---\n\n# {{id}}: {{title}}\n\n"
        .to_string()
}
pub fn append_inline_schema(cfg_path: &Path, name: &str, dry_run: bool) -> Result<bool> {
    let mut content =
        fs::read_to_string(cfg_path).with_context(|| format!("reading config {:?}", cfg_path))?;
    if content.contains(&format!("name = \"{}\"", name)) {
        return Ok(false);
    }
    if !dry_run {
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        content.push_str(&schema_block(name));
        fs::write(cfg_path, content)
            .with_context(|| format!("writing schema '{}' to {:?}", name, cfg_path))?;
    }
    Ok(true)
}
pub fn write_separate_schema(
    cfg_path: &Path,
    name: &str,
    _force: bool,
    dry_run: bool,
) -> Result<(FileStatus, FileStatus, bool)> {
    let cfg_dir = cfg_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let templates_dir = cfg_dir.join(".cli-rag").join("templates");
    let template_path = templates_dir.join(format!("{}.toml", name));
    let body_path = templates_dir.join(format!("{}.md", name));
    let template_status = stage_write(&template_path, &schema_block(name), dry_run)?;
    let body_status = stage_write(&body_path, &note_stub(name), dry_run)?;
    let rel_entry = format!(".cli-rag/templates/{}.toml", name);
    let config_updated = update_config_import(cfg_path, &rel_entry, dry_run)?;
    Ok((template_status, body_status, config_updated))
}
pub fn update_config_import(cfg_path: &Path, entry: &str, dry_run: bool) -> Result<bool> {
    let raw =
        fs::read_to_string(cfg_path).with_context(|| format!("reading config {:?}", cfg_path))?;
    let mut tv: TomlValue =
        toml::from_str(&raw).with_context(|| format!("parsing TOML config {:?}", cfg_path))?;
    let mut changed = false;
    if let Some(config_table) = tv.get_mut("config").and_then(|v| v.as_table_mut()) {
        let templates_value = config_table
            .entry("templates")
            .or_insert_with(|| TomlValue::Table(TomlMap::new()));
        let templates_table = templates_value
            .as_table_mut()
            .ok_or_else(|| anyhow!("config.templates must be a table"))?;
        let import_value = templates_table
            .entry("import")
            .or_insert_with(|| TomlValue::Array(Vec::new()));
        let import_array = import_value
            .as_array_mut()
            .ok_or_else(|| anyhow!("config.templates.import must be an array"))?;
        if !import_array.iter().any(|v| v.as_str() == Some(entry)) {
            import_array.push(TomlValue::String(entry.to_string()));
            changed = true;
        }
    } else {
        let root = tv
            .as_table_mut()
            .ok_or_else(|| anyhow!("config must be a TOML table"))?;
        let import_value = root
            .entry("import")
            .or_insert_with(|| TomlValue::Array(Vec::new()));
        let import_array = import_value
            .as_array_mut()
            .ok_or_else(|| anyhow!("import must be an array"))?;
        if !import_array.iter().any(|v| v.as_str() == Some(entry)) {
            import_array.push(TomlValue::String(entry.to_string()));
            changed = true;
        }
    }
    if changed && !dry_run {
        let rendered = toml::to_string_pretty(&tv)?;
        fs::write(cfg_path, rendered)
            .with_context(|| format!("updating imports in {:?}", cfg_path))?;
    }
    Ok(changed)
}

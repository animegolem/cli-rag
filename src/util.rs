use anyhow::{anyhow, Result};
use std::borrow::Cow;
use std::path::Path;

pub fn try_open_editor(path: &Path) -> Result<()> {
    let editors = vec![
        std::env::var("VISUAL").ok(),
        std::env::var("EDITOR").ok(),
        Some("nano".into()),
        Some("vi".into()),
        Some("vim".into()),
    ];
    for ed in editors.into_iter().flatten() {
        let status = std::process::Command::new(ed.clone()).arg(path).status();
        if let Ok(st) = status {
            if st.success() {
                return Ok(());
            }
        }
    }
    Err(anyhow!("no editor found or editor failed"))
}

/// Convert a filesystem path into a forward-slash string for JSON outputs.
pub fn normalize_display_path<P: AsRef<Path>>(path: P) -> String {
    let raw: Cow<'_, str> = path.as_ref().to_string_lossy();
    if raw.contains('\\') {
        raw.replace('\\', "/")
    } else {
        raw.into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_display_path;

    #[test]
    fn normalize_display_path_converts_backslashes() {
        assert_eq!(
            normalize_display_path("docs\\RAG\\ADR\\ADR-001.md"),
            "docs/RAG/ADR/ADR-001.md"
        );
    }
}

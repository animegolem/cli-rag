use anyhow::{anyhow, Result};
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
        if let Ok(st) = status { if st.success() { return Ok(()); } }
    }
    Err(anyhow!("no editor found or editor failed"))
}


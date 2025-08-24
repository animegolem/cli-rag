use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct FrontMatter {
    pub id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub groups: Option<Vec<String>>, 
    pub depends_on: Option<Vec<String>>, 
    pub supersedes: Option<OneOrMany>,
    pub superseded_by: Option<OneOrMany>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AdrDoc {
    pub file: PathBuf,
    pub id: Option<String>,
    pub title: String,
    pub tags: Vec<String>,
    pub status: Option<String>,
    pub groups: Vec<String>,
    pub depends_on: Vec<String>,
    pub supersedes: Vec<String>,
    pub superseded_by: Vec<String>,
    pub fm: BTreeMap<String, serde_yaml::Value>,
    pub mtime: Option<u64>,
    pub size: Option<u64>,
}

pub fn parse_front_matter_and_title(content: &str, path: &Path) -> AdrDoc {
    let mut fm: FrontMatter = FrontMatter::default();
    let mut fm_map: BTreeMap<String, serde_yaml::Value> = BTreeMap::new();
    let mut body = content;
    // Normalize line endings for delimiter scanning
    let norm = content.replace("\r\n", "\n");
    if norm.starts_with("---\n") || norm.starts_with("+++\n") {
        let delim = if norm.starts_with("---\n") { "---" } else { "+++" };
        let start = 4; // skip delimiter and newline
        // find closing delimiter on its own line
        let needle = format!("\n{}\n", delim);
        let end_opt = norm[start..].find(&needle).map(|i| start + i);
        let (fm_text, body_start) = if let Some(end) = end_opt {
            (&norm[start..end], end + needle.len())
        } else if norm[start..].ends_with(&format!("\n{}", delim)) {
            let end = norm.len() - (delim.len() + 1);
            (&norm[start..end], end + delim.len() + 1)
        } else {
            ("", start)
        };
        if !fm_text.is_empty() {
            if delim == "---" {
                if let Ok(parsed) = serde_yaml::from_str::<FrontMatter>(fm_text) { fm = parsed; }
                if let Ok(mapping) = serde_yaml::from_str::<serde_yaml::Mapping>(fm_text) {
                    for (k, v) in mapping { if let Some(key) = k.as_str() { fm_map.insert(key.to_string(), v); } }
                }
            } else {
                // +++ TOML front matter
                if let Ok(parsed) = toml::from_str::<FrontMatter>(fm_text) { fm = parsed; }
                if let Ok(tval) = toml::from_str::<toml::Value>(fm_text) {
                    if let Some(table) = tval.as_table() {
                        for (k, _v) in table.iter() { fm_map.insert(k.clone(), serde_yaml::Value::Null); }
                    }
                }
            }
            // Map body slice back to original content by position if possible
            let offset = body_start.min(norm.len());
            let tail = &norm[offset..];
            body = tail;
        }
    }
    let title_re = Regex::new(r"(?m)^#\s+(.+)$").unwrap();
    let title = title_re
        .captures(body)
        .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
        .unwrap_or_else(|| path.file_name().unwrap_or_default().to_string_lossy().to_string());

    AdrDoc {
        file: path.to_path_buf(),
        id: fm.id,
        title,
        tags: fm.tags.unwrap_or_default(),
        status: fm.status,
        groups: fm.groups.unwrap_or_default(),
        depends_on: fm.depends_on.unwrap_or_default(),
        supersedes: match fm.supersedes { Some(OneOrMany::One(s)) => vec![s], Some(OneOrMany::Many(v)) => v, None => vec![] },
        superseded_by: match fm.superseded_by { Some(OneOrMany::One(s)) => vec![s], Some(OneOrMany::Many(v)) => v, None => vec![] },
        fm: fm_map,
        mtime: file_mtime(path).ok(),
        size: file_size(path).ok(),
    }
}

pub fn file_mtime(p: &Path) -> Result<u64> {
    let md = fs::metadata(p)?;
    let m = md.modified()?;
    let d = m.duration_since(SystemTime::UNIX_EPOCH).unwrap_or(Duration::from_secs(0));
    Ok(d.as_secs())
}

pub fn file_size(p: &Path) -> Result<u64> {
    let md = fs::metadata(p)?;
    Ok(md.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parse_front_matter_and_title() {
        let md = r#"---
id: ADR-123
tags: [a, b]
status: proposed
groups: ["Tools & Execution"]
depends_on: [ADR-100]
supersedes: ADR-050
---

# ADR-123: Sample

Body here.
"#;
        let path = Path::new("/tmp/ADR-123-sample.md");
        let doc = parse_front_matter_and_title(md, path);
        assert_eq!(doc.id.as_deref(), Some("ADR-123"));
        assert_eq!(doc.title, "ADR-123: Sample");
        assert_eq!(doc.status.as_deref(), Some("proposed"));
        assert_eq!(doc.tags, vec!["a", "b"]);
        assert_eq!(doc.groups, vec!["Tools & Execution".to_string()]);
        assert_eq!(doc.depends_on, vec!["ADR-100".to_string()]);
        assert_eq!(doc.supersedes, vec!["ADR-050".to_string()]);
    }

    #[test]
    fn test_parse_toml_front_matter_and_title() {
        let md = r#"+++
id = "ADR-200"
tags = ["x"]
status = "accepted"
groups = ["G1"]
depends_on = ["ADR-100"]
+++

# ADR-200: TOML Sample

Body here.
"#;
        let path = Path::new("/tmp/ADR-200-sample.md");
        let doc = parse_front_matter_and_title(md, path);
        assert_eq!(doc.id.as_deref(), Some("ADR-200"));
        assert_eq!(doc.title, "ADR-200: TOML Sample");
        assert_eq!(doc.status.as_deref(), Some("accepted"));
        assert_eq!(doc.tags, vec!["x"]);
        assert_eq!(doc.groups, vec!["G1".to_string()]);
        assert_eq!(doc.depends_on, vec!["ADR-100".to_string()]);
    }

    #[test]
    fn test_parse_yaml_crlf_and_no_front_matter() {
        // CRLF front matter + title
        let md_crlf = "---\r\nstatus: proposed\r\n---\r\n\r\n# T\r\n";
        let doc_crlf = parse_front_matter_and_title(md_crlf, Path::new("/t.md"));
        assert_eq!(doc_crlf.status.as_deref(), Some("proposed"));
        assert_eq!(doc_crlf.title, "T");

        // No front matter, title from H1
        let md = "# Hello\nBody";
        let doc = parse_front_matter_and_title(md, Path::new("/hello.md"));
        assert_eq!(doc.id, None);
        assert_eq!(doc.title, "Hello");
    }
}

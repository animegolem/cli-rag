use chrono::Local;
use heck::{ToKebabCase, ToLowerCamelCase, ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use regex::Regex;
use uuid::Uuid;

use crate::config::Config;
use crate::model::AdrDoc;

pub fn render_template(mut s: String, id: &str, title: &str) -> String {
    s = s.replace("{{id}}", id);
    s = s.replace("{{title}}", title);
    let now = Local::now();
    s = s.replace("{{date}}", &now.format("%Y-%m-%d").to_string());
    s = s.replace("{{time}}", &now.format("%H:%M").to_string());
    if let Ok(re) = Regex::new(r"\{\{LOC\|(\d+)\}\}") {
        s = re
            .replace_all(&s, |caps: &regex::Captures| {
                let n: usize = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(0);
                "\n".repeat(n)
            })
            .to_string();
    }
    if s.contains("((frontmatter))") {
        let fm = format!(
            "id: {}\n{}{}{}",
            id, "tags: []\n", "status: draft\n", "depends_on: []\n"
        );
        s = s.replace("((frontmatter))", &fm);
    }
    if s.starts_with("---\n") {
        if let Some(end) = s.find("\n---\n") {
            let fm_content = &s[4..end];
            let rest = &s[end + 5..];
            if let Ok(val) = serde_yaml::from_str::<serde_yaml::Value>(fm_content) {
                use serde_yaml::{Mapping, Value};
                let mut map = match val {
                    Value::Mapping(m) => m,
                    _ => Mapping::new(),
                };
                map.insert(Value::String("id".into()), Value::String(id.into()));
                map.entry(Value::String("tags".into()))
                    .or_insert_with(|| Value::Sequence(vec![]));
                map.entry(Value::String("status".into()))
                    .or_insert_with(|| Value::String("draft".into()));
                map.entry(Value::String("depends_on".into()))
                    .or_insert_with(|| Value::Sequence(vec![]));
                let yaml = serde_yaml::to_string(&Value::Mapping(map)).unwrap_or_default();
                let front = format!("---\n{}---\n", yaml);
                s = format!("{}{}", front, rest);
            }
        }
    }
    s
}

pub fn compute_next_id(prefix: &str, docs: &Vec<AdrDoc>, padding: usize) -> String {
    let re = Regex::new(&format!(r"^{}(\d+)$", regex::escape(prefix))).unwrap();
    let mut max_n: usize = 0;
    for d in docs {
        if let Some(id) = &d.id {
            if let Some(caps) = re.captures(id) {
                if let Some(m) = caps.get(1) {
                    if let Ok(n) = m.as_str().parse::<usize>() {
                        max_n = max_n.max(n);
                    }
                }
            }
        }
    }
    format!("{}{:0width$}", prefix, max_n + 1, width = padding)
}

pub fn generate_initial_id(cfg: &Config, schema: &str, docs: &Vec<AdrDoc>) -> String {
    if let Some(schema_cfg) = cfg.schema.iter().find(|s| s.name == schema) {
        if let Some(new_cfg) = schema_cfg.new.as_ref() {
            if let Some(id_cfg) = new_cfg.id_generator.as_ref() {
                let prefix_default = format!("{}-", schema);
                let prefix = id_cfg.prefix.clone().unwrap_or(prefix_default.clone());
                let padding = id_cfg.padding.unwrap_or(3);
                return match id_cfg.strategy.as_str() {
                    "increment" | "" => compute_next_id(&prefix, docs, padding),
                    "datetime" => {
                        let ts = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
                        format!("{}{}", prefix, ts)
                    }
                    "uuid" => {
                        let u = Uuid::new_v4().simple().to_string();
                        format!("{}{}", prefix, u)
                    }
                    _ => compute_next_id(&prefix, docs, padding),
                };
            }
        }
    }
    compute_next_id(&format!("{}-", schema), docs, 3)
}

fn sanitize_filename_component(s: &str) -> String {
    let out = s.replace('/', "-").replace(['\n', '\r'], " ");
    out.trim().to_string()
}

pub fn render_filename_template(tpl: &str, id: &str, title: &str, schema: &str) -> String {
    let now = Local::now();
    let re = Regex::new(r"\{\{\s*([a-zA-Z0-9_.]+)\s*(?:\|\s*([^}]+))?\s*\}\}").unwrap();
    let rendered = re
        .replace_all(tpl, |caps: &regex::Captures| {
            let var = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let modifier = caps.get(2).map(|m| m.as_str().trim());
            let raw = match var {
                "id" => id.to_string(),
                "title" => title.to_string(),
                "now" => now.format("%Y-%m-%dT%H:%M:%S").to_string(),
                "schema.name" => schema.to_string(),
                _ => String::new(),
            };
            if let Some(mods) = modifier {
                match mods {
                    "kebab-case" => raw.to_kebab_case(),
                    "snake_case" => raw.to_snake_case(),
                    "SCREAMING_SNAKE_CASE" => raw.to_shouty_snake_case(),
                    "camelCase" => raw.to_lower_camel_case(),
                    "PascalCase" => raw.to_upper_camel_case(),
                    m if m.starts_with("date:") => {
                        let pat = m.split_once(':').map(|(_, v)| v).unwrap_or("");
                        let pat = pat.trim().trim_matches('"').trim_matches('\'');
                        now.format(pat).to_string()
                    }
                    _ => raw,
                }
            } else {
                raw
            }
        })
        .to_string();
    let mut f = sanitize_filename_component(&rendered);
    if !f.ends_with(".md") {
        f.push_str(".md");
    }
    f
}

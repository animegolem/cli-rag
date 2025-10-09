use crate::model::AdrDoc;
use globset::{Glob, GlobSetBuilder};
use std::collections::HashMap;

pub fn check_enum_values(
    doc: &AdrDoc,
    field: &str,
    value: &serde_yaml::Value,
    allowed: &[String],
    sev_err: bool,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let values = match value {
        serde_yaml::Value::String(s) => vec![s.to_string()],
        serde_yaml::Value::Sequence(seq) => seq
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    };
    for val in values {
        if !allowed.iter().any(|a| a == &val) {
            push_rule_diagnostic(
                errors,
                warnings,
                sev_err,
                format!(
                    "{}: '{}' value '{}' not in {:?}",
                    doc.file.display(),
                    field,
                    val,
                    allowed
                ),
            );
        }
    }
}

pub fn check_globs(
    doc: &AdrDoc,
    field: &str,
    value: &serde_yaml::Value,
    globs: &[String],
    sev_err: bool,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let mut builder = GlobSetBuilder::new();
    for pattern in globs {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
        }
    }
    let set = match builder.build() {
        Ok(s) => s,
        Err(err) => {
            errors.push(format!(
                "config: could not compile globs for '{}': {}",
                field, err
            ));
            return;
        }
    };
    let values = match value {
        serde_yaml::Value::String(s) => vec![s.to_string()],
        serde_yaml::Value::Sequence(seq) => seq
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    };
    for val in values {
        if !set.is_match(&val) {
            push_rule_diagnostic(
                errors,
                warnings,
                sev_err,
                format!(
                    "{}: '{}' value '{}' does not match glob patterns {:?}",
                    doc.file.display(),
                    field,
                    val,
                    globs
                ),
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn check_numeric_bounds(
    doc: &AdrDoc,
    field: &str,
    value: &serde_yaml::Value,
    min: Option<f64>,
    max: Option<f64>,
    cast_int: bool,
    sev_err: bool,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let numbers = match value {
        serde_yaml::Value::Number(num) => num.as_f64().map(|v| vec![cast_num(v, cast_int)]),
        serde_yaml::Value::String(s) => s
            .trim()
            .parse::<f64>()
            .ok()
            .map(|v| vec![cast_num(v, cast_int)]),
        serde_yaml::Value::Sequence(seq) => Some(
            seq.iter()
                .filter_map(|item| match item {
                    serde_yaml::Value::Number(num) => num.as_f64(),
                    serde_yaml::Value::String(s) => s.trim().parse::<f64>().ok(),
                    _ => None,
                })
                .map(|v| cast_num(v, cast_int))
                .collect(),
        ),
        _ => None,
    };

    let numbers = match numbers {
        Some(nums) if !nums.is_empty() => nums,
        _ => {
            push_rule_diagnostic(
                errors,
                warnings,
                sev_err,
                format!("{}: '{}' must be number", doc.file.display(), field),
            );
            return;
        }
    };

    for value in numbers {
        if let Some(min) = min {
            if value < min {
                push_rule_diagnostic(
                    errors,
                    warnings,
                    sev_err,
                    format!(
                        "{}: '{}' value {} below minimum {}",
                        doc.file.display(),
                        field,
                        value,
                        min
                    ),
                );
            }
        }
        if let Some(max) = max {
            if value > max {
                push_rule_diagnostic(
                    errors,
                    warnings,
                    sev_err,
                    format!(
                        "{}: '{}' value {} above maximum {}",
                        doc.file.display(),
                        field,
                        value,
                        max
                    ),
                );
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn check_reference_types(
    doc: &AdrDoc,
    field: &str,
    value: &serde_yaml::Value,
    allowed_types: &[String],
    doc_schema: &HashMap<String, String>,
    id_to_docs: &HashMap<String, Vec<AdrDoc>>,
    sev_err: bool,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let arr = match value.as_sequence() {
        Some(seq) => seq,
        None => return,
    };
    for v in arr {
        if let Some(dep_id) = v.as_str() {
            if let Some(dep_docs) = id_to_docs.get(dep_id) {
                if let Some(dep_doc) = dep_docs.first() {
                    if let Some(dep_doc_id) = &dep_doc.id {
                        if let Some(dep_type) = doc_schema.get(dep_doc_id) {
                            if !allowed_types.iter().any(|t| t == dep_type) {
                                push_rule_diagnostic(
                                    errors,
                                    warnings,
                                    sev_err,
                                    format!(
                                        "{}: '{}' references {} of type '{}' not in {:?}",
                                        doc.file.display(),
                                        field,
                                        dep_id,
                                        dep_type,
                                        allowed_types
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn cast_num(value: f64, cast_int: bool) -> f64 {
    if cast_int {
        value.trunc()
    } else {
        value
    }
}

pub(crate) fn push_rule_diagnostic(
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
    sev_err: bool,
    message: String,
) {
    if sev_err {
        errors.push(message);
    } else {
        warnings.push(message);
    }
}

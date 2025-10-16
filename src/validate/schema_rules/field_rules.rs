use super::value_checks::{
    check_enum_values, check_globs, check_numeric_bounds, check_reference_types,
    push_rule_diagnostic,
};
use crate::config::SchemaCfg;
use crate::model::AdrDoc;
use std::collections::HashMap;

pub struct FieldRuleContext<'a> {
    pub doc: &'a AdrDoc,
    pub schema_cfg: &'a SchemaCfg,
    pub doc_schema: &'a HashMap<String, String>,
    pub id_to_docs: &'a HashMap<String, Vec<AdrDoc>>,
    pub errors: &'a mut Vec<String>,
    pub warnings: &'a mut Vec<String>,
}

pub fn validate_field_rules(ctx: FieldRuleContext<'_>) {
    for (field, rule) in &ctx.schema_cfg.rules {
        let sev_err = rule.severity.as_deref().unwrap_or("error") == "error";
        let value = match ctx.doc.fm.get(field) {
            Some(v) => v,
            None => continue,
        };

        if let Some(kind) = &rule.r#type {
            enforce_type(
                ctx.doc,
                field,
                value,
                kind,
                sev_err,
                ctx.errors,
                ctx.warnings,
            );
        }

        if let Some(enum_values) = &rule.enum_values {
            check_enum_values(
                ctx.doc,
                field,
                value,
                enum_values,
                sev_err,
                ctx.errors,
                ctx.warnings,
            );
        } else if !rule.allowed.is_empty() {
            check_enum_values(
                ctx.doc,
                field,
                value,
                &rule.allowed,
                sev_err,
                ctx.errors,
                ctx.warnings,
            );
        }

        if let Some(globs) = &rule.globs {
            check_globs(
                ctx.doc,
                field,
                value,
                globs,
                sev_err,
                ctx.errors,
                ctx.warnings,
            );
        }

        if let Some(int_rule) = &rule.integer {
            check_numeric_bounds(
                ctx.doc,
                field,
                value,
                int_rule.min.map(|v| v as f64),
                int_rule.max.map(|v| v as f64),
                true,
                sev_err,
                ctx.errors,
                ctx.warnings,
            );
        }

        if let Some(float_rule) = &rule.float {
            check_numeric_bounds(
                ctx.doc,
                field,
                value,
                float_rule.min,
                float_rule.max,
                false,
                sev_err,
                ctx.errors,
                ctx.warnings,
            );
        }

        if let Some(ref_types) = &rule.refers_to_types {
            check_reference_types(
                ctx.doc,
                field,
                value,
                ref_types,
                ctx.doc_schema,
                ctx.id_to_docs,
                sev_err,
                ctx.errors,
                ctx.warnings,
            );
        }
    }
}

fn enforce_type(
    doc: &AdrDoc,
    field: &str,
    value: &serde_yaml::Value,
    expected: &str,
    sev_err: bool,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    let doc_path = doc.display_path();
    match expected {
        "array" if !value.is_sequence() => push_rule_diagnostic(
            errors,
            warnings,
            sev_err,
            format!("{}: '{}' should be array", doc_path, field),
        ),
        "date" => {
            if let Some(fmt) = value_format(value) {
                if chrono::NaiveDate::parse_from_str(fmt.value, fmt.format).is_err() {
                    push_rule_diagnostic(
                        errors,
                        warnings,
                        sev_err,
                        format!(
                            "{}: '{}' not a valid date '{}', format {}",
                            doc_path,
                            field,
                            fmt.value,
                            fmt.format
                        ),
                    );
                }
            }
        }
        _ => {}
    }
}

struct DateFormat<'a> {
    value: &'a str,
    format: &'a str,
}

fn value_format(value: &serde_yaml::Value) -> Option<DateFormat<'_>> {
    match value {
        serde_yaml::Value::String(s) => Some(DateFormat {
            value: s,
            format: "%Y-%m-%d",
        }),
        _ => None,
    }
}

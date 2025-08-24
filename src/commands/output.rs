use anyhow::Result;
use std::str::FromStr;

pub fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

pub fn print_ndjson_value(value: &serde_json::Value) -> Result<()> {
    println!("{}", serde_json::to_string(value)?);
    Ok(())
}

pub fn print_ndjson_iter<T, I>(iter: I) -> Result<()>
where
    T: serde::Serialize,
    I: IntoIterator<Item = T>,
{
    for item in iter {
        println!("{}", serde_json::to_string(&item)?);
    }
    Ok(())
}

// Optional enum for centralized format selection (scaffold; not yet wired everywhere)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format { Plain, Json, Ndjson }

impl FromStr for Format {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "plain" => Ok(Format::Plain),
            "json" => Ok(Format::Json),
            "ndjson" => Ok(Format::Ndjson),
            _ => Ok(Format::Plain),
        }
    }
}

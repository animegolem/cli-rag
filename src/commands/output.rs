use anyhow::Result;

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

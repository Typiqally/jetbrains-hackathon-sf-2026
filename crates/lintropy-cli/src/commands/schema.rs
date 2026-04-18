//! `lintropy schema` — emit the config JSON schema (§10.1).

use lintropy_core::Config;

use crate::exit::{CliError, EXIT_OK};

pub fn run() -> Result<u8, CliError> {
    let schema = Config::json_schema();
    let rendered = serde_json::to_string_pretty(&schema)
        .map_err(|err| CliError::internal(format!("schema: {err}")))?;
    println!("{rendered}");
    Ok(EXIT_OK)
}

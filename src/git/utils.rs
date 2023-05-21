use std::io::Read;

use anyhow::{Context, Result};

pub fn read_null_terminated_string(reader: &mut impl Read) -> Result<String> {
    let string = String::from_utf8(
        reader
            .bytes()
            .take_while(|byte| byte.as_ref().map(|b| *b != b'\0').unwrap_or(false))
            .collect::<Result<Vec<u8>, _>>()
            .with_context(|| "Failed to read null terminated string as bytes")?,
    )
    .with_context(|| "Failed to parse null terminated string as UTF-8")?;
    Ok(string)
}

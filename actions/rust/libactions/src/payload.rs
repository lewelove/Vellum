use anyhow::Result;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::io::Read;

pub fn read_stdin_string() -> Result<String> {
    let mut stdin_data = String::new();
    std::io::stdin().read_to_string(&mut stdin_data)?;
    Ok(stdin_data)
}

pub fn read_stdin_json() -> Result<Value> {
    let raw = read_stdin_string()?;
    let val: Value = serde_json::from_str(&raw)?;
    Ok(val)
}

pub fn read_stdin_payload<T: DeserializeOwned>() -> Result<T> {
    let raw = read_stdin_string()?;
    let payload: T = serde_json::from_str(&raw)?;
    Ok(payload)
}

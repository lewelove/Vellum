use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Read;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ActionAlbum {
    pub path: PathBuf,
    pub lock: Value,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ActionConfigEnvelope<A = Value> {
    #[serde(default)]
    pub dale: Value,
    #[serde(default)]
    pub action: A,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ActionPayload<A = Value> {
    #[serde(default)]
    pub albums: Vec<ActionAlbum>,
    #[serde(default)]
    pub config: ActionConfigEnvelope<A>,
    #[serde(default)]
    pub options: String,
}

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

pub fn read_stdin_payload<T: serde::de::DeserializeOwned>() -> Result<T> {
    let raw = read_stdin_string()?;
    let payload: T = serde_json::from_str(&raw)?;
    Ok(payload)
}

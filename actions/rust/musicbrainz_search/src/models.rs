use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ActionPayload {
    #[serde(default)]
    pub albums: Vec<serde_json::Value>,
    #[serde(default)]
    pub options: String,
}

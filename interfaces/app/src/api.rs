pub const DEFAULT_SERVER_HTTP_HOST: &str = "http://127.0.0.1:8000";
pub const DEFAULT_SERVER_WS_HOST: &str = "ws://127.0.0.1:8000/ws";

pub async fn fetch_cover_bytes(
    algo: &str,
    size_px: u32,
    hash: &str,
) -> Result<Vec<u8>, reqwest::Error> {
    let url = format!("{DEFAULT_SERVER_HTTP_HOST}/api/covers/{algo}/{size_px}px/{hash}");
    let resp = reqwest::get(&url).await?;
    let bytes = resp.bytes().await?;
    Ok(bytes.to_vec())
}

#[allow(dead_code)]
pub async fn fetch_interface_config(
    name: &str,
) -> Result<serde_json::Value, reqwest::Error> {
    let url = format!("{DEFAULT_SERVER_HTTP_HOST}/api/interfaces/{name}/config");
    let resp = reqwest::get(&url).await?;
    let json_val = resp.json().await?;
    Ok(json_val)
}

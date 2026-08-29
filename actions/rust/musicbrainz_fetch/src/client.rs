use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

const RATE_LIMIT_DELAY: Duration = Duration::from_millis(1050);
const USER_AGENT: &str = "Dale/0.1.0 (https://github.com/lewelove/dale)";
const BROWSE_LIMIT: usize = 100;

pub struct MusicBrainzClient {
    http_client: Client,
}

impl MusicBrainzClient {
    pub fn new() -> Result<Self> {
        let http_client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .context("Failed to build HTTP client")?;
        Ok(Self { http_client })
    }

    pub async fn fetch_release(&self, release_id: &str) -> Result<Value> {
        let url = format!(
            "https://musicbrainz.org/ws/2/release/{release_id}?inc=recordings+artist-credits+labels+discids+isrcs+media+release-groups+genres+tags+ratings+aliases+annotation+url-rels&fmt=json"
        );
        self.fetch_json(&url).await
    }

    pub async fn fetch_release_group(&self, rg_id: &str) -> Result<Value> {
        let url = format!(
            "https://musicbrainz.org/ws/2/release-group/{rg_id}?inc=artists+ratings+genres+tags+aliases+annotation+url-rels&fmt=json"
        );
        self.fetch_json(&url).await
    }

    pub async fn browse_all_releases(&self, rg_id: &str) -> Result<Value> {
        let mut all_releases = Vec::new();
        let mut offset = 0;

        loop {
            let url = format!(
                "https://musicbrainz.org/ws/2/release?release-group={rg_id}&inc=media+recordings+artist-credits+labels+discids+isrcs+release-groups+genres+tags+ratings+aliases&limit={BROWSE_LIMIT}&offset={offset}&fmt=json"
            );

            let page_val = self.fetch_json(&url).await?;
            let total_count = page_val
                .get("release-count")
                .and_then(Value::as_u64)
                .and_then(|c| usize::try_from(c).ok())
                .unwrap_or(0);

            let Some(releases) = page_val.get("releases").and_then(Value::as_array)
            else {
                break;
            };

            if releases.is_empty() {
                break;
            }

            let page_len = releases.len();
            all_releases.extend(releases.clone());
            offset += page_len;

            if offset >= total_count {
                break;
            }

            tokio::time::sleep(RATE_LIMIT_DELAY).await;
        }

        Ok(Value::Array(all_releases))
    }

    pub async fn wait_rate_limit() {
        tokio::time::sleep(RATE_LIMIT_DELAY).await;
    }

    async fn fetch_json(&self, url: &str) -> Result<Value> {
        let resp = self
            .http_client
            .get(url)
            .send()
            .await
            .context(format!("Failed request to {url}"))?;

        let status = resp.status();
        if !status.is_success() {
            anyhow::bail!("MusicBrainz request to {url} returned status {status}");
        }

        let val = resp.json().await.context("Failed to parse JSON response")?;
        Ok(val)
    }
}

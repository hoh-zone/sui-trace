use async_trait::async_trait;
use serde_json::Value;
use trace_common::{Error, error::Result};

use super::Channel;

pub struct DiscordChannel {
    webhook_url: String,
    http: reqwest::Client,
}

impl DiscordChannel {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Channel for DiscordChannel {
    fn kind(&self) -> &'static str {
        "discord"
    }

    async fn send(&self, payload: &Value) -> Result<()> {
        let title = payload
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("sui-trace alert");
        let body = payload.get("body").and_then(|v| v.as_str()).unwrap_or("");
        let link = payload.get("link").and_then(|v| v.as_str());
        let mut content = format!("**{title}**\n{body}");
        if let Some(l) = link {
            content.push_str(&format!("\n{l}"));
        }
        let resp = self
            .http
            .post(&self.webhook_url)
            .json(&serde_json::json!({"content": content}))
            .send()
            .await
            .map_err(|e| Error::Alert(e.to_string()))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Alert(format!("discord {status}: {body}")));
        }
        Ok(())
    }
}

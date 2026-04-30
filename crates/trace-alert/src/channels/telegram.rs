use async_trait::async_trait;
use serde_json::Value;
use trace_common::{Error, error::Result};

use super::Channel;

pub struct TelegramChannel {
    bot_token: String,
    chat_id: String,
    http: reqwest::Client,
}

impl TelegramChannel {
    pub fn new(bot_token: String, chat_id: String) -> Self {
        Self {
            bot_token,
            chat_id,
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Channel for TelegramChannel {
    fn kind(&self) -> &'static str {
        "telegram"
    }

    async fn send(&self, payload: &Value) -> Result<()> {
        if self.bot_token.is_empty() {
            return Err(Error::Alert("telegram bot token not configured".into()));
        }
        let url = format!("https://api.telegram.org/bot{}/sendMessage", self.bot_token);
        let text = format_telegram(payload);
        let resp = self
            .http
            .post(url)
            .json(&serde_json::json!({
                "chat_id": self.chat_id,
                "text": text,
                "parse_mode": "Markdown",
                "disable_web_page_preview": true,
            }))
            .send()
            .await
            .map_err(|e| Error::Alert(e.to_string()))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Alert(format!("telegram {status}: {body}")));
        }
        Ok(())
    }
}

fn format_telegram(payload: &Value) -> String {
    let title = payload
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("sui-trace alert");
    let body = payload
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("(no body)");
    let link = payload.get("link").and_then(|v| v.as_str());
    if let Some(l) = link {
        format!("*{title}*\n{body}\n[Open]({l})")
    } else {
        format!("*{title}*\n{body}")
    }
}

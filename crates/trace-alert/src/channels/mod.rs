//! Notification channels.
//!
//! Each channel implements the same minimal trait. The engine picks channels
//! based on the watchlist's `channels` JSON column, e.g.:
//! ```json
//! [
//!   {"kind": "telegram", "chat_id": "12345"},
//!   {"kind": "webhook", "url": "https://example.com/hook", "secret": "abc"}
//! ]
//! ```

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;
use trace_common::error::Result;

pub mod discord;
pub mod email;
pub mod telegram;
pub mod webhook;

#[async_trait]
pub trait Channel: Send + Sync {
    fn kind(&self) -> &'static str;
    async fn send(&self, payload: &Value) -> Result<()>;
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum ChannelSpec {
    Telegram {
        chat_id: String,
        bot_token: Option<String>,
    },
    Webhook {
        url: String,
        secret: Option<String>,
    },
    Email {
        to: String,
    },
    Discord {
        webhook_url: String,
    },
}

pub fn build(spec: ChannelSpec, defaults: &ChannelDefaults) -> Box<dyn Channel> {
    match spec {
        ChannelSpec::Telegram { chat_id, bot_token } => {
            let token = bot_token.unwrap_or_else(|| defaults.telegram_bot_token.clone());
            Box::new(telegram::TelegramChannel::new(token, chat_id))
        }
        ChannelSpec::Webhook { url, secret } => Box::new(webhook::WebhookChannel::new(url, secret)),
        ChannelSpec::Email { to } => Box::new(email::EmailChannel::new(to, defaults.smtp.clone())),
        ChannelSpec::Discord { webhook_url } => Box::new(discord::DiscordChannel::new(webhook_url)),
    }
}

#[derive(Clone, Default)]
pub struct ChannelDefaults {
    pub telegram_bot_token: String,
    pub smtp: SmtpConfig,
}

#[derive(Clone, Default)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub from: String,
}

use async_trait::async_trait;
use serde_json::Value;
use trace_common::{Error, error::Result};

use super::{Channel, SmtpConfig};

/// Minimal email channel. The first iteration uses Resend's REST API when a
/// `RESEND_API_KEY` env var is present; otherwise it logs a warning and
/// returns an error. SMTP delivery is left as a follow-up so we don't pull
/// the `lettre` dependency before a deployment actually needs it.
pub struct EmailChannel {
    to: String,
    cfg: SmtpConfig,
}

impl EmailChannel {
    pub fn new(to: String, cfg: SmtpConfig) -> Self {
        Self { to, cfg }
    }
}

#[async_trait]
impl Channel for EmailChannel {
    fn kind(&self) -> &'static str {
        "email"
    }

    async fn send(&self, payload: &Value) -> Result<()> {
        if let Ok(api_key) = std::env::var("RESEND_API_KEY") {
            return resend(&api_key, &self.cfg.from, &self.to, payload).await;
        }
        Err(Error::Alert(
            "no email backend configured (set RESEND_API_KEY or wire up SMTP)".into(),
        ))
    }
}

async fn resend(api_key: &str, from: &str, to: &str, payload: &Value) -> Result<()> {
    let subject = payload
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("sui-trace alert");
    let html = payload
        .get("body")
        .and_then(|v| v.as_str())
        .map(|s| format!("<pre>{s}</pre>"))
        .unwrap_or_default();
    let resp = reqwest::Client::new()
        .post("https://api.resend.com/emails")
        .bearer_auth(api_key)
        .json(&serde_json::json!({
            "from": from,
            "to": [to],
            "subject": subject,
            "html": html,
        }))
        .send()
        .await
        .map_err(|e| Error::Alert(e.to_string()))?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(Error::Alert(format!("resend {status}: {body}")));
    }
    Ok(())
}

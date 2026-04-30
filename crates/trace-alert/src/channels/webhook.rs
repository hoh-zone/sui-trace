use async_trait::async_trait;
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use trace_common::{Error, error::Result};

use super::Channel;

pub struct WebhookChannel {
    url: String,
    secret: Option<String>,
    http: reqwest::Client,
}

impl WebhookChannel {
    pub fn new(url: String, secret: Option<String>) -> Self {
        Self {
            url,
            secret,
            http: reqwest::Client::new(),
        }
    }

    fn signature(&self, body: &[u8]) -> Option<String> {
        let secret = self.secret.as_ref()?;
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).ok()?;
        mac.update(body);
        Some(hex::encode(mac.finalize().into_bytes()))
    }
}

#[async_trait]
impl Channel for WebhookChannel {
    fn kind(&self) -> &'static str {
        "webhook"
    }

    async fn send(&self, payload: &Value) -> Result<()> {
        let body = serde_json::to_vec(payload)?;
        let mut req = self
            .http
            .post(&self.url)
            .header("Content-Type", "application/json");
        if let Some(sig) = self.signature(&body) {
            req = req.header("X-SuiTrace-Signature", sig);
        }
        let resp = req
            .body(body)
            .send()
            .await
            .map_err(|e| Error::Alert(e.to_string()))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::Alert(format!("webhook {status}: {body}")));
        }
        Ok(())
    }
}

use anyhow::Result;
use trace_alert::{AlertEngine, channels::ChannelDefaults};
use trace_common::config::AppConfig;
use trace_storage::Db;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    trace_common::telemetry::init("trace-alert");
    let cfg_path = std::env::var("TRACE_CONFIG").unwrap_or_else(|_| "config/default.toml".into());
    let cfg = AppConfig::load(&cfg_path)?;
    let db = Db::connect(&cfg.database).await?;
    let defaults = ChannelDefaults {
        telegram_bot_token: std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default(),
        smtp: trace_alert::channels::SmtpConfig {
            host: std::env::var("SMTP_HOST").unwrap_or_default(),
            port: std::env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587),
            user: std::env::var("SMTP_USER").unwrap_or_default(),
            pass: std::env::var("SMTP_PASS").unwrap_or_default(),
            from: std::env::var("SMTP_FROM").unwrap_or_default(),
        },
    };
    let engine = AlertEngine::new(db, cfg.alert.clone(), defaults);
    engine.run().await?;
    Ok(())
}

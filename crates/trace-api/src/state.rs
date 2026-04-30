use std::sync::Arc;

use tokio::sync::broadcast;
use trace_alert::AlertEngine;
use trace_common::{config::AppConfig, error::Result};
use trace_indexer::client::SuiClient;
use trace_labels::LabelService;
use trace_security::SecurityEngine;
use trace_storage::{Cache, Db};

#[derive(Clone)]
pub struct AppState {
    pub cfg: Arc<AppConfig>,
    pub db: Db,
    pub cache: Cache,
    pub labels: LabelService,
    pub security: SecurityEngine,
    pub alerts: AlertEngine,
    /// Live JSON-RPC client for on-demand enrichment (transaction details,
    /// PTB commands, object changes, etc.). The indexer also has its own
    /// instance — sharing one is fine because the underlying `reqwest::Client`
    /// pools connections.
    pub sui: Arc<SuiClient>,
    /// Broadcast channel powering the `/ws` checkpoint stream. Senders push
    /// new checkpoint summaries here; subscribers fan them out to clients.
    pub events: broadcast::Sender<serde_json::Value>,
}

impl AppState {
    pub async fn new(cfg: AppConfig) -> Result<Self> {
        let db = Db::connect(&cfg.database).await?;
        let cache = Cache::connect(&cfg.redis)?;
        let labels = LabelService::new(db.clone());
        let security = SecurityEngine::new(db.clone());
        let alerts = AlertEngine::new(
            db.clone(),
            cfg.alert.clone(),
            trace_alert::channels::ChannelDefaults::default(),
        );
        let sui = Arc::new(SuiClient::new(&cfg.sui.rest_url));
        let (tx, _) = broadcast::channel(1024);
        Ok(Self {
            cfg: Arc::new(cfg),
            db,
            cache,
            labels,
            security,
            alerts,
            sui,
            events: tx,
        })
    }
}

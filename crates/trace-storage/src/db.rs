use std::time::Duration;

use sqlx::postgres::{PgPool, PgPoolOptions};
use trace_common::{Error, config::DatabaseConfig, error::Result};

#[derive(Clone)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    pub async fn connect(cfg: &DatabaseConfig) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(cfg.max_connections)
            .min_connections(cfg.min_connections)
            .acquire_timeout(Duration::from_secs(cfg.connect_timeout_secs))
            .connect(&cfg.url)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run the embedded sqlx migrations under `migrations/`.
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("../../migrations")
            .run(&self.pool)
            .await
            .map_err(|e| Error::Database(format!("migrate: {e}")))?;
        tracing::info!("database migrations applied");
        Ok(())
    }

    pub async fn health(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| Error::Database(e.to_string()))
    }
}

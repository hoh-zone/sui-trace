use deadpool_redis::{Config as PoolConfig, Pool, Runtime};
use redis::AsyncCommands;
use trace_common::{Error, config::RedisConfig, error::Result};

#[derive(Clone)]
pub struct Cache {
    pool: Pool,
}

impl Cache {
    pub fn connect(cfg: &RedisConfig) -> Result<Self> {
        let pool_cfg = PoolConfig::from_url(&cfg.url);
        let pool = pool_cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| Error::Internal(format!("redis pool: {e}")))?;
        Ok(Self { pool })
    }

    pub fn pool(&self) -> &Pool {
        &self.pool
    }

    pub async fn set_json<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: Option<u64>,
    ) -> Result<()> {
        let payload = serde_json::to_vec(value)?;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        if let Some(ttl) = ttl_secs {
            let _: () = conn
                .set_ex(key, payload, ttl)
                .await
                .map_err(|e| Error::Internal(e.to_string()))?;
        } else {
            let _: () = conn
                .set(key, payload)
                .await
                .map_err(|e| Error::Internal(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let payload: Option<Vec<u8>> = conn
            .get(key)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        match payload {
            Some(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
            None => Ok(None),
        }
    }

    pub async fn publish<T: serde::Serialize>(&self, channel: &str, message: &T) -> Result<()> {
        let payload = serde_json::to_vec(message)?;
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let _: () = conn
            .publish(channel, payload)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        Ok(())
    }
}

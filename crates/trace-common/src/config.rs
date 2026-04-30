use std::path::Path;

use serde::Deserialize;

use crate::error::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub app: AppSection,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub sui: SuiConfig,
    pub api: ApiConfig,
    pub auth: AuthConfig,
    pub clickhouse: ClickhouseConfig,
    pub s3: S3Config,
    pub defillama: DefiLlamaConfig,
    pub alert: AlertConfig,
    pub indexer: IndexerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppSection {
    pub name: String,
    pub env: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_conns")]
    pub max_connections: u32,
    #[serde(default = "default_min_conns")]
    pub min_connections: u32,
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,
}

fn default_max_conns() -> u32 {
    20
}
fn default_min_conns() -> u32 {
    2
}
fn default_connect_timeout() -> u64 {
    10
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default = "default_redis_pool")]
    pub pool_size: usize,
}

fn default_redis_pool() -> usize {
    16
}

#[derive(Debug, Clone, Deserialize)]
pub struct SuiConfig {
    pub grpc_url: String,
    pub rest_url: String,
    pub network: String,
    #[serde(default)]
    pub start_checkpoint: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    pub bind: String,
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    #[serde(default = "default_jwt_ttl")]
    pub jwt_ttl_secs: u64,
    /// Optional shared secret for headless ingestion endpoints (e.g. the
    /// external decompiler pushing module sources). When set, requests with
    /// header `X-Trace-Ingest-Key: <value>` are accepted without a JWT.
    #[serde(default)]
    pub ingest_api_key: Option<String>,
}

fn default_jwt_ttl() -> u64 {
    3600
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClickhouseConfig {
    pub url: String,
    pub database: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    #[serde(default = "default_s3_region")]
    pub region: String,
}

fn default_s3_region() -> String {
    "us-east-1".into()
}

#[derive(Debug, Clone, Deserialize)]
pub struct DefiLlamaConfig {
    pub base: String,
    #[serde(default = "default_defillama_interval")]
    pub poll_interval_secs: u64,
}

fn default_defillama_interval() -> u64 {
    300
}

#[derive(Debug, Clone, Deserialize)]
pub struct AlertConfig {
    #[serde(default = "default_dedupe_window")]
    pub dedupe_window_secs: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_dedupe_window() -> u64 {
    300
}
fn default_max_retries() -> u32 {
    3
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndexerConfig {
    #[serde(default)]
    pub concurrent_pipelines: Vec<String>,
    #[serde(default)]
    pub sequential_pipelines: Vec<String>,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_batch_size() -> usize {
    200
}

impl AppConfig {
    /// Load config from a TOML file, layered with environment variables prefixed `TRACE_`.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let _ = dotenvy::dotenv();
        let builder = config::Config::builder()
            .add_source(config::File::from(path.as_ref()).required(false))
            .add_source(config::Environment::with_prefix("TRACE").separator("__"));

        let cfg = builder
            .build()
            .map_err(|e| Error::Config(e.to_string()))?
            .try_deserialize::<AppConfig>()
            .map_err(|e| Error::Config(e.to_string()))?;
        Ok(cfg)
    }

    pub fn load_default() -> Result<Self> {
        Self::load("config/default.toml")
    }
}

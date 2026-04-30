//! Cross-crate domain models. These are the canonical representations stored
//! in Postgres and exposed through the API. They intentionally do not depend
//! on the `sui-sdk` types so the API surface stays decoupled from the
//! upstream rapid-release cadence.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type Address = String;
pub type Digest = String;
pub type ObjectId = String;
pub type PackageId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Network {
    Mainnet,
    Testnet,
    Devnet,
}

impl Network {
    pub fn as_str(&self) -> &'static str {
        match self {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
            Network::Devnet => "devnet",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub sequence_number: u64,
    pub digest: Digest,
    pub timestamp_ms: i64,
    pub previous_digest: Option<Digest>,
    pub network_total_transactions: u64,
    pub epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub digest: Digest,
    pub checkpoint_seq: u64,
    pub timestamp: DateTime<Utc>,
    pub sender: Address,
    pub status: TxStatus,
    pub gas_used: u64,
    pub gas_price: u64,
    pub kind: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TxStatus {
    Success,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub tx_digest: Digest,
    pub event_seq: u32,
    pub package_id: PackageId,
    pub module: String,
    pub event_type: String,
    pub sender: Address,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: PackageId,
    pub original_id: PackageId,
    pub version: u64,
    pub publisher: Address,
    pub modules_count: u32,
    pub source_verified: bool,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageModule {
    pub package_id: PackageId,
    pub module_name: String,
    pub bytecode_hash: String,
    pub abi_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectSnapshot {
    pub object_id: ObjectId,
    pub version: u64,
    pub object_type: String,
    pub owner: Option<String>,
    pub contents: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    pub tx_digest: Digest,
    pub owner: Address,
    pub coin_type: String,
    pub amount: bigdecimal::BigDecimal,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn weight(&self) -> f32 {
        match self {
            Severity::Info => 0.5,
            Severity::Low => 1.0,
            Severity::Medium => 3.0,
            Severity::High => 7.0,
            Severity::Critical => 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub rule_id: String,
    pub rule_name: String,
    pub severity: Severity,
    pub confidence: f32,
    pub module: String,
    pub function: Option<String>,
    pub location: String,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub package_id: PackageId,
    pub version: u64,
    pub score: f32,
    pub max_severity: Severity,
    pub findings: Vec<SecurityFinding>,
    pub scanned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressLabel {
    pub address: Address,
    pub label: String,
    pub category: LabelCategory,
    pub source: LabelSource,
    pub confidence: f32,
    pub evidence_url: Option<String>,
    pub verified: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LabelCategory {
    Exchange,
    CexHotwallet,
    CexColdwallet,
    MarketMaker,
    VcFund,
    ProtocolTreasury,
    Bridge,
    Validator,
    Hacker,
    Scam,
    Phishing,
    Mixer,
    Sanctioned,
    RugPull,
    TeamMultisig,
    Vesting,
    AirdropDistributor,
    Other,
}

impl LabelCategory {
    pub fn is_risky(&self) -> bool {
        matches!(
            self,
            LabelCategory::Hacker
                | LabelCategory::Scam
                | LabelCategory::Phishing
                | LabelCategory::Mixer
                | LabelCategory::Sanctioned
                | LabelCategory::RugPull
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LabelSource {
    Official,
    Community,
    Oracle,
    Heuristic,
    Imported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvlPoint {
    pub protocol_id: String,
    pub timestamp: DateTime<Utc>,
    pub tvl_usd: f64,
    pub breakdown: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub id: uuid::Uuid,
    pub user_id: Option<uuid::Uuid>,
    pub watchlist_id: Option<uuid::Uuid>,
    pub rule_id: String,
    pub fired_at: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub delivered: bool,
}

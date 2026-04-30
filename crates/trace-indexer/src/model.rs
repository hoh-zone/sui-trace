//! Wire types representing one indexed checkpoint, decoupled from the
//! upstream Sui SDK so the pipelines stay testable.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointBundle {
    pub checkpoint: CheckpointHeader,
    pub transactions: Vec<TxEnvelope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointHeader {
    pub sequence_number: u64,
    pub digest: String,
    pub timestamp_ms: i64,
    pub previous_digest: Option<String>,
    pub network_total_transactions: u64,
    pub epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxEnvelope {
    pub digest: String,
    pub sender: String,
    pub status: String,
    pub gas_used: u64,
    pub gas_price: u64,
    pub kind: String,
    pub events: Vec<RawEvent>,
    pub balance_changes: Vec<RawBalanceChange>,
    pub published_packages: Vec<RawPackage>,
    pub mutated_objects: Vec<RawObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvent {
    pub seq: u32,
    pub package_id: String,
    pub module: String,
    pub event_type: String,
    pub sender: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawBalanceChange {
    pub owner: String,
    pub coin_type: String,
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPackage {
    pub id: String,
    pub original_id: String,
    pub version: u64,
    pub publisher: String,
    pub modules: Vec<RawModule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawModule {
    pub name: String,
    /// Hex-encoded module bytecode.
    pub bytecode_hex: String,
    /// Optional ABI; when missing the security worker derives it from bytecode.
    pub abi: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawObject {
    pub object_id: String,
    pub version: u64,
    pub object_type: String,
    pub owner: Option<String>,
    pub contents: serde_json::Value,
}

//! Thin Sui JSON-RPC client. Backed by `reqwest`.
//!
//! Only the subset of endpoints needed by the M1 pipelines is implemented:
//!   * `sui_getLatestCheckpointSequenceNumber`
//!   * `sui_getCheckpoint`
//!
//! For richer extraction (events, balance changes, packages, mutated
//! objects) we issue a `sui_multiGetTransactionBlocks` call per checkpoint
//! batch with the show flags set. The response is normalised into the local
//! `CheckpointBundle` type to keep downstream code simple.

use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use trace_common::{Error, error::Result};

use crate::model::{
    CheckpointBundle, CheckpointHeader, RawBalanceChange, RawEvent, RawModule, RawObject,
    RawPackage, TxEnvelope,
};

#[derive(Clone)]
pub struct SuiClient {
    http: Client,
    rpc_url: String,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    #[serde(default)]
    jsonrpc: String,
    #[serde(default)]
    id: u64,
    #[serde(default = "default_none")]
    result: Option<T>,
    #[serde(default)]
    error: Option<RpcError>,
}

fn default_none<T>() -> Option<T> {
    None
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

impl SuiClient {
    pub fn new(rpc_url: impl Into<String>) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(8)
            .build()
            .expect("reqwest client");
        Self {
            http,
            rpc_url: rpc_url.into(),
        }
    }

    async fn rpc<T: serde::de::DeserializeOwned>(&self, method: &str, params: Value) -> Result<T> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params
        });
        let resp = self
            .http
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;
        let text = resp.text().await.map_err(|e| Error::Http(e.to_string()))?;
        let parsed: RpcResponse<T> = serde_json::from_str(&text)
            .map_err(|e| Error::Http(format!("decode {method}: {e}: {text}")))?;
        if let Some(err) = parsed.error {
            return Err(Error::Http(format!(
                "rpc {method}: {} {}",
                err.code, err.message
            )));
        }
        let _ = parsed.jsonrpc;
        let _ = parsed.id;
        parsed
            .result
            .ok_or_else(|| Error::Http(format!("rpc {method} returned empty result")))
    }

    /// Fetch a single transaction with the full set of show flags. The
    /// returned `Value` is the raw `sui_getTransactionBlock` response;
    /// callers (the API layer) pass it through to the frontend so the
    /// SuiVision-style detail page can render PTB commands, object
    /// changes, signatures, gas details, etc.
    pub async fn get_transaction_block(&self, digest: &str) -> Result<Value> {
        let opts = json!({
            "showInput": true,
            "showRawInput": true,
            "showEffects": true,
            "showEvents": true,
            "showBalanceChanges": true,
            "showObjectChanges": true,
        });
        self.rpc("sui_getTransactionBlock", json!([digest, opts]))
            .await
    }

    pub async fn latest_checkpoint(&self) -> Result<u64> {
        let s: String = self
            .rpc("sui_getLatestCheckpointSequenceNumber", json!([]))
            .await?;
        s.parse::<u64>()
            .map_err(|e| Error::Indexer(format!("parse latest checkpoint: {e}")))
    }

    /// Pull a checkpoint together with the transactions it contains.
    /// Returns `None` if the checkpoint is not yet available on the node.
    pub async fn get_checkpoint(&self, seq: u64) -> Result<Option<CheckpointBundle>> {
        let raw: Value = match self
            .rpc::<Value>("sui_getCheckpoint", json!([seq.to_string()]))
            .await
        {
            Ok(v) => v,
            Err(Error::Http(msg)) if msg.contains("not found") || msg.contains("-32602") => {
                return Ok(None);
            }
            Err(e) => return Err(e),
        };

        let header = parse_header(&raw)?;
        let tx_digests: Vec<String> = raw
            .get("transactions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|d| d.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let transactions = if tx_digests.is_empty() {
            Vec::new()
        } else {
            self.fetch_transactions(&tx_digests).await?
        };

        Ok(Some(CheckpointBundle {
            checkpoint: header,
            transactions,
        }))
    }

    async fn fetch_transactions(&self, digests: &[String]) -> Result<Vec<TxEnvelope>> {
        let opts = json!({
            "showInput": true,
            "showRawInput": false,
            "showEffects": true,
            "showEvents": true,
            "showBalanceChanges": true,
            "showObjectChanges": true,
        });

        let mut out = Vec::with_capacity(digests.len());
        for chunk in digests.chunks(50) {
            let res: Vec<Value> = self
                .rpc::<Vec<Value>>("sui_multiGetTransactionBlocks", json!([chunk, opts]))
                .await?;
            for v in res {
                out.push(parse_tx(&v)?);
            }
        }
        Ok(out)
    }
}

fn parse_header(raw: &Value) -> Result<CheckpointHeader> {
    Ok(CheckpointHeader {
        sequence_number: raw
            .get("sequenceNumber")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| Error::Indexer("missing sequenceNumber".into()))?,
        digest: raw
            .get("digest")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Indexer("missing digest".into()))?
            .to_string(),
        timestamp_ms: raw
            .get("timestampMs")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or_default(),
        previous_digest: raw
            .get("previousDigest")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        network_total_transactions: raw
            .get("networkTotalTransactions")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or_default(),
        epoch: raw
            .get("epoch")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or_default(),
    })
}

fn parse_tx(v: &Value) -> Result<TxEnvelope> {
    let digest = v
        .get("digest")
        .and_then(|x| x.as_str())
        .ok_or_else(|| Error::Indexer("tx missing digest".into()))?
        .to_string();
    let sender = v
        .get("transaction")
        .and_then(|t| t.get("data"))
        .and_then(|d| d.get("sender"))
        .and_then(|s| s.as_str())
        .unwrap_or("0x0")
        .to_string();
    let status = v
        .get("effects")
        .and_then(|e| e.get("status"))
        .and_then(|s| s.get("status"))
        .and_then(|s| s.as_str())
        .unwrap_or("failure")
        .to_string();
    let kind = v
        .get("transaction")
        .and_then(|t| t.get("data"))
        .and_then(|d| d.get("transaction"))
        .and_then(|t| t.get("kind"))
        .and_then(|k| k.as_str())
        .unwrap_or("unknown")
        .to_string();
    let gas_used = v
        .get("effects")
        .and_then(|e| e.get("gasUsed"))
        .and_then(|g| g.get("computationCost"))
        .and_then(|s| s.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0u64);
    let gas_price = v
        .get("transaction")
        .and_then(|t| t.get("data"))
        .and_then(|d| d.get("gasData"))
        .and_then(|g| g.get("price"))
        .and_then(|s| s.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0u64);

    let events = v
        .get("events")
        .and_then(|e| e.as_array())
        .map(|arr| {
            arr.iter()
                .enumerate()
                .map(|(i, ev)| RawEvent {
                    seq: i as u32,
                    package_id: ev
                        .get("packageId")
                        .and_then(|x| x.as_str())
                        .unwrap_or("0x0")
                        .to_string(),
                    module: ev
                        .get("transactionModule")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string(),
                    event_type: ev
                        .get("type")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string(),
                    sender: ev
                        .get("sender")
                        .and_then(|x| x.as_str())
                        .unwrap_or(&sender)
                        .to_string(),
                    payload: ev.get("parsedJson").cloned().unwrap_or(Value::Null),
                })
                .collect()
        })
        .unwrap_or_default();

    let balance_changes = v
        .get("balanceChanges")
        .and_then(|b| b.as_array())
        .map(|arr| {
            arr.iter()
                .map(|c| RawBalanceChange {
                    owner: extract_owner(c.get("owner")),
                    coin_type: c
                        .get("coinType")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string(),
                    amount: c
                        .get("amount")
                        .and_then(|x| x.as_str())
                        .unwrap_or("0")
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default();

    let mut published_packages = Vec::new();
    let mut mutated_objects = Vec::new();
    if let Some(arr) = v.get("objectChanges").and_then(|x| x.as_array()) {
        for change in arr {
            let kind = change.get("type").and_then(|x| x.as_str()).unwrap_or("");
            match kind {
                "published" => {
                    let id = change
                        .get("packageId")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    let modules = change
                        .get("modules")
                        .and_then(|m| m.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|m| m.as_str())
                                .map(|name| RawModule {
                                    name: name.to_string(),
                                    bytecode_hex: String::new(),
                                    abi: Value::Null,
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    let version = change
                        .get("version")
                        .and_then(|x| x.as_str())
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1u64);
                    published_packages.push(RawPackage {
                        id: id.clone(),
                        original_id: id,
                        version,
                        publisher: sender.clone(),
                        modules,
                    });
                }
                _ => {
                    if let Some(obj_id) = change.get("objectId").and_then(|x| x.as_str()) {
                        mutated_objects.push(RawObject {
                            object_id: obj_id.to_string(),
                            version: change
                                .get("version")
                                .and_then(|x| x.as_str())
                                .and_then(|s| s.parse().ok())
                                .unwrap_or(0u64),
                            object_type: change
                                .get("objectType")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string(),
                            owner: change.get("owner").map(extract_owner_value),
                            contents: change.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(TxEnvelope {
        digest,
        sender,
        status,
        gas_used,
        gas_price,
        kind,
        events,
        balance_changes,
        published_packages,
        mutated_objects,
    })
}

fn extract_owner(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Object(map)) => map
            .get("AddressOwner")
            .or_else(|| map.get("ObjectOwner"))
            .and_then(|x| x.as_str())
            .unwrap_or("shared")
            .to_string(),
        _ => "shared".into(),
    }
}

fn extract_owner_value(v: &Value) -> String {
    extract_owner(Some(v))
}

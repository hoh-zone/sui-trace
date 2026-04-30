//! Background worker that listens for `package_published` events on Redis,
//! pulls the module set from Sui's RPC and runs the engine.

use std::time::Duration;

use redis::AsyncCommands;
use serde::Deserialize;
use serde_json::Value;
use trace_common::{Error, error::Result};
use trace_storage::{Cache, Db};

use crate::context::ModuleContext;
use crate::engine::SecurityEngine;

#[derive(Debug, Deserialize)]
struct PackageMsg {
    package_id: String,
    version: u64,
}

pub struct SecurityWorker {
    db: Db,
    cache: Cache,
    rpc: String,
    engine: SecurityEngine,
}

impl SecurityWorker {
    pub fn new(db: Db, cache: Cache, rpc: String) -> Self {
        let engine = SecurityEngine::new(db.clone());
        Self {
            db,
            cache,
            rpc,
            engine,
        }
    }

    pub async fn run(self) -> Result<()> {
        tracing::info!(rules = self.engine.rule_count(), "security worker started");
        loop {
            match self.tick().await {
                Ok(0) => tokio::time::sleep(Duration::from_secs(2)).await,
                Ok(_) => {}
                Err(e) => {
                    tracing::error!(?e, "security worker error, backing off");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn tick(&self) -> Result<usize> {
        let mut conn = self
            .cache
            .pool()
            .get()
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let job: Option<Vec<String>> = conn
            .brpop("trace:packages:to_scan", 1f64)
            .await
            .map_err(|e| Error::Internal(e.to_string()))?;
        let Some(values) = job else { return Ok(0) };
        // BRPOP returns ["queue", "value"].
        let raw = values.get(1).cloned().unwrap_or_default();
        if raw.is_empty() {
            return Ok(0);
        }
        let msg: PackageMsg = serde_json::from_str(&raw)?;
        let modules = self.fetch_modules(&msg.package_id, msg.version).await?;
        let report = self
            .engine
            .analyse_and_save(&msg.package_id, msg.version, &modules)
            .await?;
        tracing::info!(
            package = %msg.package_id,
            version = msg.version,
            findings = report.findings.len(),
            score = report.score,
            "security report saved"
        );
        let _ = self.db.health().await;
        Ok(1)
    }

    /// Pull the normalized module map for a package from a Sui RPC node and
    /// translate it into the engine's `ModuleContext` shape. Best-effort —
    /// any module the RPC fails to return is simply skipped.
    async fn fetch_modules(&self, package_id: &str, version: u64) -> Result<Vec<ModuleContext>> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sui_getNormalizedMoveModulesByPackage",
            "params": [package_id]
        });
        let resp: Value = reqwest::Client::new()
            .post(&self.rpc)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Http(e.to_string()))?
            .json()
            .await
            .map_err(|e| Error::Http(e.to_string()))?;

        let result = resp
            .get("result")
            .ok_or_else(|| Error::Security("rpc returned no result".into()))?
            .as_object()
            .ok_or_else(|| Error::Security("modules result is not an object".into()))?;

        let mut out = Vec::with_capacity(result.len());
        for (mod_name, raw) in result {
            out.push(crate::context::ModuleContext {
                package_id: package_id.to_string(),
                version,
                name: mod_name.clone(),
                functions: parse_functions(raw),
                structs: parse_structs(raw),
                friends: raw
                    .get("friends")
                    .and_then(|v| v.as_array())
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_object())
                            .map(|o| {
                                format!(
                                    "{}::{}",
                                    o.get("address").and_then(|v| v.as_str()).unwrap_or(""),
                                    o.get("name").and_then(|v| v.as_str()).unwrap_or(""),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                bytecode_hex: None,
            });
        }
        Ok(out)
    }
}

fn parse_functions(raw: &Value) -> Vec<crate::context::FunctionInfo> {
    let mut out = Vec::new();
    let Some(funcs) = raw.get("exposedFunctions").and_then(|v| v.as_object()) else {
        return out;
    };
    for (name, info) in funcs {
        let visibility = match info.get("visibility").and_then(|v| v.as_str()) {
            Some("Public") => crate::context::Visibility::Public,
            Some("Friend") => crate::context::Visibility::PublicFriend,
            Some("Private") => crate::context::Visibility::Private,
            _ => crate::context::Visibility::Private,
        };
        let is_entry = info
            .get("isEntry")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let parameters = info
            .get("parameters")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|p| crate::context::TypeRef {
                        type_name: p.to_string(),
                        generic: false,
                        is_mut_ref: p.to_string().contains("MutableReference"),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let return_types = info
            .get("return")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|p| crate::context::TypeRef {
                        type_name: p.to_string(),
                        generic: false,
                        is_mut_ref: false,
                    })
                    .collect()
            })
            .unwrap_or_default();
        out.push(crate::context::FunctionInfo {
            name: name.clone(),
            visibility,
            is_entry,
            parameters,
            return_types,
            callees: Vec::new(),
            tags: Vec::new(),
        });
    }
    out
}

fn parse_structs(raw: &Value) -> Vec<crate::context::StructInfo> {
    let mut out = Vec::new();
    let Some(structs) = raw.get("structs").and_then(|v| v.as_object()) else {
        return out;
    };
    for (name, info) in structs {
        let abilities: Vec<String> = info
            .get("abilities")
            .and_then(|v| v.get("abilities"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|a| a.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let fields = info
            .get("fields")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|f| {
                        let n = f.get("name")?.as_str()?.to_string();
                        let t = f.get("type_")?.to_string();
                        Some((
                            n,
                            crate::context::TypeRef {
                                type_name: t,
                                generic: false,
                                is_mut_ref: false,
                            },
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let is_capability = name.ends_with("Cap")
            || name.ends_with("Capability")
            || abilities
                .iter()
                .any(|a| a == "key" && (name.contains("Cap")));
        out.push(crate::context::StructInfo {
            name: name.clone(),
            abilities,
            fields,
            is_capability,
        });
    }
    out
}

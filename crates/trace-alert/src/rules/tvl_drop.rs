use serde_json::json;
use trace_common::error::Result;
use trace_storage::Db;
use trace_storage::repo::tvl::TvlRepo;

/// Returns `Some(payload)` if the protocol's TVL fell by more than `threshold_pct`
/// within `window_secs`. Caller is responsible for dedup + delivery.
pub async fn tvl_drop(
    db: &Db,
    protocol_id: &str,
    window_secs: i64,
    threshold_pct: f64,
) -> Result<Option<serde_json::Value>> {
    let drop = TvlRepo::new(db)
        .recent_drop_pct(protocol_id, window_secs)
        .await?;
    if let Some(d) = drop
        && d >= threshold_pct
    {
        return Ok(Some(json!({
            "title": format!("TVL drop detected: {protocol_id}"),
            "body": format!("TVL fell by {d:.2}% in the last {window_secs} seconds."),
            "rule": "tvl_drop",
            "protocol_id": protocol_id,
            "drop_pct": d,
            "window_secs": window_secs,
        })));
    }
    Ok(None)
}

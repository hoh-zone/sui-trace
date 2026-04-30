use serde_json::json;
use sqlx::Row;
use trace_common::{Error, error::Result};
use trace_storage::Db;

type PgRow = sqlx::postgres::PgRow;

/// User-defined watchlist rule: trigger an alert any time the watched
/// address has activity (sender or recipient) in the last `window_secs`.
pub async fn address_activity(
    db: &Db,
    address: &str,
    window_secs: i64,
) -> Result<Vec<serde_json::Value>> {
    let rows: Vec<PgRow> = sqlx::query(
        r#"SELECT digest, sender, status, gas_used, timestamp
           FROM transactions
           WHERE sender = $1
             AND timestamp >= NOW() - ($2 || ' seconds')::interval
           ORDER BY timestamp DESC LIMIT 50"#,
    )
    .bind(address)
    .bind(window_secs)
    .fetch_all(db.pool())
    .await
    .map_err(|e: sqlx::Error| Error::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r: PgRow| {
            let digest: String = r.get(0);
            let sender: String = r.get(1);
            let status: String = r.get(2);
            json!({
                "title": format!("Watchlist activity: {sender}"),
                "body": format!("{sender} sent tx {digest} (status: {status})"),
                "rule": "address_activity",
                "address": sender,
                "tx_digest": digest,
                "status": status,
            })
        })
        .collect())
}

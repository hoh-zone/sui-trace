use serde_json::json;
use sqlx::Row;
use trace_common::{Error, error::Result};
use trace_storage::Db;

type PgRow = sqlx::postgres::PgRow;

/// Detect transfers landing on a tagged risky address (hacker / mixer /
/// sanctioned) within `window_secs`. Useful for after-the-fact attribution.
pub async fn suspicious_recipient(db: &Db, window_secs: i64) -> Result<Vec<serde_json::Value>> {
    let rows: Vec<PgRow> = sqlx::query(
        r#"
        SELECT bc.owner, bc.coin_type, bc.amount, bc.tx_digest, l.label, l.category
        FROM balance_changes bc
        JOIN address_labels l ON l.address = bc.owner
        WHERE l.category IN ('hacker','mixer','sanctioned','phishing','rug_pull')
          AND bc.timestamp >= NOW() - ($1 || ' seconds')::interval
          AND bc.amount > 0
        LIMIT 100
        "#,
    )
    .bind(window_secs)
    .fetch_all(db.pool())
    .await
    .map_err(|e: sqlx::Error| Error::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r: PgRow| {
            let owner: String = r.get(0);
            let coin_type: String = r.get(1);
            let amount: bigdecimal::BigDecimal = r.get(2);
            let tx_digest: String = r.get(3);
            let label: String = r.get(4);
            let category: String = r.get(5);
            json!({
                "title": format!("Funds arrived at risky address ({})", category),
                "body": format!("{owner} ({label}) received {amount} of {coin_type} via tx {tx_digest}."),
                "rule": "suspicious_recipient",
                "address": owner,
                "label": label,
                "category": category,
                "amount": amount.to_string(),
                "tx_digest": tx_digest,
            })
        })
        .collect())
}

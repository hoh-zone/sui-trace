use serde_json::json;
use sqlx::Row;
use trace_common::{Error, error::Result};
use trace_storage::Db;

type PgRow = sqlx::postgres::PgRow;

/// Find addresses tagged as `protocol_treasury` that emitted a single
/// outbound transfer larger than `threshold_lamports` within the last
/// `window_secs` seconds.
pub async fn large_outflow(
    db: &Db,
    threshold_lamports: i128,
    window_secs: i64,
) -> Result<Vec<serde_json::Value>> {
    let rows: Vec<PgRow> = sqlx::query(
        r#"
        SELECT bc.owner, bc.coin_type, bc.amount, bc.tx_digest
        FROM balance_changes bc
        JOIN address_labels l ON l.address = bc.owner AND l.category = 'protocol_treasury'
        WHERE bc.timestamp >= NOW() - ($1 || ' seconds')::interval
          AND bc.amount < 0
          AND ABS(bc.amount) >= $2
        LIMIT 50
        "#,
    )
    .bind(window_secs)
    .bind(bigdecimal::BigDecimal::from(
        threshold_lamports.unsigned_abs() as i64,
    ))
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
            json!({
                "title": format!("Large treasury outflow: {owner}"),
                "body": format!("{owner} sent {amount} of {coin_type} (tx {tx_digest})."),
                "rule": "large_outflow",
                "address": owner,
                "amount": amount.to_string(),
                "coin_type": coin_type,
                "tx_digest": tx_digest,
            })
        })
        .collect())
}

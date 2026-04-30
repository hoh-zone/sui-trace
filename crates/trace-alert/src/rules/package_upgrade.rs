use serde_json::json;
use sqlx::Row;
use trace_common::{Error, error::Result};
use trace_storage::Db;

type PgRow = sqlx::postgres::PgRow;

/// Find packages whose `version` increased relative to a previous row
/// recently. Returns one payload per upgrade event.
pub async fn package_upgrade(db: &Db, window_secs: i64) -> Result<Vec<serde_json::Value>> {
    let rows: Vec<PgRow> = sqlx::query(
        r#"
        WITH latest AS (
            SELECT id, original_id, version, publisher, published_at,
                   LAG(version) OVER (PARTITION BY original_id ORDER BY version) AS prev_version
            FROM packages
        )
        SELECT id, original_id, version, prev_version, publisher, published_at
        FROM latest
        WHERE prev_version IS NOT NULL
          AND version > prev_version
          AND published_at >= NOW() - ($1 || ' seconds')::interval
        ORDER BY published_at DESC
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
            let id: String = r.get(0);
            let original_id: String = r.get(1);
            let version: i64 = r.get(2);
            let prev: i64 = r.get(3);
            let publisher: String = r.get(4);
            json!({
                "title": format!("Package upgraded: {original_id}"),
                "body": format!("{original_id} upgraded {prev} -> {version} (new id {id}, publisher {publisher})"),
                "rule": "package_upgrade",
                "package_id": id,
                "original_id": original_id,
                "version": version,
                "previous_version": prev,
                "publisher": publisher,
            })
        })
        .collect())
}

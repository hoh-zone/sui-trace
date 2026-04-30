use serde_json::json;
use sqlx::Row;
use trace_common::{Error, error::Result};
use trace_storage::Db;

type PgRow = sqlx::postgres::PgRow;

/// Find newly scanned packages whose security score crosses `threshold` (0-10)
/// in the last `window_secs` seconds. Drives the "new high-severity package"
/// public alert.
pub async fn high_severity_package(
    db: &Db,
    window_secs: i64,
    threshold: f32,
) -> Result<Vec<serde_json::Value>> {
    let rows: Vec<PgRow> = sqlx::query(
        r#"SELECT package_id, version, score, max_severity, scanned_at
           FROM security_reports
           WHERE scanned_at >= NOW() - ($1 || ' seconds')::interval
             AND score >= $2
           ORDER BY scanned_at DESC LIMIT 50"#,
    )
    .bind(window_secs)
    .bind(threshold)
    .fetch_all(db.pool())
    .await
    .map_err(|e: sqlx::Error| Error::Database(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r: PgRow| {
            let package_id: String = r.get(0);
            let version: i64 = r.get(1);
            let score: f32 = r.get(2);
            let max_severity: String = r.get(3);
            json!({
                "title": format!("High-severity package detected: {package_id}"),
                "body": format!("score={score:.2}, max_severity={max_severity}, version={version}"),
                "rule": "high_severity_package",
                "package_id": package_id,
                "version": version,
                "score": score,
                "max_severity": max_severity,
            })
        })
        .collect())
}

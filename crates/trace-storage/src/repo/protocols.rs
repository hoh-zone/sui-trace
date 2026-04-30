//! Curated "watched protocols" registry + per-protocol code-update events.
//!
//! Two responsibilities:
//!
//! * CRUD for the `protocols` table, which is operator-curated. Each row
//!   declares a protocol's identity (name, packages, addresses) and how
//!   prominently we surface it on the dashboard.
//!
//! * `protocol_code_events` — the indexer calls
//!   [`ProtocolRepo::record_code_event`] on every observed publish/upgrade
//!   whose `original_id` is listed in some protocol's `package_ids`. The
//!   stored row carries a JSON `summary` describing what changed
//!   (modules added/removed/renamed and a per-module bytecode-hash diff)
//!   plus a coarse severity used by the alert pipeline.

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::Row;
use trace_common::{Error, error::Result};

use crate::Db;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Protocol {
    pub id: String,
    pub name: String,
    pub package_ids: Vec<String>,
    pub category: String,
    pub website: Option<String>,
    pub defillama_slug: Option<String>,
    // Watchlist-only fields (added by migration 040000):
    pub watched: bool,
    pub priority: i32,
    pub risk_level: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub tags: Vec<String>,
    pub treasury_addresses: Vec<String>,
    pub multisig_addresses: Vec<String>,
    pub contact: Option<String>,
    pub notes: Option<String>,
    pub added_by: Option<String>,
    pub added_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct ProtocolUpsert {
    pub id: String,
    pub name: String,
    pub package_ids: Vec<String>,
    pub category: String,
    pub website: Option<String>,
    pub defillama_slug: Option<String>,
    pub watched: bool,
    pub priority: i32,
    pub risk_level: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub tags: Vec<String>,
    pub treasury_addresses: Vec<String>,
    pub multisig_addresses: Vec<String>,
    pub contact: Option<String>,
    pub notes: Option<String>,
    pub added_by: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CodeEvent {
    pub id: i64,
    pub protocol_id: String,
    pub package_id: String,
    pub original_id: String,
    pub version: i64,
    pub previous_id: Option<String>,
    pub publish_tx: Option<String>,
    pub publisher: String,
    pub kind: String,
    pub summary: JsonValue,
    pub severity: String,
    pub detected_at: DateTime<Utc>,
    pub happened_at: DateTime<Utc>,
}

pub struct ProtocolRepo<'a> {
    db: &'a Db,
}

impl<'a> ProtocolRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    // ---- Read paths ---------------------------------------------------------

    pub async fn list(&self, only_watched: bool) -> Result<Vec<Protocol>> {
        let rows = if only_watched {
            sqlx::query(&format!(
                "{SELECT_PROTOCOL} WHERE watched = TRUE ORDER BY priority DESC, name"
            ))
            .fetch_all(self.db.pool())
            .await
        } else {
            sqlx::query(&format!(
                "{SELECT_PROTOCOL} ORDER BY watched DESC, priority DESC, name"
            ))
            .fetch_all(self.db.pool())
            .await
        }
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_protocol).collect())
    }

    pub async fn get(&self, id: &str) -> Result<Option<Protocol>> {
        let row = sqlx::query(&format!("{SELECT_PROTOCOL} WHERE id = $1"))
            .bind(id)
            .fetch_optional(self.db.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.map(row_to_protocol))
    }

    /// Return every protocol that lists `original_id` in `package_ids`. Used
    /// by the indexer to fan-out publish/upgrade observations into
    /// `protocol_code_events`.
    pub async fn protocols_for_original(&self, original_id: &str) -> Result<Vec<Protocol>> {
        let rows = sqlx::query(&format!(
            "{SELECT_PROTOCOL} WHERE package_ids @> ARRAY[$1::text]"
        ))
        .bind(original_id)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_protocol).collect())
    }

    /// Return every protocol whose treasury or multisig list contains `addr`.
    pub async fn protocols_for_address(&self, addr: &str) -> Result<Vec<Protocol>> {
        let rows = sqlx::query(&format!(
            "{SELECT_PROTOCOL} WHERE treasury_addresses @> ARRAY[$1::text]
                 OR multisig_addresses @> ARRAY[$1::text]"
        ))
        .bind(addr)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_protocol).collect())
    }

    // ---- Write paths --------------------------------------------------------

    pub async fn upsert(&self, p: &ProtocolUpsert) -> Result<Protocol> {
        sqlx::query(
            r#"INSERT INTO protocols
                  (id, name, package_ids, category, website, defillama_slug,
                   watched, priority, risk_level, description, logo_url, tags,
                   treasury_addresses, multisig_addresses, contact, notes,
                   added_by, added_at, updated_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,NOW(),NOW())
               ON CONFLICT (id) DO UPDATE SET
                  name               = EXCLUDED.name,
                  package_ids        = EXCLUDED.package_ids,
                  category           = EXCLUDED.category,
                  website            = EXCLUDED.website,
                  defillama_slug     = EXCLUDED.defillama_slug,
                  watched            = EXCLUDED.watched,
                  priority           = EXCLUDED.priority,
                  risk_level         = EXCLUDED.risk_level,
                  description        = EXCLUDED.description,
                  logo_url           = EXCLUDED.logo_url,
                  tags               = EXCLUDED.tags,
                  treasury_addresses = EXCLUDED.treasury_addresses,
                  multisig_addresses = EXCLUDED.multisig_addresses,
                  contact            = EXCLUDED.contact,
                  notes              = EXCLUDED.notes,
                  added_by           = COALESCE(protocols.added_by, EXCLUDED.added_by),
                  updated_at         = NOW()"#,
        )
        .bind(&p.id)
        .bind(&p.name)
        .bind(&p.package_ids)
        .bind(&p.category)
        .bind(&p.website)
        .bind(&p.defillama_slug)
        .bind(p.watched)
        .bind(p.priority)
        .bind(&p.risk_level)
        .bind(&p.description)
        .bind(&p.logo_url)
        .bind(&p.tags)
        .bind(&p.treasury_addresses)
        .bind(&p.multisig_addresses)
        .bind(&p.contact)
        .bind(&p.notes)
        .bind(&p.added_by)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(self.get(&p.id).await?.expect("just upserted"))
    }

    pub async fn delete(&self, id: &str) -> Result<bool> {
        let r = sqlx::query("DELETE FROM protocols WHERE id = $1")
            .bind(id)
            .execute(self.db.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(r.rows_affected() > 0)
    }

    // ---- Code events --------------------------------------------------------

    /// Record a publish/upgrade event for a watched protocol. Idempotent on
    /// `(protocol_id, package_id)`.
    #[allow(clippy::too_many_arguments)]
    pub async fn record_code_event(
        &self,
        protocol_id: &str,
        package_id: &str,
        original_id: &str,
        version: u64,
        previous_id: Option<&str>,
        publish_tx: Option<&str>,
        publisher: &str,
        kind: &str,
        summary: &JsonValue,
        severity: &str,
        happened_at: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO protocol_code_events
                  (protocol_id, package_id, original_id, version, previous_id,
                   publish_tx, publisher, kind, summary, severity, happened_at)
               VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
               ON CONFLICT (protocol_id, package_id) DO UPDATE SET
                  summary    = EXCLUDED.summary,
                  severity   = EXCLUDED.severity,
                  publish_tx = COALESCE(protocol_code_events.publish_tx, EXCLUDED.publish_tx)"#,
        )
        .bind(protocol_id)
        .bind(package_id)
        .bind(original_id)
        .bind(version as i64)
        .bind(previous_id)
        .bind(publish_tx)
        .bind(publisher)
        .bind(kind)
        .bind(summary)
        .bind(severity)
        .bind(happened_at)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn recent_events_for(&self, protocol_id: &str, limit: i64) -> Result<Vec<CodeEvent>> {
        let rows = sqlx::query(
            r#"SELECT id, protocol_id, package_id, original_id, version, previous_id,
                      publish_tx, publisher, kind, summary, severity, detected_at, happened_at
               FROM protocol_code_events
               WHERE protocol_id = $1
               ORDER BY happened_at DESC
               LIMIT $2"#,
        )
        .bind(protocol_id)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_code_event).collect())
    }

    pub async fn recent_events(&self, limit: i64) -> Result<Vec<CodeEvent>> {
        let rows = sqlx::query(
            r#"SELECT id, protocol_id, package_id, original_id, version, previous_id,
                      publish_tx, publisher, kind, summary, severity, detected_at, happened_at
               FROM protocol_code_events
               ORDER BY happened_at DESC
               LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_code_event).collect())
    }

    /// Number of code events per protocol since `since`. Powers the
    /// dashboard counter on each protocol card.
    pub async fn event_counts_since(&self, since: DateTime<Utc>) -> Result<Vec<(String, i64)>> {
        let rows = sqlx::query(
            r#"SELECT protocol_id, COUNT(*)::bigint
               FROM protocol_code_events
               WHERE happened_at >= $1
               GROUP BY protocol_id"#,
        )
        .bind(since)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(|r| (r.get(0), r.get(1))).collect())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProtocolActivity {
    pub tx_digest: String,
    pub event_seq: i32,
    pub package_id: String,
    pub module: String,
    pub event_type: String,
    pub sender: String,
    pub timestamp: DateTime<Utc>,
}

impl<'a> ProtocolRepo<'a> {
    /// Recent on-chain events whose package_id matches *any* of `package_ids`.
    /// `package_ids` should be the current + historical versions of the
    /// protocol so we capture activity across upgrades.
    pub async fn recent_activity(
        &self,
        package_ids: &[String],
        limit: i64,
    ) -> Result<Vec<ProtocolActivity>> {
        if package_ids.is_empty() {
            return Ok(vec![]);
        }
        let rows = sqlx::query(
            r#"SELECT tx_digest, event_seq, package_id, module, event_type, sender, timestamp
               FROM events
               WHERE package_id = ANY($1)
               ORDER BY timestamp DESC
               LIMIT $2"#,
        )
        .bind(package_ids)
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| ProtocolActivity {
                tx_digest: r.get(0),
                event_seq: r.get(1),
                package_id: r.get(2),
                module: r.get(3),
                event_type: r.get(4),
                sender: r.get(5),
                timestamp: r.get(6),
            })
            .collect())
    }

    /// Activity count over the last `since` for the given package set. Used
    /// for the per-protocol "24h" tile on the dashboard.
    pub async fn activity_count_since(
        &self,
        package_ids: &[String],
        since: DateTime<Utc>,
    ) -> Result<i64> {
        if package_ids.is_empty() {
            return Ok(0);
        }
        let row = sqlx::query(
            r#"SELECT COUNT(*)::bigint FROM events
               WHERE package_id = ANY($1) AND timestamp >= $2"#,
        )
        .bind(package_ids)
        .bind(since)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.get(0))
    }
}

const SELECT_PROTOCOL: &str = r#"SELECT id, name, package_ids, category, website, defillama_slug,
       watched, priority, risk_level, description, logo_url, tags,
       treasury_addresses, multisig_addresses, contact, notes,
       added_by, added_at, updated_at
FROM protocols"#;

fn row_to_protocol(r: sqlx::postgres::PgRow) -> Protocol {
    Protocol {
        id: r.get(0),
        name: r.get(1),
        package_ids: r.get(2),
        category: r.get(3),
        website: r.get(4),
        defillama_slug: r.get(5),
        watched: r.get(6),
        priority: r.get(7),
        risk_level: r.get(8),
        description: r.get(9),
        logo_url: r.get(10),
        tags: r.get(11),
        treasury_addresses: r.get(12),
        multisig_addresses: r.get(13),
        contact: r.get(14),
        notes: r.get(15),
        added_by: r.get(16),
        added_at: r.get(17),
        updated_at: r.get(18),
    }
}

fn row_to_code_event(r: sqlx::postgres::PgRow) -> CodeEvent {
    CodeEvent {
        id: r.get(0),
        protocol_id: r.get(1),
        package_id: r.get(2),
        original_id: r.get(3),
        version: r.get(4),
        previous_id: r.get(5),
        publish_tx: r.get(6),
        publisher: r.get(7),
        kind: r.get(8),
        summary: r.get(9),
        severity: r.get(10),
        detected_at: r.get(11),
        happened_at: r.get(12),
    }
}

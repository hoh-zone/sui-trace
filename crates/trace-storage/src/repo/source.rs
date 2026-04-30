//! Package version lineage + decompiled / disassembled module sources.
//!
//! Two responsibilities:
//!
//! * `package_versions` — every time the indexer ingests a `Publish` /
//!   `Upgrade` operation, [`SourceRepo::record_version`] records the
//!   denormalised lineage (package_id, version, previous_id, publish_tx)
//!   so the explorer can render an upgrade timeline without recursive
//!   queries.
//! * `package_module_sources` — an external decompiler tool (or `sui move
//!   disassemble`) pushes textual representations of module bytecode in
//!   via [`SourceRepo::upsert_module_source`]. Each row is keyed by
//!   `(package_id, module_name, format)`, so multiple representations
//!   (`move-disasm`, `move-source`, `pseudo`) can coexist.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::Row;
use trace_common::{Error, error::Result};

use crate::Db;

#[derive(Debug, Clone, serde::Serialize)]
pub struct PackageVersion {
    pub package_id: String,
    pub original_id: String,
    pub version: i64,
    pub previous_id: Option<String>,
    pub publisher: String,
    pub publish_tx: Option<String>,
    pub published_at: DateTime<Utc>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ModuleSource {
    pub package_id: String,
    pub module_name: String,
    pub format: String,
    pub source: String,
    pub decompiler: String,
    pub decompiler_version: Option<String>,
    pub bytecode_hash: Option<String>,
    pub source_hash: String,
    pub decompiled_at: DateTime<Utc>,
}

/// Lightweight summary used when listing modules for a package without
/// dragging the full source text over the wire. The frontend hits the
/// per-module endpoint to fetch the body.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModuleSourceSummary {
    pub package_id: String,
    pub module_name: String,
    pub format: String,
    pub decompiler: String,
    pub decompiler_version: Option<String>,
    pub bytecode_hash: Option<String>,
    pub source_hash: String,
    pub bytes: i64,
    pub decompiled_at: DateTime<Utc>,
}

pub struct SourceRepo<'a> {
    db: &'a Db,
}

impl<'a> SourceRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    // ---- Version lineage ----------------------------------------------------

    /// Insert a version row, automatically resolving `previous_id` from the
    /// highest version we've already seen for the same `original_id`. Idempotent.
    pub async fn record_version(
        &self,
        package_id: &str,
        original_id: &str,
        version: u64,
        publisher: &str,
        publish_tx: Option<&str>,
        published_at: DateTime<Utc>,
    ) -> Result<()> {
        // Find the largest prior version (< this one) and use its package_id
        // as `previous_id`. This handles out-of-order ingestion gracefully.
        let prev_row = sqlx::query(
            r#"SELECT package_id FROM package_versions
               WHERE original_id = $1 AND version < $2
               ORDER BY version DESC LIMIT 1"#,
        )
        .bind(original_id)
        .bind(version as i64)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        let previous_id: Option<String> = prev_row.map(|r| r.get(0));

        sqlx::query(
            r#"INSERT INTO package_versions
                (package_id, original_id, version, previous_id, publisher, publish_tx, published_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (package_id) DO UPDATE SET
                   original_id  = EXCLUDED.original_id,
                   version      = EXCLUDED.version,
                   previous_id  = COALESCE(package_versions.previous_id, EXCLUDED.previous_id),
                   publisher    = EXCLUDED.publisher,
                   publish_tx   = COALESCE(package_versions.publish_tx, EXCLUDED.publish_tx),
                   published_at = EXCLUDED.published_at"#,
        )
        .bind(package_id)
        .bind(original_id)
        .bind(version as i64)
        .bind(previous_id)
        .bind(publisher)
        .bind(publish_tx)
        .bind(published_at)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        // Also fix forward-pointing rows that were inserted before us: if a
        // higher version exists with `previous_id IS NULL` and matches our
        // `original_id`, point it at the newly recorded `package_id`.
        sqlx::query(
            r#"UPDATE package_versions
               SET previous_id = $1
               WHERE original_id = $2
                 AND version = (
                     SELECT MIN(version) FROM package_versions
                     WHERE original_id = $2 AND version > $3
                 )
                 AND previous_id IS NULL"#,
        )
        .bind(package_id)
        .bind(original_id)
        .bind(version as i64)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    pub async fn lineage_by_original(&self, original_id: &str) -> Result<Vec<PackageVersion>> {
        let rows = sqlx::query(
            r#"SELECT package_id, original_id, version, previous_id, publisher,
                      publish_tx, published_at, notes
               FROM package_versions
               WHERE original_id = $1
               ORDER BY version"#,
        )
        .bind(original_id)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_version).collect())
    }

    /// Convenience: resolve `original_id` from a `package_id` and return its
    /// full lineage.
    pub async fn lineage_for(&self, package_id: &str) -> Result<Vec<PackageVersion>> {
        let row = sqlx::query("SELECT original_id FROM package_versions WHERE package_id = $1")
            .bind(package_id)
            .fetch_optional(self.db.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        let Some(row) = row else { return Ok(vec![]) };
        let original_id: String = row.get(0);
        self.lineage_by_original(&original_id).await
    }

    pub async fn version(&self, package_id: &str) -> Result<Option<PackageVersion>> {
        let row = sqlx::query(
            r#"SELECT package_id, original_id, version, previous_id, publisher,
                      publish_tx, published_at, notes
               FROM package_versions WHERE package_id = $1"#,
        )
        .bind(package_id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.map(row_to_version))
    }

    // ---- Decompiled module sources ------------------------------------------

    /// Upsert a textual representation of a module produced by an external
    /// decompiler / disassembler.
    pub async fn upsert_module_source(&self, src: &ModuleSourceUpsert<'_>) -> Result<ModuleSource> {
        let source_hash = sha256_hex(src.source.as_bytes());
        let now = Utc::now();
        sqlx::query(
            r#"INSERT INTO package_module_sources
                (package_id, module_name, format, source, decompiler, decompiler_version,
                 bytecode_hash, source_hash, decompiled_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
               ON CONFLICT (package_id, module_name, format) DO UPDATE SET
                   source             = EXCLUDED.source,
                   decompiler         = EXCLUDED.decompiler,
                   decompiler_version = EXCLUDED.decompiler_version,
                   bytecode_hash      = COALESCE(EXCLUDED.bytecode_hash, package_module_sources.bytecode_hash),
                   source_hash        = EXCLUDED.source_hash,
                   decompiled_at      = EXCLUDED.decompiled_at"#,
        )
        .bind(src.package_id)
        .bind(src.module_name)
        .bind(src.format)
        .bind(src.source)
        .bind(src.decompiler)
        .bind(src.decompiler_version)
        .bind(src.bytecode_hash)
        .bind(&source_hash)
        .bind(now)
        .execute(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(ModuleSource {
            package_id: src.package_id.to_string(),
            module_name: src.module_name.to_string(),
            format: src.format.to_string(),
            source: src.source.to_string(),
            decompiler: src.decompiler.to_string(),
            decompiler_version: src.decompiler_version.map(str::to_string),
            bytecode_hash: src.bytecode_hash.map(str::to_string),
            source_hash,
            decompiled_at: now,
        })
    }

    /// Return summaries (no source body) for every module source attached
    /// to the package.
    pub async fn list_modules(&self, package_id: &str) -> Result<Vec<ModuleSourceSummary>> {
        let rows = sqlx::query(
            r#"SELECT package_id, module_name, format, decompiler, decompiler_version,
                      bytecode_hash, source_hash, OCTET_LENGTH(source)::bigint AS bytes,
                      decompiled_at
               FROM package_module_sources
               WHERE package_id = $1
               ORDER BY module_name, format"#,
        )
        .bind(package_id)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| ModuleSourceSummary {
                package_id: r.get(0),
                module_name: r.get(1),
                format: r.get(2),
                decompiler: r.get(3),
                decompiler_version: r.get(4),
                bytecode_hash: r.get(5),
                source_hash: r.get(6),
                bytes: r.get(7),
                decompiled_at: r.get(8),
            })
            .collect())
    }

    pub async fn get_module(
        &self,
        package_id: &str,
        module_name: &str,
        format: Option<&str>,
    ) -> Result<Option<ModuleSource>> {
        // When a caller doesn't pin `format`, prefer the highest-fidelity
        // representation we have: source > pseudo > disasm.
        let row = match format {
            Some(f) => {
                sqlx::query(
                    r#"SELECT package_id, module_name, format, source, decompiler,
                          decompiler_version, bytecode_hash, source_hash, decompiled_at
                   FROM package_module_sources
                   WHERE package_id = $1 AND module_name = $2 AND format = $3"#,
                )
                .bind(package_id)
                .bind(module_name)
                .bind(f)
                .fetch_optional(self.db.pool())
                .await
            }
            None => {
                sqlx::query(
                    r#"SELECT package_id, module_name, format, source, decompiler,
                          decompiler_version, bytecode_hash, source_hash, decompiled_at
                   FROM package_module_sources
                   WHERE package_id = $1 AND module_name = $2
                   ORDER BY CASE format
                                WHEN 'move-source' THEN 0
                                WHEN 'pseudo'      THEN 1
                                WHEN 'move-disasm' THEN 2
                                ELSE 3
                            END
                   LIMIT 1"#,
                )
                .bind(package_id)
                .bind(module_name)
                .fetch_optional(self.db.pool())
                .await
            }
        }
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.map(|r| ModuleSource {
            package_id: r.get(0),
            module_name: r.get(1),
            format: r.get(2),
            source: r.get(3),
            decompiler: r.get(4),
            decompiler_version: r.get(5),
            bytecode_hash: r.get(6),
            source_hash: r.get(7),
            decompiled_at: r.get(8),
        }))
    }
}

/// Borrow-friendly payload for [`SourceRepo::upsert_module_source`].
pub struct ModuleSourceUpsert<'a> {
    pub package_id: &'a str,
    pub module_name: &'a str,
    pub format: &'a str,
    pub source: &'a str,
    pub decompiler: &'a str,
    pub decompiler_version: Option<&'a str>,
    pub bytecode_hash: Option<&'a str>,
}

fn row_to_version(r: sqlx::postgres::PgRow) -> PackageVersion {
    PackageVersion {
        package_id: r.get(0),
        original_id: r.get(1),
        version: r.get(2),
        previous_id: r.get(3),
        publisher: r.get(4),
        publish_tx: r.get(5),
        published_at: r.get(6),
        notes: r.get(7),
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let h = Sha256::digest(bytes);
    hex::encode(h)
}

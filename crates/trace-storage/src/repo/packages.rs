use chrono::{DateTime, Utc};
use sqlx::Row;
use trace_common::{
    Error,
    error::Result,
    model::{Package, PackageModule},
};

use crate::Db;

pub struct PackageRepo<'a> {
    db: &'a Db,
}

impl<'a> PackageRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn upsert(&self, pkg: &Package, modules: &[PackageModule]) -> Result<()> {
        let mut tx = self
            .db
            .pool()
            .begin()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO packages
                (id, original_id, version, publisher, modules_count, source_verified, published_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                version = EXCLUDED.version,
                publisher = EXCLUDED.publisher,
                modules_count = EXCLUDED.modules_count,
                source_verified = EXCLUDED.source_verified
            "#,
        )
        .bind(&pkg.id)
        .bind(&pkg.original_id)
        .bind(pkg.version as i64)
        .bind(&pkg.publisher)
        .bind(pkg.modules_count as i32)
        .bind(pkg.source_verified)
        .bind(pkg.published_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        for m in modules {
            sqlx::query(
                r#"
                INSERT INTO package_modules (package_id, module_name, bytecode_hash, abi_json)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (package_id, module_name) DO UPDATE SET
                    bytecode_hash = EXCLUDED.bytecode_hash,
                    abi_json = EXCLUDED.abi_json
                "#,
            )
            .bind(&m.package_id)
            .bind(&m.module_name)
            .bind(&m.bytecode_hash)
            .bind(&m.abi_json)
            .execute(&mut *tx)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        }
        tx.commit()
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Result<Option<Package>> {
        let row = sqlx::query(
            r#"SELECT id, original_id, version, publisher, modules_count, source_verified, published_at
               FROM packages WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.map(row_to_pkg))
    }

    pub async fn modules(&self, package_id: &str) -> Result<Vec<PackageModule>> {
        let rows = sqlx::query(
            r#"SELECT package_id, module_name, bytecode_hash, abi_json
               FROM package_modules WHERE package_id = $1 ORDER BY module_name"#,
        )
        .bind(package_id)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows
            .into_iter()
            .map(|r| PackageModule {
                package_id: r.get(0),
                module_name: r.get(1),
                bytecode_hash: r.get(2),
                abi_json: r.get(3),
            })
            .collect())
    }

    pub async fn recent(&self, limit: i64) -> Result<Vec<Package>> {
        let rows = sqlx::query(
            r#"SELECT id, original_id, version, publisher, modules_count, source_verified, published_at
               FROM packages ORDER BY published_at DESC LIMIT $1"#,
        )
        .bind(limit)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_pkg).collect())
    }

    pub async fn count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*)::bigint FROM packages")
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.get(0))
    }

    pub async fn count_since(&self, since: DateTime<Utc>) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*)::bigint FROM packages WHERE published_at > $1")
            .bind(since)
            .fetch_one(self.db.pool())
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.get(0))
    }

    pub async fn published_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Package>> {
        let rows = sqlx::query(
            r#"SELECT id, original_id, version, publisher, modules_count, source_verified, published_at
               FROM packages WHERE published_at >= $1 AND published_at < $2
               ORDER BY published_at DESC"#,
        )
        .bind(from)
        .bind(to)
        .fetch_all(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(rows.into_iter().map(row_to_pkg).collect())
    }
}

fn row_to_pkg(r: sqlx::postgres::PgRow) -> Package {
    Package {
        id: r.get(0),
        original_id: r.get(1),
        version: r.get::<i64, _>(2) as u64,
        publisher: r.get(3),
        modules_count: r.get::<i32, _>(4) as u32,
        source_verified: r.get(5),
        published_at: r.get::<DateTime<Utc>, _>(6),
    }
}

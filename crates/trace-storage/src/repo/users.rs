use chrono::{DateTime, Utc};
use sqlx::Row;
use trace_common::{Error, error::Result};
use uuid::Uuid;

use crate::Db;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: Option<String>,
    pub sui_address: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

pub struct UserRepo<'a> {
    db: &'a Db,
}

impl<'a> UserRepo<'a> {
    pub fn new(db: &'a Db) -> Self {
        Self { db }
    }

    pub async fn upsert_by_address(&self, address: &str) -> Result<User> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (id, email, sui_address, role, created_at)
            VALUES (gen_random_uuid(), NULL, $1, 'user', NOW())
            ON CONFLICT (sui_address) DO UPDATE SET sui_address = EXCLUDED.sui_address
            RETURNING id, email, sui_address, role, created_at
            "#,
        )
        .bind(address)
        .fetch_one(self.db.pool())
        .await
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(User {
            id: row.get(0),
            email: row.get(1),
            sui_address: row.get(2),
            role: row.get(3),
            created_at: row.get(4),
        })
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>> {
        let row =
            sqlx::query("SELECT id, email, sui_address, role, created_at FROM users WHERE id = $1")
                .bind(id)
                .fetch_optional(self.db.pool())
                .await
                .map_err(|e| Error::Database(e.to_string()))?;
        Ok(row.map(|r| User {
            id: r.get(0),
            email: r.get(1),
            sui_address: r.get(2),
            role: r.get(3),
            created_at: r.get(4),
        }))
    }
}

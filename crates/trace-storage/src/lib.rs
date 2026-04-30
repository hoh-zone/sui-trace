//! Storage abstractions used by the indexer, the API and the workers.
//!
//! The crate exposes:
//!   * `Db` — a pooled `sqlx` Postgres handle plus a typed migration runner;
//!   * `Cache` — a `deadpool-redis` based cache + pubsub helper;
//!   * `repo` — repositories that hide raw SQL behind typed methods.

pub mod cache;
pub mod db;
pub mod repo;

pub use cache::Cache;
pub use db::Db;

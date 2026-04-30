//! Alert engine.
//!
//! The engine is fed by two sources:
//!   * built-in rules (TVL drop, large outflow, suspicious recipient, new
//!     high-severity package, package upgrade, address activity);
//!   * user watchlists (subscribed via the Postgres `watchlists` table).
//!
//! When a rule fires, the engine performs deduplication, persists the alert
//! into `alert_events` and dispatches the payload to every channel the
//! watchlist requested. Channels live under `channels/` and share the
//! `Channel` trait so adding new ones (Slack, PagerDuty, …) only touches
//! one file.

pub mod channels;
pub mod dedup;
pub mod engine;
pub mod rules;

pub use engine::AlertEngine;

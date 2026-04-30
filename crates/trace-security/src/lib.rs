//! Static analysis engine for Move packages deployed on Sui.
//!
//! The engine ingests new packages via a Redis queue populated by the
//! `trace-indexer` `PackagePipeline`. For every package version it:
//!   1. Re-fetches the BCS bytecode through the Sui REST API
//!      (`/api/v1/packages/{id}`) — the indexer only stores hashes;
//!   2. Builds a lightweight per-module representation (`ModuleContext`);
//!   3. Runs every registered rule and aggregates findings into a
//!      `SecurityReport`;
//!   4. Persists the report and triggers a high-severity alert if the
//!      computed score crosses the configured threshold.
//!
//! Rules ship under `rules/` — each rule is one file. Adding a rule is a
//! matter of `mod`/`Box::new` in `engine.rs`. The trait surface is small on
//! purpose so contributors can write new heuristics without touching the
//! orchestration code.

pub mod context;
pub mod engine;
pub mod report;
pub mod rules;
pub mod worker;

pub use engine::SecurityEngine;
pub use report::ReportBuilder;

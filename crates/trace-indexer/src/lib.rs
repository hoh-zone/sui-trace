//! Sui chain ingester.
//!
//! The crate is modelled on the official `sui-indexer-alt-framework` pipeline
//! abstraction but is implemented against Sui's public JSON-RPC interface so
//! the workspace can build without pulling the Mysten monorepo. Each pipeline
//! is a long-running task that reads from a shared checkpoint stream and
//! writes its own slice of state into Postgres while persisting an
//! independent watermark.
//!
//! The trait surface is identical to what an upgrade to gRPC streaming would
//! require, so swapping the source out is a localised change to `client.rs`.

pub mod client;
pub mod model;
pub mod pipeline;
pub mod pipelines;
pub mod runner;

pub use pipeline::{Pipeline, PipelineKind};
pub use runner::IndexerRunner;

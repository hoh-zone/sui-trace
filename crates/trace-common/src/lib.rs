//! Shared primitives for the sui-trace workspace.
//!
//! This crate centralises configuration loading, error and result types,
//! tracing/observability bootstrap and the cross-crate domain models that
//! every other component depends on.

pub mod config;
pub mod error;
pub mod model;
pub mod telemetry;
pub mod time;

pub use error::{Error, Result};

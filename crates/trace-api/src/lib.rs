//! HTTP, GraphQL and WebSocket surface for sui-trace.
//!
//! The crate is binary-friendly (`src/main.rs`) but exposes its router and
//! state objects from `lib.rs` so integration tests can mount the same
//! application without spinning up the binary.

pub mod auth;
pub mod graphql;
pub mod routes;
pub mod state;
pub mod ws;

pub use state::AppState;

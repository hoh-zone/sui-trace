//! Address tagging service. Combines several sources:
//!   * curated lists shipped under `data/seeds/` (Sui Foundation, validators,
//!     known CEX wallets);
//!   * external imports — OFAC SDN list, public hack incident dumps;
//!   * community submissions reviewed in the admin console.
//!
//! The crate exposes a `LabelService` whose `lookup` method merges all
//! sources and returns the highest-confidence labels for an address — used
//! by both the UI and the alert engine's "suspicious recipient" rule.

pub mod importers;
pub mod service;

pub use service::LabelService;

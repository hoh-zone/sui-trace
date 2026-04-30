//! Periodic aggregations: daily deploy counts, active project rankings and
//! TVL polling. Everything lives behind cron-like schedulers driven by
//! `tokio` intervals — there is no external scheduler dependency.

pub mod jobs;
pub mod scheduler;
pub mod tvl;

pub use scheduler::Scheduler;

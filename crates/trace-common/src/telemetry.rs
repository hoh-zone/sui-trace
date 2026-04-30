use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Initialise tracing for a binary. Honours `RUST_LOG` and falls back to `info`.
pub fn init(service: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_level(true)
        .compact();

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .try_init();

    tracing::info!(service, "telemetry initialised");
}

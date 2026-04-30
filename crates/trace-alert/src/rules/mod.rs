//! Built-in alert rules.

mod address_activity;
mod high_severity_package;
mod large_outflow;
mod package_upgrade;
mod suspicious_recipient;
mod tvl_drop;

pub use address_activity::address_activity;
pub use high_severity_package::high_severity_package;
pub use large_outflow::large_outflow;
pub use package_upgrade::package_upgrade;
pub use suspicious_recipient::suspicious_recipient;
pub use tvl_drop::tvl_drop;

//! End-to-end smoke test against the real Sui mainnet RPC.
//!
//! Run with:
//! ```bash
//! cargo run -p trace-indexer --example smoke
//! ```
//!
//! It prints the latest checkpoint sequence number, then walks back a few
//! recent checkpoints and shows what we'd actually persist.

use anyhow::Result;
use trace_indexer::client::SuiClient;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    let url =
        std::env::var("SUI_RPC").unwrap_or_else(|_| "https://fullnode.mainnet.sui.io:443".into());
    println!("→ connecting to {url}");
    let client = SuiClient::new(&url);

    let latest = client.latest_checkpoint().await?;
    println!("✓ latest checkpoint: {latest}");

    // Probe a fresh checkpoint and an old one.
    for seq in [latest.saturating_sub(2), 1u64] {
        println!("\n--- checkpoint #{seq} ---");
        let bundle = client.get_checkpoint(seq).await?;
        let Some(b) = bundle else {
            println!("  (not available)");
            continue;
        };
        println!(
            "digest             {}\nepoch              {}\ntimestamp_ms       {}\nprevious_digest    {:?}\nnetwork_total_tx   {}\ntransactions       {}",
            b.checkpoint.digest,
            b.checkpoint.epoch,
            b.checkpoint.timestamp_ms,
            b.checkpoint.previous_digest,
            b.checkpoint.network_total_transactions,
            b.transactions.len(),
        );

        if let Some(t) = b.transactions.first() {
            println!(
                "first tx           digest={} sender={} status={} kind={} gas={} events={} balance_changes={} pkgs={} mut_objs={}",
                t.digest,
                t.sender,
                t.status,
                t.kind,
                t.gas_used,
                t.events.len(),
                t.balance_changes.len(),
                t.published_packages.len(),
                t.mutated_objects.len(),
            );
            if let Some(ev) = t.events.first() {
                println!(
                    "first event        package={} module={} type={}",
                    ev.package_id, ev.module, ev.event_type
                );
            }
        }
    }

    println!("\n✓ smoke test passed — client can read mainnet end-to-end");
    Ok(())
}

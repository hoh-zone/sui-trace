use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use trace_common::config::AppConfig;
use trace_storage::Db;
use trace_storage::repo::protocols::{ProtocolRepo, ProtocolUpsert};
use trace_storage::repo::source::{ModuleSourceUpsert, SourceRepo};

#[derive(Parser)]
#[command(name = "trace", about = "sui-trace operator CLI")]
struct Cli {
    #[arg(long, default_value = "config/default.toml", env = "TRACE_CONFIG")]
    config: String,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Apply database migrations
    Migrate,
    /// Print indexer pipeline watermarks
    Watermarks,
    /// Import labels from a JSON file
    ImportLabels {
        #[arg(long)]
        source: String,
        #[arg(long)]
        file: String,
    },
    /// Print the upgrade lineage for a package
    Versions {
        /// Package id (the on-chain id, not the original_id)
        #[arg(long)]
        package: String,
    },
    /// Manage the curated "watched protocols" registry surfaced on the
    /// operator dashboard.
    #[command(subcommand)]
    Watch(WatchCmd),
    /// Push a decompiled / disassembled module source produced by an
    /// external tool. Reads `--file` (or stdin) as UTF-8 text and writes it
    /// into `package_module_sources`. Idempotent.
    PushSource {
        /// Package id the source belongs to
        #[arg(long)]
        package: String,
        /// Module name (e.g. `coin`, `pool`, `Foo`)
        #[arg(long)]
        module: String,
        /// Source format. One of: `move-disasm`, `move-source`, `pseudo`.
        #[arg(long, default_value = "move-disasm")]
        format: String,
        /// File containing the source. Use `-` for stdin.
        #[arg(long)]
        file: String,
        /// Tool name used to produce the artefact (e.g. `sui-disassembler`)
        #[arg(long, default_value = "sui-disassembler")]
        decompiler: String,
        /// Tool version
        #[arg(long)]
        decompiler_version: Option<String>,
        /// Optional sha256 of the source bytecode (hex)
        #[arg(long)]
        bytecode_hash: Option<String>,
    },
}

// Clap subcommands naturally have large variants because of all the
// per-flag fields; boxing every option just to satisfy this lint would make
// the CLI code unreadable.
#[allow(clippy::large_enum_variant)]
#[derive(Subcommand)]
enum WatchCmd {
    /// List protocols (defaults to watched only)
    List {
        /// Include protocols where `watched = false`
        #[arg(long)]
        all: bool,
    },
    /// Add or update a protocol in the watchlist. Idempotent on `--id`.
    Add {
        #[arg(long)]
        id: String,
        #[arg(long)]
        name: String,
        /// Comma-separated list of `original_id`s the protocol owns.
        #[arg(long, value_delimiter = ',')]
        packages: Vec<String>,
        #[arg(long, default_value = "other")]
        category: String,
        #[arg(long)]
        website: Option<String>,
        #[arg(long)]
        defillama_slug: Option<String>,
        #[arg(long, default_value_t = true)]
        watched: bool,
        #[arg(long, default_value_t = 50)]
        priority: i32,
        #[arg(long, default_value = "unknown")]
        risk: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        logo_url: Option<String>,
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        treasury: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        multisig: Vec<String>,
        #[arg(long)]
        contact: Option<String>,
        #[arg(long)]
        notes: Option<String>,
        #[arg(long, default_value = "operator-cli")]
        added_by: String,
    },
    /// Remove a protocol entirely (also drops its `protocol_code_events`).
    Remove {
        #[arg(long)]
        id: String,
    },
    /// Print the protocol's recent code events
    Events {
        #[arg(long)]
        id: String,
        #[arg(long, default_value_t = 20)]
        limit: i64,
    },
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    trace_common::telemetry::init("trace-cli");
    let cli = Cli::parse();
    let cfg = AppConfig::load(&cli.config)?;
    let db = Db::connect(&cfg.database).await?;

    match cli.cmd {
        Command::Migrate => {
            db.migrate().await?;
            println!("migrations applied");
        }
        Command::Watermarks => {
            let names = [
                "checkpoints",
                "transactions",
                "events",
                "objects",
                "balance_changes",
                "packages",
            ];
            let repo = trace_storage::repo::checkpoints::CheckpointRepo::new(&db);
            for n in names {
                let wm = repo.watermark(n).await?;
                println!("{n:>16} -> {wm:?}");
            }
        }
        Command::ImportLabels { source, file } => {
            trace_labels::importers::import_from_file(&db, &source, &file).await?;
            println!("labels imported from {file}");
        }
        Command::Versions { package } => {
            let lineage = SourceRepo::new(&db).lineage_for(&package).await?;
            if lineage.is_empty() {
                println!("no version rows recorded yet for {package}");
            } else {
                println!(
                    "{:>3}  {:<66}  {:<66}  published_at",
                    "v", "package_id", "previous_id"
                );
                for v in lineage {
                    println!(
                        "{:>3}  {:<66}  {:<66}  {}",
                        v.version,
                        v.package_id,
                        v.previous_id.unwrap_or_else(|| "—".into()),
                        v.published_at
                    );
                }
            }
        }
        Command::Watch(cmd) => run_watch(cmd, &db).await?,
        Command::PushSource {
            package,
            module,
            format,
            file,
            decompiler,
            decompiler_version,
            bytecode_hash,
        } => {
            let body = if file == "-" {
                let mut buf = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf)
                    .context("read stdin")?;
                buf
            } else {
                std::fs::read_to_string(&file).with_context(|| format!("read {file}"))?
            };
            if body.is_empty() {
                bail!("source body is empty");
            }
            let stored = SourceRepo::new(&db)
                .upsert_module_source(&ModuleSourceUpsert {
                    package_id: &package,
                    module_name: &module,
                    format: &format,
                    source: &body,
                    decompiler: &decompiler,
                    decompiler_version: decompiler_version.as_deref(),
                    bytecode_hash: bytecode_hash.as_deref(),
                })
                .await?;
            println!(
                "stored {pkg}::{module} ({format}, {bytes} bytes, sha256 {hash}…) decompiler={dc}",
                pkg = stored.package_id,
                module = stored.module_name,
                format = stored.format,
                bytes = stored.source.len(),
                hash = &stored.source_hash[..12.min(stored.source_hash.len())],
                dc = stored.decompiler
            );
        }
    }
    Ok(())
}

async fn run_watch(cmd: WatchCmd, db: &Db) -> Result<()> {
    let repo = ProtocolRepo::new(db);
    match cmd {
        WatchCmd::List { all } => {
            let rows = repo.list(!all).await?;
            if rows.is_empty() {
                println!("(no protocols)");
            } else {
                println!(
                    "{:<14} {:<24} {:<10} {:<8} {:<10} pkgs",
                    "id", "name", "category", "watched", "risk"
                );
                for p in rows {
                    println!(
                        "{:<14} {:<24} {:<10} {:<8} {:<10} {}",
                        p.id,
                        p.name,
                        p.category,
                        p.watched,
                        p.risk_level,
                        p.package_ids.len()
                    );
                }
            }
        }
        WatchCmd::Add {
            id,
            name,
            packages,
            category,
            website,
            defillama_slug,
            watched,
            priority,
            risk,
            description,
            logo_url,
            tags,
            treasury,
            multisig,
            contact,
            notes,
            added_by,
        } => {
            if id.is_empty() || name.is_empty() {
                bail!("--id and --name are required");
            }
            let stored = repo
                .upsert(&ProtocolUpsert {
                    id,
                    name,
                    package_ids: packages,
                    category,
                    website,
                    defillama_slug,
                    watched,
                    priority,
                    risk_level: risk,
                    description,
                    logo_url,
                    tags,
                    treasury_addresses: treasury,
                    multisig_addresses: multisig,
                    contact,
                    notes,
                    added_by: Some(added_by),
                })
                .await?;
            println!(
                "stored {id} ({name}) watched={watched} risk={risk} pkgs={n}",
                id = stored.id,
                name = stored.name,
                watched = stored.watched,
                risk = stored.risk_level,
                n = stored.package_ids.len()
            );
        }
        WatchCmd::Remove { id } => {
            if repo.delete(&id).await? {
                println!("removed {id}");
            } else {
                println!("not found: {id}");
            }
        }
        WatchCmd::Events { id, limit } => {
            let evs = repo.recent_events_for(&id, limit).await?;
            if evs.is_empty() {
                println!("(no code events)");
            } else {
                for e in evs {
                    println!(
                        "{ts}  {kind:<8} {sev:<8} v{ver:<3} {pkg}  prev={prev}",
                        ts = e.happened_at,
                        kind = e.kind,
                        sev = e.severity,
                        ver = e.version,
                        pkg = e.package_id,
                        prev = e.previous_id.unwrap_or_else(|| "-".into())
                    );
                }
            }
        }
    }
    Ok(())
}

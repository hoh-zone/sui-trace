# Sui-Trace Architecture

## Crate map

| Crate            | Role                                                      | Binary           |
| ---------------- | --------------------------------------------------------- | ---------------- |
| `trace-common`   | Config, error types, telemetry, shared domain models      | —                |
| `trace-storage`  | Postgres pool (sqlx), Redis cache + pubsub, repositories  | —                |
| `trace-indexer`  | Pipeline framework + 6 pipelines reading Sui RPC         | `trace-indexer`  |
| `trace-security` | Move static-analysis engine + 10 rules + Redis worker     | `trace-security` |
| `trace-analytics`| TVL poller + daily/active aggregations scheduler          | `trace-analytics`|
| `trace-alert`    | Rule engine + 4 channels + dedup + retry                  | `trace-alert`    |
| `trace-labels`   | Address tag service + bulk importer                       | —                |
| `trace-api`      | REST + GraphQL + WebSocket surface (Axum 0.8)             | `trace-api`      |
| `trace-cli`      | Operator CLI (migrate, import-labels, watermarks)         | `trace`          |

## Process topology

```
            +-----------------+
            |  Sui Mainnet    |
            +--------+--------+
                     |
            JSON-RPC | (sui_getCheckpoint, sui_multiGetTransactionBlocks)
                     v
+------------+   +---------+   +--------------+
| trace-     |-->| Postgres|<--| trace-api    |--> REST / GraphQL / WS
| indexer    |   | + Redis |   +--------------+
+-----+------+   +----+----+
      |               ^
      | enqueue       |
      v               |
+-------------+   +---+----------+
| trace-      |   | trace-       |
| security    |   | analytics    |
+-------------+   +--------------+
                       |
                       v
                  +-----------+
                  |trace-alert|---> Telegram / Webhook / Email / Discord
                  +-----------+
```

## Data flow

1. **Indexer** reads checkpoints in batches, fans them out to six idempotent
   pipelines (checkpoints, transactions, events, objects, balance_changes,
   packages). Each pipeline persists its own watermark in
   `pipeline_watermarks` so the runner can resume cleanly.
2. The **PackagePipeline** also pushes a `{package_id, version}` envelope onto
   the `trace:packages:to_scan` Redis list.
3. **trace-security** pops the queue, fetches the package's normalized Move
   modules from the Sui RPC, runs all rules, scores the report and saves it
   into `security_reports` + `security_findings`.
4. **trace-analytics** runs three loops on tokio intervals: the daily-deploy
   aggregation, the active-package ranking, and the DefiLlama TVL poller.
5. **trace-alert** ticks every 60s, applies built-in rules (TVL drop,
   suspicious recipient, large outflow, package upgrade, high-severity
   package), deduplicates per `dedup_window_secs`, persists each fired alert
   in `alert_events` and dispatches it through the channels each watchlist
   declares.
6. The **API** is the single read path. REST is the lowest-friction surface,
   GraphQL is exposed at `/graphql` with an embedded GraphiQL playground at
   the same URL, and WebSocket clients subscribe at `/ws`.

## Storage

- Postgres 17 with the optional TimescaleDB extension. The hot tables
  (`checkpoints`, `transactions`, `events`, `tvl_snapshots`) are converted to
  hypertables when the extension is available; the migration is a no-op
  otherwise so the same schema works on vanilla Postgres.
- Redis 7 carries the security scan queue, the alert pubsub channel and the
  WebSocket fan-out cache.
- MinIO/S3 is reserved for storing raw module bytecode and verified source
  archives; the M1 indexer stores hashes only.

## Security model

- Service-to-service: workers connect through the cluster's network policy,
  no exposed ports.
- Public API: stateless JWTs (HS256) with a `jwt_ttl_secs` rolling expiry.
  Login is via Sign-In With Sui (SIWS) personal message; the verifier is
  pluggable so the SDK-grade signature check can replace the M1 stub
  without touching call sites.
- Sensitive secrets (DB URL, JWT secret, bot tokens, SMTP creds) are pulled
  from Kubernetes secrets via the Helm `externalSecrets` block.

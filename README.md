# sui-trace

Developer-first explorer for the [Sui](https://sui.io) blockchain. Goes
beyond a generic explorer with three focus areas:

1. **Real-time security checks** for every newly deployed Move package.
2. **Daily deployment + activity analytics** for builders and analysts.
3. **TVL tracking and theft alerts** for protocols and treasuries.

Plus the usual explorer surface (transactions, addresses, packages, objects,
checkpoints, search) and a community-curated **address label library** with
risk tagging.

The backend is a pure Rust workspace (Rust 1.95 / edition 2024, Axum 0.8,
sqlx 0.8, Tokio, async-graphql). The frontend is React 19 + Vite 6 +
TanStack Router/Query + Tailwind + ECharts.

## Repository layout

```
sui-trace/
  Cargo.toml                 # Cargo workspace
  crates/
    trace-common/            # config, error, telemetry, shared models
    trace-storage/           # Postgres + Redis + repositories
    trace-indexer/           # 6 pipelines + Sui RPC client + runner
    trace-security/          # static-analysis engine + 10 rules + worker
    trace-analytics/         # daily/active aggregations + DefiLlama poller
    trace-alert/             # rule engine + 4 channels + dedup
    trace-labels/            # address tag service + JSON importer
    trace-api/               # REST + GraphQL + WebSocket
    trace-cli/               # operator CLI: migrate, import-labels, …
  migrations/                # sqlx Postgres migrations
  config/                    # default.toml etc.
  web/                       # React app
  deploy/                    # Docker, docker-compose, Helm chart, Grafana
  docs/                      # architecture, openapi, security rules, api
```

## Quick start

```bash
# 1. Bring up Postgres, Redis, ClickHouse, MinIO, Prometheus, Grafana
docker compose -f deploy/docker-compose.yml up -d

# 2. Apply migrations
cargo run --bin trace -- migrate

# 3. Run the indexer (talks to Sui mainnet RPC by default)
cargo run --bin trace-indexer

# 4. Run the security worker
cargo run --bin trace-security

# 5. Run the analytics scheduler (daily/active stats + TVL polling)
cargo run --bin trace-analytics

# 6. Run the alert engine
cargo run --bin trace-alert

# 7. Run the API
cargo run --bin trace-api

# 8. Run the frontend
cd web && npm install && npm run dev
```

The API listens on `:8080`, the frontend on `:5173`. The Vite dev server
already proxies `/api`, `/graphql` and `/ws` so the two halves run as one
during development.

## Configuration

All services read [`config/default.toml`](config/default.toml). Any value
can be overridden by an environment variable named
`TRACE__<section>__<key>`, e.g. `TRACE__database__url=postgres://...`.

## Operator CLI

```bash
cargo run --bin trace -- migrate
cargo run --bin trace -- watermarks
cargo run --bin trace -- import-labels --source imported --file data/ofac.json
```

## Documentation

- [`docs/architecture.md`](docs/architecture.md) — process topology and crate
  responsibilities.
- [`docs/security-rules.md`](docs/security-rules.md) — V1 rule library and
  how to add a rule.
- [`docs/api.md`](docs/api.md) — REST / GraphQL / WS surface notes and auth.
- [`docs/openapi.yaml`](docs/openapi.yaml) — machine-readable OpenAPI 3.1
  description.

## Project plan

The full product/architecture plan lives in
[`.cursor/plans/sui-trace_项目规划_*.plan.md`](.cursor/plans). The codebase
ships with a working slice of every milestone (M1 explorer + indexer, M2
security + labels + alerts, M3 analytics + TVL + frontend). Production
hardening (full SIWS verification with the Mysten SDK, source-validation
service, ClickHouse mirror, Helm hardening, OpenTelemetry traces) is tracked
under M4+ in the plan.

## License

Apache-2.0.

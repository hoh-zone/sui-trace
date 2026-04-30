-- Sui-Trace initial schema. Idempotent where possible; assumes Postgres 15+.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS pipeline_watermarks (
    pipeline TEXT PRIMARY KEY,
    high_watermark BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS checkpoints (
    sequence_number BIGINT PRIMARY KEY,
    digest TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    previous_digest TEXT,
    network_total_transactions BIGINT NOT NULL DEFAULT 0,
    epoch BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS checkpoints_ts_idx ON checkpoints (timestamp DESC);

CREATE TABLE IF NOT EXISTS transactions (
    digest TEXT PRIMARY KEY,
    checkpoint_seq BIGINT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    sender TEXT NOT NULL,
    status TEXT NOT NULL,
    gas_used BIGINT NOT NULL DEFAULT 0,
    gas_price BIGINT NOT NULL DEFAULT 0,
    kind TEXT NOT NULL DEFAULT 'unknown'
);
CREATE INDEX IF NOT EXISTS transactions_sender_idx ON transactions (sender, timestamp DESC);
CREATE INDEX IF NOT EXISTS transactions_ts_idx ON transactions (timestamp DESC);
CREATE INDEX IF NOT EXISTS transactions_cp_idx ON transactions (checkpoint_seq);

CREATE TABLE IF NOT EXISTS events (
    tx_digest TEXT NOT NULL,
    event_seq INT NOT NULL,
    package_id TEXT NOT NULL,
    module TEXT NOT NULL,
    event_type TEXT NOT NULL,
    sender TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    payload JSONB NOT NULL,
    PRIMARY KEY (tx_digest, event_seq)
);
CREATE INDEX IF NOT EXISTS events_pkg_idx ON events (package_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS events_type_idx ON events (event_type);
CREATE INDEX IF NOT EXISTS events_sender_idx ON events (sender, timestamp DESC);

CREATE TABLE IF NOT EXISTS packages (
    id TEXT PRIMARY KEY,
    original_id TEXT NOT NULL,
    version BIGINT NOT NULL,
    publisher TEXT NOT NULL,
    modules_count INT NOT NULL,
    source_verified BOOLEAN NOT NULL DEFAULT FALSE,
    published_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX IF NOT EXISTS packages_publisher_idx ON packages (publisher);
CREATE INDEX IF NOT EXISTS packages_published_idx ON packages (published_at DESC);
CREATE INDEX IF NOT EXISTS packages_original_idx ON packages (original_id);

CREATE TABLE IF NOT EXISTS package_modules (
    package_id TEXT NOT NULL,
    module_name TEXT NOT NULL,
    bytecode_hash TEXT NOT NULL,
    abi_json JSONB NOT NULL,
    PRIMARY KEY (package_id, module_name)
);

CREATE TABLE IF NOT EXISTS objects (
    object_id TEXT NOT NULL,
    version BIGINT NOT NULL,
    object_type TEXT NOT NULL,
    owner TEXT,
    contents JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (object_id, version)
);
CREATE INDEX IF NOT EXISTS objects_id_idx ON objects (object_id, version DESC);
CREATE INDEX IF NOT EXISTS objects_owner_idx ON objects (owner);
CREATE INDEX IF NOT EXISTS objects_type_idx ON objects (object_type);

CREATE TABLE IF NOT EXISTS balance_changes (
    tx_digest TEXT NOT NULL,
    seq SERIAL,
    owner TEXT NOT NULL,
    coin_type TEXT NOT NULL,
    amount NUMERIC(78, 0) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (tx_digest, seq)
);
CREATE INDEX IF NOT EXISTS balance_owner_idx ON balance_changes (owner, timestamp DESC);

CREATE TABLE IF NOT EXISTS security_reports (
    package_id TEXT NOT NULL,
    version BIGINT NOT NULL,
    score REAL NOT NULL,
    max_severity TEXT NOT NULL,
    findings_count INT NOT NULL DEFAULT 0,
    scanned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (package_id, version)
);
CREATE INDEX IF NOT EXISTS security_reports_pkg_idx ON security_reports (package_id, version DESC);
CREATE INDEX IF NOT EXISTS security_reports_score_idx ON security_reports (score DESC);

CREATE TABLE IF NOT EXISTS security_findings (
    id BIGSERIAL PRIMARY KEY,
    package_id TEXT NOT NULL,
    version BIGINT NOT NULL,
    rule_id TEXT NOT NULL,
    rule_name TEXT NOT NULL,
    severity TEXT NOT NULL,
    confidence REAL NOT NULL,
    module TEXT NOT NULL,
    function TEXT,
    location TEXT NOT NULL,
    message TEXT NOT NULL,
    suggestion TEXT NOT NULL DEFAULT ''
);
CREATE INDEX IF NOT EXISTS findings_pkg_idx ON security_findings (package_id, version);
CREATE INDEX IF NOT EXISTS findings_rule_idx ON security_findings (rule_id);

CREATE TABLE IF NOT EXISTS address_labels (
    address TEXT NOT NULL,
    label TEXT NOT NULL,
    category TEXT NOT NULL,
    source TEXT NOT NULL,
    confidence REAL NOT NULL DEFAULT 0.5,
    evidence_url TEXT,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (address, label, source)
);
CREATE INDEX IF NOT EXISTS labels_addr_idx ON address_labels (address);
CREATE INDEX IF NOT EXISTS labels_cat_idx ON address_labels (category);

CREATE TABLE IF NOT EXISTS protocols (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    package_ids TEXT[] NOT NULL DEFAULT '{}',
    category TEXT NOT NULL DEFAULT 'other',
    website TEXT,
    defillama_slug TEXT
);

CREATE TABLE IF NOT EXISTS tvl_snapshots (
    protocol_id TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    tvl_usd DOUBLE PRECISION NOT NULL,
    breakdown JSONB NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (protocol_id, timestamp)
);
CREATE INDEX IF NOT EXISTS tvl_protocol_ts_idx ON tvl_snapshots (protocol_id, timestamp DESC);

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email TEXT UNIQUE,
    sui_address TEXT UNIQUE,
    role TEXT NOT NULL DEFAULT 'user',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS watchlists (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    target_type TEXT NOT NULL,
    target_id TEXT NOT NULL,
    rules JSONB NOT NULL DEFAULT '{}'::jsonb,
    channels JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS watchlists_user_idx ON watchlists (user_id);
CREATE INDEX IF NOT EXISTS watchlists_target_idx ON watchlists (target_type, target_id);

CREATE TABLE IF NOT EXISTS alert_events (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    watchlist_id UUID REFERENCES watchlists(id) ON DELETE SET NULL,
    rule_id TEXT NOT NULL,
    fired_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    payload JSONB NOT NULL,
    delivered BOOLEAN NOT NULL DEFAULT FALSE,
    attempts INT NOT NULL DEFAULT 0,
    last_error TEXT
);
CREATE INDEX IF NOT EXISTS alerts_user_idx ON alert_events (user_id, fired_at DESC);
CREATE INDEX IF NOT EXISTS alerts_undelivered_idx ON alert_events (delivered, fired_at) WHERE delivered = FALSE;

CREATE TABLE IF NOT EXISTS alert_dedup (
    key TEXT PRIMARY KEY,
    fired_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS alert_dedup_ts_idx ON alert_dedup (fired_at);

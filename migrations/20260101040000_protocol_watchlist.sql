-- Sui-Trace: featured-protocol watchlist + per-protocol code-update events.
--
-- The base `protocols` table only carried name/category/defillama_slug. To
-- run a curated "watched protocols" dashboard we need:
--
--   * Operator-facing fields: watched flag, priority, risk_level, logos,
--     descriptions, treasury / multisig addresses, security contacts,
--     audit notes, tags.
--   * A per-protocol audit trail of every package publish / upgrade we've
--     observed — `protocol_code_events`. Rows are inserted by the indexer
--     whenever it sees a `Publish` / `Upgrade` whose original_id is listed
--     in any watched protocol's `package_ids`.

ALTER TABLE protocols
    ADD COLUMN IF NOT EXISTS watched           BOOLEAN     NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS priority          INTEGER     NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS risk_level        TEXT        NOT NULL DEFAULT 'unknown',
    ADD COLUMN IF NOT EXISTS description       TEXT,
    ADD COLUMN IF NOT EXISTS logo_url          TEXT,
    ADD COLUMN IF NOT EXISTS tags              TEXT[]      NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS treasury_addresses TEXT[]     NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS multisig_addresses TEXT[]     NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS contact           TEXT,
    ADD COLUMN IF NOT EXISTS notes             TEXT,
    ADD COLUMN IF NOT EXISTS added_by          TEXT,
    ADD COLUMN IF NOT EXISTS added_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ADD COLUMN IF NOT EXISTS updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- GIN index lets the indexer answer "which protocols watch this original_id?"
-- in a single shot:
--   SELECT id FROM protocols WHERE package_ids @> ARRAY[$1::text];
CREATE INDEX IF NOT EXISTS protocols_pkg_ids_gin   ON protocols USING GIN (package_ids);
CREATE INDEX IF NOT EXISTS protocols_treasury_gin  ON protocols USING GIN (treasury_addresses);
CREATE INDEX IF NOT EXISTS protocols_multisig_gin  ON protocols USING GIN (multisig_addresses);
CREATE INDEX IF NOT EXISTS protocols_watched_idx   ON protocols (watched, priority DESC);

CREATE TABLE IF NOT EXISTS protocol_code_events (
    id           BIGSERIAL PRIMARY KEY,
    protocol_id  TEXT        NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
    package_id   TEXT        NOT NULL,
    original_id  TEXT        NOT NULL,
    version      BIGINT      NOT NULL,
    previous_id  TEXT,
    publish_tx   TEXT,
    publisher    TEXT        NOT NULL,
    -- 'publish' for the first version we ever see for an original_id under this
    -- protocol; 'upgrade' for every subsequent version.
    kind         TEXT        NOT NULL,
    -- Free-form JSON describing what changed: { modules_added, modules_removed,
    -- modules_changed[{name, prev_hash, new_hash}], ... }.
    summary      JSONB       NOT NULL DEFAULT '{}'::jsonb,
    -- 'info' for normal upgrades, 'warning' for module additions/removals
    -- on a watched protocol, 'critical' if the security worker flagged the
    -- new package.
    severity     TEXT        NOT NULL DEFAULT 'info',
    detected_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    happened_at  TIMESTAMPTZ NOT NULL,
    UNIQUE (protocol_id, package_id)
);
CREATE INDEX IF NOT EXISTS pce_protocol_idx ON protocol_code_events (protocol_id, happened_at DESC);
CREATE INDEX IF NOT EXISTS pce_severity_idx ON protocol_code_events (severity, happened_at DESC);
CREATE INDEX IF NOT EXISTS pce_happened_idx ON protocol_code_events (happened_at DESC);

-- Mark seed protocols as `watched` so the dashboard isn't empty on a fresh
-- install. Operators can flip more on via the admin API.
UPDATE protocols
SET watched = TRUE,
    priority = CASE id
                  WHEN 'cetus-amm' THEN 90
                  WHEN 'navi'      THEN 85
                  WHEN 'scallop'   THEN 80
                  WHEN 'suilend'   THEN 75
                  WHEN 'aftermath' THEN 70
                  ELSE priority
              END
WHERE id IN ('cetus-amm', 'navi', 'scallop', 'suilend', 'aftermath');

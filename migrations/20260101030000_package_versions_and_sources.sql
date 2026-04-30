-- Sui-Trace: package version lineage + decompiled / disassembled module sources.
--
-- The base `packages` table records every (package_id, version) we observe.
-- This migration adds two pieces of context that the explorer needs but that
-- can't be derived cheaply at query time:
--
--   1. `package_versions` is a denormalised lineage table. For every version
--      `N` of a package's `original_id`, we record the previous on-chain
--      package_id (`N-1`) and the publish/upgrade transaction digest. This
--      lets the frontend render the whole upgrade timeline in O(versions)
--      without a recursive query.
--
--   2. `package_module_sources` stores per-module source code. The indexer
--      itself cannot decompile Move bytecode efficiently — instead, an
--      external decompiler (or the disassembler shipped with `sui move`)
--      pushes textual results into this table via the API. Each row is
--      keyed by (package_id, module_name, format) so multiple
--      representations (`move-disasm`, `move-source`, `pseudo`, …) can
--      coexist for the same module.

CREATE TABLE IF NOT EXISTS package_versions (
    package_id  TEXT PRIMARY KEY,
    original_id TEXT NOT NULL,
    version     BIGINT NOT NULL,
    previous_id TEXT,
    publisher   TEXT NOT NULL,
    publish_tx  TEXT,
    published_at TIMESTAMPTZ NOT NULL,
    notes       TEXT,
    CONSTRAINT package_versions_unique UNIQUE (original_id, version)
);
CREATE INDEX IF NOT EXISTS package_versions_orig_idx
    ON package_versions (original_id, version DESC);
CREATE INDEX IF NOT EXISTS package_versions_pub_idx
    ON package_versions (publisher);
CREATE INDEX IF NOT EXISTS package_versions_published_idx
    ON package_versions (published_at DESC);

CREATE TABLE IF NOT EXISTS package_module_sources (
    package_id        TEXT NOT NULL,
    module_name       TEXT NOT NULL,
    format            TEXT NOT NULL DEFAULT 'move-disasm',
    source            TEXT NOT NULL,
    decompiler        TEXT NOT NULL DEFAULT 'unknown',
    decompiler_version TEXT,
    bytecode_hash     TEXT,
    source_hash       TEXT NOT NULL,
    decompiled_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (package_id, module_name, format)
);
CREATE INDEX IF NOT EXISTS pkg_src_pkg_idx ON package_module_sources (package_id);
CREATE INDEX IF NOT EXISTS pkg_src_decompiled_idx
    ON package_module_sources (decompiled_at DESC);

-- Backfill `package_versions` from existing rows in `packages` so the new
-- table is immediately useful on already-running deployments. The previous_id
-- is computed via window function ordering by version.
INSERT INTO package_versions (package_id, original_id, version, previous_id, publisher, publish_tx, published_at)
SELECT
    p.id,
    p.original_id,
    p.version,
    LAG(p.id) OVER (PARTITION BY p.original_id ORDER BY p.version) AS previous_id,
    p.publisher,
    NULL,
    p.published_at
FROM packages p
ON CONFLICT (package_id) DO NOTHING;

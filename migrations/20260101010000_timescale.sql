-- Convert hot tables to TimescaleDB hypertables when the extension is available.
-- Safe no-op if running on vanilla Postgres.

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_available_extensions WHERE name = 'timescaledb') THEN
        CREATE EXTENSION IF NOT EXISTS timescaledb;
        PERFORM create_hypertable('checkpoints', 'timestamp', if_not_exists => true, migrate_data => true);
        PERFORM create_hypertable('transactions', 'timestamp', if_not_exists => true, migrate_data => true);
        PERFORM create_hypertable('events', 'timestamp', if_not_exists => true, migrate_data => true);
        PERFORM create_hypertable('tvl_snapshots', 'timestamp', if_not_exists => true, migrate_data => true);
    END IF;
END $$;

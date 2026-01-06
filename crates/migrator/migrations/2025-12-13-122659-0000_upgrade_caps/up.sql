-- Your SQL goes here
CREATE TYPE upgrade_compatibility_policy AS ENUM (
    'compatible',
    'additive',
    'dep-only',
    'immutable'
);

CREATE TABLE IF NOT EXISTS upgrade_caps (
    object_id TEXT PRIMARY KEY,
    policy upgrade_compatibility_policy NOT NULL DEFAULT 'compatible',
    created_seq_checkpoint BIGINT NOT NULL,
    created_tx_digest TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS upgrade_cap_transfers (
    object_id TEXT NOT NULL,
    old_owner_address TEXT NOT NULL,
    new_owner_address TEXT NOT NULL,
    seq_checkpoint BIGINT NOT NULL,
    tx_digest TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (object_id, tx_digest)
);

CREATE INDEX IF NOT EXISTS 
    upgrade_cap_transfers_object_idx ON upgrade_cap_transfers USING HASH (object_id);

CREATE TABLE IF NOT EXISTS upgrade_cap_versions (
    object_id TEXT NOT NULL,
    package_id TEXT NOT NULL,
    version BIGINT NOT NULL,
    seq_checkpoint BIGINT NOT NULL,
    tx_digest TEXT NOT NULL,
    publisher TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (object_id, version)
);

CREATE INDEX IF NOT EXISTS 
    upgrade_cap_versions_object_idx ON upgrade_cap_versions USING HASH (object_id);

CREATE INDEX IF NOT EXISTS 
    upgrade_cap_versions_package_idx ON upgrade_cap_versions USING HASH (package_id);
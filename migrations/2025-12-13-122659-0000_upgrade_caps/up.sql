-- Your SQL goes here
CREATE TYPE upgrade_compatibility_policy AS ENUM (
    'compatible',
    'additive',
    'dep-only',
    'immutable'
);

CREATE TABLE IF NOT EXISTS upgrade_caps (
    object_id TEXT PRIMARY KEY,
    package_id TEXT NOT NULL,
    owner_address TEXT NOT NULL,
    policy upgrade_compatibility_policy NOT NULL DEFAULT 'compatible',
    version BIGINT NOT NULL DEFAULT 1,
    init_seq_checkpoint BIGINT NOT NULL,
    init_tx_digest TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);


CREATE TABLE IF NOT EXISTS upgrade_cap_transfers_history (
    object_id TEXT NOT NULL,
    old_owner_address TEXT NOT NULL,
    new_owner_address TEXT NOT NULL,
    tx_seq_checkpoint BIGINT NOT NULL,
    tx_digest TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (object_id, tx_digest)
);

CREATE TABLE IF NOT EXISTS upgrade_cap_versions_history (
    object_id TEXT NOT NULL,
    version INT NOT NULL,
    tx_seq_checkpoint BIGINT NOT NULL,
    tx_digest TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (object_id, version)
);
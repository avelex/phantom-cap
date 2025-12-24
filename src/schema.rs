// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "upgrade_compatibility_policy"))]
    pub struct UpgradeCompatibilityPolicy;
}

diesel::table! {
    upgrade_cap_transfers_history (object_id, tx_digest) {
        object_id -> Text,
        old_owner_address -> Text,
        new_owner_address -> Text,
        tx_seq_checkpoint -> Int8,
        tx_digest -> Text,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    upgrade_cap_versions_history (object_id, version) {
        object_id -> Text,
        version -> Int4,
        tx_seq_checkpoint -> Int8,
        tx_digest -> Text,
        timestamp -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UpgradeCompatibilityPolicy;

    upgrade_caps (object_id) {
        object_id -> Text,
        package_id -> Text,
        owner_address -> Text,
        policy -> UpgradeCompatibilityPolicy,
        version -> Int8,
        init_seq_checkpoint -> Int8,
        init_tx_digest -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    upgrade_cap_transfers_history,
    upgrade_cap_versions_history,
    upgrade_caps,
);

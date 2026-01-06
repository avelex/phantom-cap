use anyhow;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};

use crate::models;

use crate::schema::upgrade_cap_transfers::dsl as upgrade_cap_transfers_dsl;
use crate::schema::upgrade_cap_versions::dsl as upgrade_cap_versions_dsl;
use crate::schema::upgrade_caps::dsl as upgrade_caps_dsl;

pub async fn get_cap_by_id(
    conn: &mut AsyncPgConnection,
    cap_id: &str,
) -> anyhow::Result<models::UpgradeCap> {
    upgrade_caps_dsl::upgrade_caps
        .filter(upgrade_caps_dsl::object_id.eq(cap_id))
        .first::<models::UpgradeCap>(conn)
        .await
        .map_err(|_| anyhow::anyhow!("Cap not found"))
}

pub async fn get_cap_latest_version(
    conn: &mut AsyncPgConnection,
    cap_id: &str,
) -> anyhow::Result<models::UpgradeCapVersion> {
    upgrade_cap_versions_dsl::upgrade_cap_versions
        .filter(upgrade_cap_versions_dsl::object_id.eq(cap_id))
        .order(upgrade_cap_versions_dsl::version.desc())
        .first::<models::UpgradeCapVersion>(conn)
        .await
        .map_err(|_| anyhow::anyhow!("Cap version not found"))
}

pub async fn get_cap_latest_transfer(
    conn: &mut AsyncPgConnection,
    cap_id: &str,
) -> anyhow::Result<models::UpgradeCapTransfer> {
    upgrade_cap_transfers_dsl::upgrade_cap_transfers
        .filter(upgrade_cap_transfers_dsl::object_id.eq(cap_id))
        .order(upgrade_cap_transfers_dsl::seq_checkpoint.desc())
        .first::<models::UpgradeCapTransfer>(conn)
        .await
        .map_err(|_| anyhow::anyhow!("Cap transfer not found"))
}

pub async fn get_cap_first_transfer(
    conn: &mut AsyncPgConnection,
    cap_id: &str,
) -> anyhow::Result<models::UpgradeCapTransfer> {
    upgrade_cap_transfers_dsl::upgrade_cap_transfers
        .filter(upgrade_cap_transfers_dsl::object_id.eq(cap_id))
        .order(upgrade_cap_transfers_dsl::seq_checkpoint.asc())
        .first::<models::UpgradeCapTransfer>(conn)
        .await
        .map_err(|_| anyhow::anyhow!("Cap transfer not found"))
}

pub async fn get_cap_versions_history(
    conn: &mut AsyncPgConnection,
    cap_id: &str,
) -> anyhow::Result<Vec<models::UpgradeCapVersion>> {
    upgrade_cap_versions_dsl::upgrade_cap_versions
        .filter(upgrade_cap_versions_dsl::object_id.eq(cap_id))
        .order(upgrade_cap_versions_dsl::seq_checkpoint.desc())
        .load::<models::UpgradeCapVersion>(conn)
        .await
        .map_err(|_| anyhow::anyhow!("Failed to get cap versions history"))
}

pub async fn get_cap_transfers_history(
    conn: &mut AsyncPgConnection,
    cap_id: &str,
) -> anyhow::Result<Vec<models::UpgradeCapTransfer>> {
    upgrade_cap_transfers_dsl::upgrade_cap_transfers
        .filter(upgrade_cap_transfers_dsl::object_id.eq(cap_id))
        .order(upgrade_cap_transfers_dsl::seq_checkpoint.desc())
        .load::<models::UpgradeCapTransfer>(conn)
        .await
        .map_err(|_| anyhow::anyhow!("Failed to get cap transfers history"))
}

pub async fn get_upgrade_caps_count(conn: &mut AsyncPgConnection) -> anyhow::Result<i64> {
    upgrade_caps_dsl::upgrade_caps
        .count()
        .get_result::<i64>(conn)
        .await
        .map_err(|_| anyhow::anyhow!("Failed to get upgrade caps count"))
}

pub async fn get_packages_count(conn: &mut AsyncPgConnection) -> anyhow::Result<i64> {
    upgrade_cap_versions_dsl::upgrade_cap_versions
        .count()
        .get_result::<i64>(conn)
        .await
        .map_err(|_| anyhow::anyhow!("Failed to get packages count"))
}

pub async fn get_transfers_count(conn: &mut AsyncPgConnection) -> anyhow::Result<i64> {
    upgrade_cap_transfers_dsl::upgrade_cap_transfers
        .count()
        .get_result::<i64>(conn)
        .await
        .map_err(|_| anyhow::anyhow!("Failed to get transfers count"))
}

pub async fn get_package_by_id(
    conn: &mut AsyncPgConnection,
    id: &String,
) -> anyhow::Result<models::UpgradeCapVersion> {
    upgrade_cap_versions_dsl::upgrade_cap_versions
        .filter(upgrade_cap_versions_dsl::package_id.eq(&id))
        .first::<models::UpgradeCapVersion>(conn)
        .await
        .map_err(|_| anyhow::anyhow!("Package not found"))
}

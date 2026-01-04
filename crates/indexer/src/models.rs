use crate::schema::sql_types::UpgradeCompatibilityPolicy;
use crate::schema::*;
use anyhow::{Error, anyhow};
use chrono::{DateTime, Utc};
use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::*;
use std::io::Write;
use sui_indexer_alt_framework::FieldCount;
use sui_types::base_types::SuiAddress;
use sui_types::move_package::UpgradePolicy;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Clone)]
#[diesel(sql_type = UpgradeCompatibilityPolicy)]
pub enum UpgradeCompatibilityPolicyEnum {
    Compatible,
    Additive,
    DepOnly,
    Immutable,
}

impl UpgradeCompatibilityPolicyEnum {
    pub fn from_u8(policy: u8) -> Result<Self, Error> {
        match policy {
            val if val == UpgradePolicy::Compatible as u8 => {
                Ok(UpgradeCompatibilityPolicyEnum::Compatible)
            }
            val if val == UpgradePolicy::Additive as u8 => {
                Ok(UpgradeCompatibilityPolicyEnum::Additive)
            }
            val if val == UpgradePolicy::DepOnly as u8 => {
                Ok(UpgradeCompatibilityPolicyEnum::DepOnly)
            }
            _ => Err(anyhow!(format!(
                "Invalid UpgradeCompatibilityPolicy: {}",
                policy
            ))),
        }
    }
}

impl ToSql<UpgradeCompatibilityPolicy, Pg> for UpgradeCompatibilityPolicyEnum {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            UpgradeCompatibilityPolicyEnum::Compatible => out.write_all(b"compatible")?,
            UpgradeCompatibilityPolicyEnum::Additive => out.write_all(b"additive")?,
            UpgradeCompatibilityPolicyEnum::DepOnly => out.write_all(b"dep_only")?,
            UpgradeCompatibilityPolicyEnum::Immutable => out.write_all(b"immutable")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<UpgradeCompatibilityPolicy, Pg> for UpgradeCompatibilityPolicyEnum {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"compatible" => Ok(UpgradeCompatibilityPolicyEnum::Compatible),
            b"additive" => Ok(UpgradeCompatibilityPolicyEnum::Additive),
            b"dep_only" => Ok(UpgradeCompatibilityPolicyEnum::DepOnly),
            b"immutable" => Ok(UpgradeCompatibilityPolicyEnum::Immutable),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

pub struct FullUpgradeCap {
    pub object_id: String,
    pub package_id: String,
    pub version: i64,
    pub owner_address: String,
    pub policy: UpgradeCompatibilityPolicyEnum,
    pub created_seq_checkpoint: i64,
    pub created_tx_digest: String,
    pub created_at: DateTime<Utc>,
}

impl FullUpgradeCap {
    pub fn db_dto(&self) -> UpgradeCap {
        UpgradeCap {
            object_id: self.object_id.clone(),
            policy: self.policy.clone(),
            created_seq_checkpoint: self.created_seq_checkpoint,
            created_tx_digest: self.created_tx_digest.clone(),
            created_at: self.created_at,
        }
    }

    pub fn creation_version(&self) -> UpgradeCapVersion {
        UpgradeCapVersion {
            object_id: self.object_id.clone(),
            package_id: self.package_id.clone(),
            version: self.version,
            seq_checkpoint: self.created_seq_checkpoint,
            tx_digest: self.created_tx_digest.clone(),
            publisher: self.owner_address.clone(),
            timestamp: self.created_at,
        }
    }

    pub fn creation_transfer(&self) -> UpgradeCapTransfer {
        UpgradeCapTransfer {
            object_id: self.object_id.clone(),
            old_owner_address: SuiAddress::ZERO.to_string(),
            new_owner_address: self.owner_address.clone(),
            seq_checkpoint: self.created_seq_checkpoint,
            tx_digest: self.created_tx_digest.clone(),
            timestamp: self.created_at,
        }
    }
}

#[derive(Insertable, Clone, FieldCount)]
#[diesel(table_name = upgrade_caps)]
pub struct UpgradeCap {
    pub object_id: String,
    pub policy: UpgradeCompatibilityPolicyEnum,
    pub created_seq_checkpoint: i64,
    pub created_tx_digest: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Clone, FieldCount, Debug)]
#[diesel(table_name = upgrade_cap_transfers)]
pub struct UpgradeCapTransfer {
    pub object_id: String,
    pub old_owner_address: String,
    pub new_owner_address: String,
    pub seq_checkpoint: i64,
    pub tx_digest: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Insertable, Clone, FieldCount, Debug)]
#[diesel(table_name = upgrade_cap_versions)]
pub struct UpgradeCapVersion {
    pub object_id: String,
    pub package_id: String,
    pub version: i64,
    pub seq_checkpoint: i64,
    pub tx_digest: String,
    pub publisher: String,
    pub timestamp: DateTime<Utc>,
}

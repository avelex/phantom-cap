use std::fmt;
use std::io::Write;

use crate::schema::*;
use diesel::{deserialize, prelude::*, serialize};

use crate::schema::sql_types::UpgradeCompatibilityPolicy;
use chrono::{DateTime, Utc};
use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{IsNull, Output, ToSql};

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Clone)]
#[diesel(sql_type = UpgradeCompatibilityPolicy)]
pub enum UpgradeCompatibilityPolicyEnum {
    Compatible,
    Additive,
    DepOnly,
    Immutable,
}

impl fmt::Display for UpgradeCompatibilityPolicyEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            UpgradeCompatibilityPolicyEnum::Compatible => write!(f, "Compatible"),
            UpgradeCompatibilityPolicyEnum::Additive => write!(f, "Additive"),
            UpgradeCompatibilityPolicyEnum::DepOnly => write!(f, "DepOnly"),
            UpgradeCompatibilityPolicyEnum::Immutable => write!(f, "Immutable"),
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

#[derive(Insertable, Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = upgrade_caps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(primary_key(object_id))]
pub struct UpgradeCap {
    pub object_id: String,
    pub policy: UpgradeCompatibilityPolicyEnum,
    pub created_seq_checkpoint: i64,
    pub created_tx_digest: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = upgrade_cap_transfers)]
pub struct UpgradeCapTransfer {
    pub object_id: String,
    pub old_owner_address: String,
    pub new_owner_address: String,
    pub seq_checkpoint: i64,
    pub tx_digest: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Insertable, Queryable, Selectable, Clone, Debug)]
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

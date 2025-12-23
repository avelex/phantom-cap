use std::io::Write;
use std::time::SystemTime;

use crate::schema::sql_types::UpgradeCompatibilityPolicy;
use crate::schema::*;
use chrono::{DateTime, Utc};
use diesel::data_types::PgTimestamp;
use diesel::deserialize::{FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::prelude::*;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::*;
use sui_indexer_alt_framework::FieldCount;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Clone)]
#[diesel(sql_type = UpgradeCompatibilityPolicy)]
pub enum UpgradeCompatibilityPolicyEnum {
    Compatible,
    Additive,
    DepOnly,
    Immutable,
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

#[derive(Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = transaction_digests)]
pub struct StoredTransactionDigest {
    pub tx_digest: String,
    pub checkpoint_sequence_number: i64,
}

#[derive(Insertable, Clone, FieldCount)]
#[diesel(table_name = upgrade_caps)]
pub struct UpgradeCap {
    pub object_id: String,
    pub package_id: String,
    pub owner_address: String,
    pub policy: UpgradeCompatibilityPolicyEnum,
    pub version: i64,
    pub init_seq_checkpoint: i64,
    pub init_tx_digest: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

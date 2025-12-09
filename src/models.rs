use crate::schema::transaction_digests;
use diesel::prelude::*;
use sui_indexer_alt_framework::FieldCount;

#[derive(Insertable, Debug, Clone, FieldCount)]
#[diesel(table_name = transaction_digests)]
pub struct StoredTransactionDigest {
    pub tx_digest: String,
    pub checkpoint_sequence_number: i64,
}

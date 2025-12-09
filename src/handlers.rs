use anyhow::Result;
use std::sync::Arc;
use sui_indexer_alt_framework::pipeline::Processor;
use sui_types::full_checkpoint_content::Checkpoint;

use crate::models::StoredTransactionDigest;
use crate::schema::transaction_digests::dsl::*;
use diesel_async::RunQueryDsl;
use sui_indexer_alt_framework::{
    pipeline::sequential::Handler,
    postgres::{Connection, Db},
};

pub struct TransactionDigestHandler;

#[async_trait::async_trait]
impl Processor for TransactionDigestHandler {
    const NAME: &'static str = "transaction_digest_handler";

    type Value = StoredTransactionDigest;

    async fn process(&self, checkpoint: &Arc<Checkpoint>) -> Result<Vec<Self::Value>> {
        let checkpoint_seq = checkpoint.summary.sequence_number as i64;
        let digests = checkpoint
            .transactions
            .iter()
            .map(|tx| StoredTransactionDigest {
                tx_digest: tx.transaction.digest().to_string(),
                checkpoint_sequence_number: checkpoint_seq,
            })
            .collect();

        println!("Processed checkpoint {}", checkpoint_seq);
        Ok(digests)
    }
}

#[async_trait::async_trait]
impl Handler for TransactionDigestHandler {
    type Store = Db;
    type Batch = Vec<Self::Value>;

    fn batch(&self, batch: &mut Self::Batch, values: std::vec::IntoIter<Self::Value>) {
        batch.extend(values);
    }

    async fn commit<'a>(&self, batch: &Self::Batch, conn: &mut Connection<'a>) -> Result<usize> {
        let inserted = diesel::insert_into(transaction_digests)
            .values(batch)
            .on_conflict(tx_digest)
            .do_nothing()
            .execute(conn)
            .await?;
        println!("Committed {} digests", inserted);
        Ok(inserted)
    }
}

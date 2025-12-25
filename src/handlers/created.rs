use anyhow::Result;
use chrono::DateTime;
use diesel::result::Error;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use std::sync::Arc;
use sui_indexer_alt_framework::pipeline::Processor;
use sui_indexer_alt_framework::{
    pipeline::sequential::Handler,
    postgres::{Connection, Db},
};
use sui_types::object::Owner;
use sui_types::storage::ObjectKey;
use sui_types::{
    effects::TransactionEffectsAPI,
    full_checkpoint_content::{Checkpoint, ExecutedTransaction, ObjectSet},
    move_package::UpgradeCap as UpgradeCapMove,
    object::Data,
    transaction::{Command, TransactionDataAPI, TransactionKind},
};

use crate::schema::upgrade_cap_transfers::dsl::{
    object_id as upgrade_cap_transfers_object_id, tx_digest as upgrade_cap_transfers_tx_digest,
    upgrade_cap_transfers,
};
use crate::schema::upgrade_cap_versions::dsl::{
    object_id as upgrade_cap_versions_object_id, upgrade_cap_versions,
    version as upgrade_cap_versions_version,
};
use crate::schema::upgrade_caps::dsl::*;
use crate::{models::UpgradeCap, models::UpgradeCompatibilityPolicyEnum};

pub struct UpgradeCapHandler;

fn publish_command_exists(checkpoint: &Arc<Checkpoint>) -> Vec<&ExecutedTransaction> {
    let mut transactions: Vec<&ExecutedTransaction> = Vec::new();

    for tx in checkpoint.transactions.iter() {
        match tx.transaction.kind() {
            TransactionKind::ProgrammableTransaction(program_tx)
            | TransactionKind::ProgrammableSystemTransaction(program_tx) => {
                if program_tx
                    .commands
                    .iter()
                    .any(|cmd| matches!(cmd, Command::Publish(_, _)))
                {
                    transactions.push(tx);
                }
            }
            _ => continue,
        }
    }

    transactions
}

pub struct OwnableUpgradeCap {
    upgrade_cap: UpgradeCapMove,
    owner: String,
    tx_digest: String,
}

fn get_created_upgrade_caps(
    tx: &ExecutedTransaction,
    object_set: &ObjectSet,
) -> Vec<OwnableUpgradeCap> {
    let mut caps: Vec<OwnableUpgradeCap> = Vec::new();

    for effects in tx.effects.created() {
        let obj = object_set
            .get(&ObjectKey(effects.0.0, effects.0.1))
            .unwrap();

        let owner = match obj.owner() {
            Owner::AddressOwner(address) => address.to_string(),
            Owner::ObjectOwner(address) => address.to_string(),
            Owner::Shared {
                initial_shared_version: _,
            } => "shared".to_string(),
            Owner::Immutable => "immutable".to_string(),
            Owner::ConsensusAddressOwner {
                start_version: _,
                owner: address,
            } => address.to_string(),
        };

        match &obj.data {
            Data::Move(move_object) => {
                if !obj.type_().unwrap().is_upgrade_cap() {
                    continue;
                }

                let upgrade_cap = move_object.to_rust::<UpgradeCapMove>().unwrap();

                caps.push(OwnableUpgradeCap {
                    upgrade_cap,
                    owner,
                    tx_digest: tx.transaction.digest().to_string(),
                });
            }
            _ => continue,
        }
    }

    caps
}

#[async_trait::async_trait]
impl Processor for UpgradeCapHandler {
    const NAME: &'static str = "created_handler";

    type Value = UpgradeCap;

    async fn process(&self, checkpoint: &Arc<Checkpoint>) -> Result<Vec<Self::Value>> {
        let checkpoint_seq = checkpoint.summary.sequence_number as i64;
        let timestamp =
            DateTime::from_timestamp_millis(checkpoint.summary.timestamp_ms as i64).unwrap();

        let published_txs = publish_command_exists(checkpoint);
        if published_txs.is_empty() {
            return Ok(vec![]);
        }

        Ok(published_txs
            .iter()
            .filter(|tx| tx.effects.status().is_ok())
            .flat_map(|tx| get_created_upgrade_caps(tx, &checkpoint.object_set))
            .map(|ownable_upgrade_cap| {
                println!(
                    "[CREATED] Tx: {} Id: {}",
                    ownable_upgrade_cap.tx_digest,
                    ownable_upgrade_cap
                        .upgrade_cap
                        .id
                        .object_id()
                        .to_hex_literal(),
                );

                UpgradeCap {
                    object_id: ownable_upgrade_cap
                        .upgrade_cap
                        .id
                        .object_id()
                        .to_hex_literal(),
                    package_id: ownable_upgrade_cap
                        .upgrade_cap
                        .package
                        .bytes
                        .to_hex_literal(),
                    owner_address: ownable_upgrade_cap.owner,
                    policy: UpgradeCompatibilityPolicyEnum::Compatible,
                    version: ownable_upgrade_cap.upgrade_cap.version as i64,
                    init_seq_checkpoint: checkpoint_seq,
                    init_tx_digest: ownable_upgrade_cap.tx_digest,
                    created_at: timestamp,
                    updated_at: timestamp,
                }
            })
            .collect::<Vec<Self::Value>>())
    }
}

#[async_trait::async_trait]
impl Handler for UpgradeCapHandler {
    type Store = Db;
    type Batch = Vec<Self::Value>;

    fn batch(&self, batch: &mut Self::Batch, values: std::vec::IntoIter<Self::Value>) {
        batch.extend(values);
    }

    async fn commit<'a>(&self, batch: &Self::Batch, conn: &mut Connection<'a>) -> Result<usize> {
        let batch = batch.clone();
        let result = conn
            .transaction::<usize, Error, _>(|tx_conn| {
                async move {
                    let inserted = diesel::insert_into(upgrade_caps)
                        .values(&batch)
                        .on_conflict(object_id)
                        .do_nothing()
                        .execute(tx_conn)
                        .await?;

                    let creation_transfers = batch
                        .iter()
                        .map(|cap| cap.creation_transfer())
                        .collect::<Vec<_>>();

                    diesel::insert_into(upgrade_cap_transfers)
                        .values(creation_transfers)
                        .on_conflict((
                            upgrade_cap_transfers_object_id,
                            upgrade_cap_transfers_tx_digest,
                        ))
                        .do_nothing()
                        .execute(tx_conn)
                        .await?;

                    let creation_versions = batch
                        .iter()
                        .map(|cap| cap.creation_version())
                        .collect::<Vec<_>>();

                    diesel::insert_into(upgrade_cap_versions)
                        .values(creation_versions)
                        .on_conflict((upgrade_cap_versions_object_id, upgrade_cap_versions_version))
                        .do_nothing()
                        .execute(tx_conn)
                        .await?;

                    Ok(inserted)
                }
                .scope_boxed()
            })
            .await?;

        Ok(result)
    }
}

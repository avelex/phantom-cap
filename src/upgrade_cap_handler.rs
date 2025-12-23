use anyhow::Result;
use chrono::DateTime;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use sui_indexer_alt_framework::pipeline::Processor;
use sui_indexer_alt_framework::{
    pipeline::sequential::Handler,
    postgres::{Connection, Db},
};
use sui_types::object::Owner;
use sui_types::{
    base_types::{ObjectID, SuiAddress},
    effects::TransactionEffectsAPI,
    full_checkpoint_content::{Checkpoint, ExecutedTransaction, ObjectSet},
    move_package::UpgradeCap as UpgradeCapMove,
    object::Data,
    transaction::{Command, TransactionDataAPI, TransactionKind},
};

use crate::schema::upgrade_caps::dsl::*;
use crate::{models::UpgradeCap, models::UpgradeCompatibilityPolicyEnum, schema::upgrade_caps};

pub struct UpgradeCapHandler;

fn publish_command_exists(checkpoint: &Arc<Checkpoint>) -> Vec<&ExecutedTransaction> {
    let mut transactions: Vec<&ExecutedTransaction> = Vec::new();

    for tx in checkpoint.transactions.iter() {
        match tx.transaction.kind() {
            TransactionKind::ProgrammableTransaction(program_tx) => {
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
}

fn get_upgrade_cap(tx: &ExecutedTransaction, object_set: &ObjectSet) -> Option<OwnableUpgradeCap> {
    for obj in tx.output_objects(object_set) {
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

                return Some(OwnableUpgradeCap { upgrade_cap, owner });
            }
            _ => continue,
        }
    }

    None
}

#[async_trait::async_trait]
impl Processor for UpgradeCapHandler {
    const NAME: &'static str = "upgrade_cap_handler";

    type Value = UpgradeCap;

    async fn process(&self, checkpoint: &Arc<Checkpoint>) -> Result<Vec<Self::Value>> {
        let checkpoint_seq = checkpoint.summary.sequence_number as i64;
        let timestamp =
            DateTime::from_timestamp_millis(checkpoint.summary.timestamp_ms as i64).unwrap();

        let published_txs = publish_command_exists(checkpoint);
        if published_txs.is_empty() {
            return Ok(vec![]);
        }

        let mut caps: Vec<Self::Value> = Vec::new();

        for tx in published_txs.iter() {
            if let Some(ownable_upgrade_cap) = get_upgrade_cap(tx, &checkpoint.object_set) {
                println!(
                    "TX: {} Upgrade cap: {} Policy: {}",
                    tx.transaction.digest(),
                    ownable_upgrade_cap
                        .upgrade_cap
                        .id
                        .object_id()
                        .to_hex_literal(),
                    ownable_upgrade_cap.upgrade_cap.policy
                );

                caps.push(UpgradeCap {
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
                    init_tx_digest: tx.transaction.digest().to_string(),
                    created_at: timestamp,
                    updated_at: timestamp,
                });
            }
        }

        Ok(caps)
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
        let inserted = diesel::insert_into(upgrade_caps)
            .values(batch)
            .on_conflict(object_id)
            .do_nothing()
            .execute(conn)
            .await?;
        Ok(inserted)
    }
}

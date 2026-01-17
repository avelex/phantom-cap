use anyhow::{Ok, Result};
use chrono::DateTime;
use diesel_async::RunQueryDsl;
use log::info;
use std::sync::Arc;
use sui_indexer_alt_framework::pipeline::Processor;
use sui_indexer_alt_framework::{
    pipeline::sequential::Handler,
    postgres::{Connection, Db},
};
use sui_types::effects::TransactionEffectsAPI;
use sui_types::storage::ObjectKey;
use sui_types::transaction::{Argument, CallArg, ObjectArg};
use sui_types::{
    base_types::SuiAddress,
    full_checkpoint_content::Checkpoint,
    object::Data,
    transaction::{Command, TransactionDataAPI, TransactionKind},
};

use crate::models::UpgradeCapTransfer;
use crate::schema::upgrade_cap_transfers::dsl::*;

pub struct UpgradeCapHandler;

#[async_trait::async_trait]
impl Processor for UpgradeCapHandler {
    const NAME: &'static str = "transfer_handler";

    type Value = UpgradeCapTransfer;

    async fn process(&self, checkpoint: &Arc<Checkpoint>) -> Result<Vec<Self::Value>> {
        let checkpoint_seq = checkpoint.summary.sequence_number as i64;
        let checkpoint_timestamp =
            DateTime::from_timestamp_millis(checkpoint.summary.timestamp_ms as i64).unwrap();

        Ok(checkpoint
            .transactions
            .iter()
            .filter(|tx| tx.effects.status().is_ok())
            .flat_map(|tx| {
                let pt = match tx.transaction.kind() {
                    TransactionKind::ProgrammableTransaction(pt) => pt,
                    TransactionKind::ProgrammableSystemTransaction(pt) => pt,
                    _ => return vec![],
                };

                pt.commands
                    .iter()
                    .filter_map(|command| {
                        if let Command::TransferObjects(objects, receiver) = command {
                            Some((objects, receiver))
                        } else {
                            None
                        }
                    })
                    .flat_map(|(objects, receiver)| {
                        let receiver_address = if let Argument::Input(idx) = receiver {
                            pt.inputs
                                .get(*idx as usize)
                                .and_then(|arg| {
                                    if let CallArg::Pure(bytes) = arg {
                                        SuiAddress::from_bytes(bytes).ok()
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or_default()
                        } else {
                            SuiAddress::default()
                        };

                        objects.iter().filter_map(move |object_arg| {
                            let Argument::Input(idx) = object_arg else {
                                return None;
                            };

                            let CallArg::Object(ObjectArg::ImmOrOwnedObject(obj_ref)) =
                                pt.inputs.get(*idx as usize)?
                            else {
                                return None;
                            };

                            let obj = checkpoint
                                .object_set
                                .get(&ObjectKey(obj_ref.0, obj_ref.1))?;

                            let Data::Move(move_data) = &obj.data else {
                                return None;
                            };

                            if !move_data.type_().is_upgrade_cap() {
                                return None;
                            }

                            info!(
                                "[TRANSFER] Tx: {} Id: {}",
                                tx.transaction.digest().to_string(),
                                obj.id().to_hex_literal()
                            );

                            Some(UpgradeCapTransfer {
                                object_id: obj.id().to_hex_literal(),
                                old_owner_address: obj.get_single_owner().unwrap().to_string(),
                                new_owner_address: receiver_address.to_string(),
                                tx_digest: tx.transaction.digest().to_string(),
                                seq_checkpoint: checkpoint_seq,
                                timestamp: checkpoint_timestamp,
                            })
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect())
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
        // TODO: check if new owner is "immutable",
        // update upgrade cap policy as immutable.
        let inserted = diesel::insert_into(upgrade_cap_transfers)
            .values(batch)
            .on_conflict((object_id, tx_digest))
            .do_nothing()
            .execute(conn)
            .await?;
        Ok(inserted)
    }
}

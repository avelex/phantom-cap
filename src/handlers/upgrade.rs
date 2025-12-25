use anyhow::Result;
use chrono::DateTime;
use diesel_async::RunQueryDsl;
use std::sync::Arc;
use sui_types::SUI_FRAMEWORK_PACKAGE_ID;
use sui_types::effects::TransactionEffectsAPI;
use sui_types::move_package::UpgradeCap;
use sui_types::object::Data;
use sui_types::storage::ObjectKey;
use sui_types::transaction::{
    Argument, CallArg, Command, ObjectArg, TransactionDataAPI, TransactionKind,
};

use sui_indexer_alt_framework::pipeline::Processor;
use sui_indexer_alt_framework::{
    pipeline::sequential::Handler,
    postgres::{Connection, Db},
};
use sui_types::full_checkpoint_content::Checkpoint;

use crate::models::UpgradeCapVersion;
use crate::schema::upgrade_cap_versions::dsl::*;

pub struct UpgradeCapHandler;

#[async_trait::async_trait]
impl Processor for UpgradeCapHandler {
    const NAME: &'static str = "upgrade_cap_handler";

    type Value = UpgradeCapVersion;

    // нужно найти транзакции где есть MoveCall(commit_upgrade),
    // 1 аргумент должен быть upgrade cap, найти его в mutaded
    // преобразовать upgrade cap в upgrade cap version
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
                        if let Command::MoveCall(call) = command {
                            Some(call)
                        } else {
                            None
                        }
                    })
                    .filter(|call| {
                        call.package.eq(&SUI_FRAMEWORK_PACKAGE_ID)
                            && call.module == "package"
                            && call.function == "commit_upgrade"
                    })
                    .filter_map(|call| {
                        // assume that first argument is upgrade cap.
                        let mutated_upgrade_cap = call.arguments.get(0).and_then(|arg| {
                            let Argument::Input(idx) = arg else {
                                return None;
                            };

                            // take first input.
                            let CallArg::Object(ObjectArg::ImmOrOwnedObject(obj_ref)) =
                                pt.inputs.get(*idx as usize)?
                            else {
                                return None;
                            };

                            // find in mutated objects upgrade cap by object id.
                            let Some(mutated_cap_ref) = tx
                                .effects
                                .mutated_excluding_gas()
                                .into_iter()
                                .find(|(mutaded, _)| mutaded.0.eq(&obj_ref.0))
                            else {
                                return None;
                            };

                            // try to find mutated upgrade cap in checkpoint object set.
                            checkpoint
                                .object_set
                                .get(&ObjectKey(mutated_cap_ref.0.0, mutated_cap_ref.0.1))
                                .and_then(|obj| {
                                    let Data::Move(move_data) = &obj.data else {
                                        return None;
                                    };

                                    if !move_data.type_().is_upgrade_cap() {
                                        return None;
                                    }

                                    let upgrade_cap = obj.to_rust::<UpgradeCap>().unwrap();

                                    Some(upgrade_cap)
                                })
                        });

                        if let Some(upgrade_cap) = mutated_upgrade_cap {
                            println!(
                                "[UPGRADE] Tx: {} Id: {}",
                                tx.transaction.digest(),
                                upgrade_cap.id.object_id().to_hex_literal()
                            );

                            Some(UpgradeCapVersion {
                                object_id: upgrade_cap.id.object_id().to_hex_literal(),
                                version: upgrade_cap.version as i64,
                                package_id: upgrade_cap.package.bytes.to_hex_literal(),
                                tx_digest: tx.transaction.digest().to_string(),
                                tx_seq_checkpoint: checkpoint_seq,
                                timestamp: checkpoint_timestamp,
                            })
                        } else {
                            None
                        }
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
        let inserted = diesel::insert_into(upgrade_cap_versions)
            .values(batch)
            .on_conflict((object_id, version))
            .do_nothing()
            .execute(conn)
            .await?;

        Ok(inserted)
    }
}

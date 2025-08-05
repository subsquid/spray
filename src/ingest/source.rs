use crate::data::ItemIndex;
use crate::geyser::api::subscribe_update::UpdateOneof;
use crate::geyser::api::{CommitmentLevel, SubscribeRequest, SubscribeRequestFilterBlocksMeta, SubscribeRequestFilterTransactions, SubscribeUpdateBlockMeta, SubscribeUpdateTransaction};
use crate::geyser::solana::storage::confirmed_block::{CompiledInstruction, MessageAddressTableLookup, MessageHeader, TransactionStatusMeta};
use crate::geyser::GeyserClient;
use crate::Name;
use anyhow::bail;
use std::collections::HashMap;
use std::time::Duration;
use tokio_stream::StreamExt;
use tracing::{debug, error, info, instrument};


#[derive(Debug)]
pub struct SourceMessage {
    pub source: Name,
    pub update: SourceUpdate,
}


#[derive(Debug)]
pub enum SourceUpdate {
    Block(SubscribeUpdateBlockMeta),
    Transaction(TransactionUpdate)
}


#[derive(Debug)]
pub struct TransactionUpdate {
    pub slot: u64,
    pub index: ItemIndex,
    pub signatures: Vec<Vec<u8>>,
    pub header: MessageHeader,
    pub account_keys: Vec<Vec<u8>>,
    pub recent_blockhash: Vec<u8>,
    pub instructions: Vec<CompiledInstruction>,
    pub versioned: bool,
    pub address_table_lookups: Vec<MessageAddressTableLookup>,
    pub meta: TransactionStatusMeta
}


impl TransactionUpdate {
    pub fn from_subscription_update(update: SubscribeUpdateTransaction) -> Result<Self, &'static str> {
        let Some(upd) = update.transaction else {
            return Err(".transaction");
        };

        let Some(tx) = upd.transaction else {
            return Err(".transaction.transaction");
        };

        let Some(message) = tx.message else {
            return Err(".transaction.transaction.message")
        };

        let Some(header) = message.header else {
            return Err(".transaction.transaction.message.header")
        };

        let Some(meta) = upd.meta else {
            return Err(".transaction.meta")
        };

        Ok(TransactionUpdate {
            slot: update.slot,
            index: upd.index as usize,
            signatures: tx.signatures,
            header,
            account_keys: message.account_keys,
            recent_blockhash: message.recent_blockhash,
            instructions: message.instructions,
            versioned: message.versioned,
            address_table_lookups: message.address_table_lookups,
            meta
        })
    }
}


#[instrument(name = "source", skip_all, fields(source = name))]
pub async fn source_loop(
    output: tokio::sync::mpsc::Sender<SourceMessage>,
    name: Name,
    mut client: GeyserClient
) -> anyhow::Result<()> 
{
    let mut first_session = true;
    let mut errors = 0;
    let backoff_ms = [0, 0, 200, 500, 1000, 2000, 5000];
    while !output.is_closed() {
        let mut update_received = false;
        match source_session(
            &output,
            name,
            &mut client,
            &mut update_received
        ).await {
            Ok(_) => return Ok(()),
            Err(err) => {
                if first_session && !update_received {
                    return Err(err)
                } else {
                    error!(err =? err, "data source failure");
                }
                first_session = false;
                if update_received {
                    errors = 1;
                } else {
                    errors += 1;
                }
                let pause = backoff_ms[errors.min(backoff_ms.len() - 1)];
                if pause > 0 {
                    info!("will pause data source for {} ms", pause);
                    tokio::time::sleep(Duration::from_millis(pause)).await;
                }
            }
        }
    }
    Ok(())
}


async fn source_session(
    output: &tokio::sync::mpsc::Sender<SourceMessage>,
    name: Name,
    client: &mut GeyserClient,
    update_received: &mut bool
) -> anyhow::Result<()>
{
    let req = SubscribeRequest {
        transactions: HashMap::from([
            (
                "transactions".to_string(),
                SubscribeRequestFilterTransactions {
                    vote: Some(false),
                    ..SubscribeRequestFilterTransactions::default()
                }
            )
        ]),
        blocks_meta: HashMap::from([
            (
                "blocks".to_string(),
                SubscribeRequestFilterBlocksMeta::default()
            )
        ]),
        commitment: Some(CommitmentLevel::Processed as i32),
        ..SubscribeRequest::default()
    };

    let mut updates = client.subscribe(tokio_stream::once(req))
        .await?
        .into_inner();

    debug!("subscribed to updates");

    while let Some(upd) = updates.try_next().await? {
        if let Some(upd) = upd.update_oneof {
            let update = match upd {
                UpdateOneof::Transaction(tx) => {
                    match TransactionUpdate::from_subscription_update(tx) {
                        Ok(tx) => {
                            debug!(
                                slot = tx.slot,
                                transaction_index = tx.index,
                                "received"
                            );
                            SourceUpdate::Transaction(tx)
                        },
                        Err(missing_field) => {
                            bail!("got transaction update with missing {} field", missing_field)
                        },
                    }
                },
                UpdateOneof::BlockMeta(block) => {
                    debug!(
                        slot = block.slot,
                        block_time =% chrono::DateTime::from_timestamp(
                            block.block_time.map_or(0, |t| t.timestamp), 
                            0
                        ).unwrap(),
                        "published"
                    );
                    SourceUpdate::Block(block)
                },
                _ => continue
            };
            
            *update_received = true;
            
            let msg = SourceMessage {
                source: name,
                update
            };
            
            if output.send(msg).await.is_err() {
                return Ok(())
            }
        }
    }

    bail!("unexpected end of update stream")
}
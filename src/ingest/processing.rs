use super::mapping::map_transaction;
use super::source::{SourceMessage, SourceUpdate};
use crate::data::{BlockData, DataMessage};
use std::pin::pin;
use std::sync::Arc;
use tokio_stream::{Stream, StreamExt};
use tracing::{debug, error};


pub type Broadcast = tokio::sync::broadcast::Sender<Arc<DataMessage>>;


pub async fn processing_loop(
    broadcast: Broadcast,
    input: impl Stream<Item = SourceMessage>
) {
    let input = dedupe(input);
    let mut input = pin!(input);
    while let Some(msg) = input.next().await {
        let data_msg = match msg.update {
            SourceUpdate::Block(block) => {
                let block = BlockData {
                    slot: block.slot,
                    hash: block.blockhash,
                    parent_slot: block.parent_slot,
                    parent_hash: block.parent_blockhash,
                    height: block.block_height.map(|h| h.block_height),
                    timestamp: block.block_time.map_or(0, |t| t.timestamp)
                };
                debug!(
                    slot = block.slot,
                    block_time =% chrono::DateTime::from_timestamp(block.timestamp, 0).unwrap(),
                    source = msg.source,
                    "published"
                );
                crate::metrics::register_block_publication(
                    msg.source,
                    block.slot,
                    block.timestamp
                );
                DataMessage::Block(block)
            },
            SourceUpdate::Transaction(tx) => {
                let slot = tx.slot;
                let transaction_index = tx.index;
                match map_transaction(tx) {
                    Ok(tx) => {
                        debug!(
                            slot,
                            transaction_index,
                            source = msg.source,
                            "published"
                        );
                        crate::metrics::register_tx_publication(msg.source);
                        DataMessage::Transaction(tx)
                    },
                    Err(err) => {
                        error!(
                            slot,
                            transaction_index,
                            source = msg.source,
                            err =? err,
                            "failed to map transaction"
                        );
                        crate::metrics::register_mapping_error(msg.source);
                        continue
                    }
                }
            }
        };
        let _ = broadcast.send(Arc::new(data_msg));
    }
}


fn dedupe(input: impl Stream<Item = SourceMessage>) -> impl Stream<Item = SourceMessage> {
    let mut slot = 0;
    let mut received_transactions = Mask::new(5000);
    input.filter_map(move |msg| {
        match &msg.update {
            SourceUpdate::Block(block) => {
                if block.slot >= slot {
                    slot = block.slot + 1;
                    received_transactions.reset();
                    Some(msg)
                } else {
                    None
                }
            },
            SourceUpdate::Transaction(tx) => {
                if tx.slot > slot {
                    slot = tx.slot;
                    received_transactions.reset();
                }
                if tx.slot == slot && received_transactions.mark(tx.index as usize) {
                    Some(msg)
                } else {
                    None
                }
            }
        }
    })
}


struct Mask {
    inner: Vec<bool>
}


impl Mask {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: vec![false; capacity]
        }
    }

    pub fn mark(&mut self, i: usize) -> bool {
        match self.inner.get_mut(i) {
            Some(is_set) => {
                let set = !*is_set;
                *is_set = true;
                set
            },
            None => {
                let new_len = std::cmp::max(i, self.inner.len() * 2);
                self.inner.resize(new_len, false);
                true
            },
        }
    }

    pub fn reset(&mut self) {
        self.inner.fill(false)
    }
}
use crate::data::{DataMessage, JsonString};
use crate::ingest::Broadcast;
use crate::json_builder::RawJson;
use crate::query::{render_block_message, render_transaction_message, FieldSelection, Filter, SolanaQuery};
use jsonrpsee::types::{ErrorCode, ErrorObject};
use jsonrpsee::{RpcModule, SubscriptionMessage};
use tokio::select;
use tokio::sync::broadcast::error::RecvError;
use tracing::{debug, debug_span, Instrument};


pub fn build_rpc_module(broadcast: Broadcast) -> RpcModule<Broadcast> {
    let mut rpc = RpcModule::new(broadcast);
    rpc.register_subscription_raw(
        "spraySubscribe",
        "sprayNotification",
        "sprayUnsubscribe",
        |params, pending, broadcast, _| {
            let span = debug_span!("subscription", connection_id = pending.connection_id().0);
            let span_guard = span.enter();

            let query = match params.one::<SolanaQuery>() {
                Ok(query) => query,
                Err(err) => {
                    debug!(
                        "invalid query - {}",
                        err.data().map_or("unknown syntax error", |json| json.get())
                    );
                    tokio::spawn(pending.reject(err));
                    return
                }
            };
            
            if let Err(err) = SolanaQuery::validate(&query) {
                let msg = format!("invalid query: {}", err);
                debug!(msg);
                
                let err = ErrorObject::owned::<()>(
                    ErrorCode::InvalidParams.code(),
                    msg,
                    None
                );
                
                tokio::spawn(pending.reject(err));
                return 
            }

            debug!(
                query =% serde_json::to_string(&query).unwrap(),
            );
            
            let mut state = SubscriptionState::new(query);

            drop(span_guard);
            
            tokio::spawn(async move {
                let sink = match pending.accept().await {
                    Ok(sink) => sink,
                    Err(_) => {
                        debug!("closed before acceptance");
                        return
                    }
                };
                
                debug!("accepted");
                let _scope = crate::metrics::register_subscription_scope();

                let mut rx = broadcast.subscribe();
                loop {
                    select! {
                        biased;
                        _ = sink.closed() => {
                            debug!("closed");
                            return
                        },
                        event = rx.recv() => {
                            match event {
                                Ok(msg) => {
                                    if let Some(msg) = state.emit(&msg) {
                                        let msg = SubscriptionMessage::new(
                                            sink.method_name(),
                                            sink.subscription_id(),
                                            &RawJson::new(&msg)
                                        ).expect(
                                            "serialization is infallible"
                                        );
                                        if sink.send(msg).await.is_err() {
                                            debug!("closed");
                                            return
                                        }
                                    }
                                },
                                Err(RecvError::Lagged(skipped)) => {
                                    debug!(skipped = skipped, "lagging behind");
                                    continue
                                },
                                Err(RecvError::Closed) => {
                                    debug!("terminating");
                                    let eof = SubscriptionMessage::new(
                                        sink.method_name(),
                                        sink.subscription_id(),
                                        &serde_json::value::Value::Null
                                    ).expect(
                                        "serialization is infallible"
                                    );
                                    let _ = sink.send(eof).await;
                                    return
                                }
                            }
                        }
                    }
                }
            }.instrument(span));
        }
    ).unwrap();
    rpc
}


struct SubscriptionState {
    fields: FieldSelection,
    filter: Filter,
    include_all_blocks: bool,
    last_emitted_block: u64,
    last_non_empty_block: u64
}


impl SubscriptionState {
    fn new(query: SolanaQuery) -> Self {
        Self {
            fields: query.fields.clone(),
            include_all_blocks: query.include_all_blocks,
            filter: Filter::compile(query),
            last_emitted_block: 0,
            last_non_empty_block: 0
        }
    }
    
    fn emit(&mut self, msg: &DataMessage) -> Option<JsonString> {
        match msg {
            DataMessage::Block(block) => {
                if self.include_all_blocks 
                    || self.last_emitted_block + 5 <= block.slot 
                    || self.last_non_empty_block == block.slot 
                {
                    self.last_emitted_block = block.slot;
                    Some(render_block_message(&self.fields.block, block))
                } else {
                    None
                }
            },
            DataMessage::Transaction(tx) => {
                let selection = self.filter.eval(tx);
                if selection.is_empty() {
                    None
                } else {
                    self.last_non_empty_block = tx.slot;
                    Some(render_transaction_message(&self.fields, tx, &selection))   
                }
            }
        }
    }
}
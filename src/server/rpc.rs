use crate::ingest::Broadcast;
use crate::query::SolanaQuery;
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee::{RpcModule, SubscriptionMessage};
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use tokio::select;
use tokio::sync::broadcast::error::RecvError;
use tracing::{debug, debug_span, Instrument};


pub fn build_rpc_module(broadcast: Broadcast) -> RpcModule<Broadcast> {
    let mut rpc = RpcModule::new(broadcast);
    rpc.register_subscription_raw(
        "solanaSubscribe",
        "solanaNotification",
        "solanaUnsubscribe",
        |params, pending, broadcast, _| {
            let span = debug_span!("subscription", connection_id = pending.connection_id().0);

            let query = match params.one::<SolanaQuery>() {
                Ok(query) => query,
                Err(err) => {
                    let span_ = span.enter();
                    debug!(
                        "invalid query - {}",
                        err.data().map_or("unknown syntax error", |json| json.get())
                    );
                    tokio::spawn(pending.reject(err));
                    return
                }
            };

            tokio::spawn(async move {
                let sink = match pending.accept().await {
                    Ok(sink) => sink,
                    Err(_) => {
                        debug!("closed before acceptance");
                        return
                    }
                };
                debug!(
                    query =% serde_json::to_string(&query).unwrap(),
                    "accepted"
                );
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
                                    let msg = SubscriptionMessage::new(
                                        sink.method_name(),
                                        sink.subscription_id(),
                                        &RawJson::new("{}")
                                    ).expect(
                                        "serialization is infallible"
                                    );
                                    if sink.send(msg).await.is_err() {
                                        debug!("closed");
                                        return
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


struct RawJson<'a> {
    json: &'a str
}


impl<'a> RawJson<'a> {
    pub fn new(json: &'a str) -> Self {
        Self {
            json
        }
    }
}


impl<'a> Serialize for RawJson<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("$serde_json::private::RawValue", 1)?;
        s.serialize_field("$serde_json::private::RawValue", self.json)?;
        s.end()
    }
}
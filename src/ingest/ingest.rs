use super::processing::{processing_loop, Broadcast};
use super::source::{source_loop, SourceMessage, SourceUpdate};
use crate::data::{BlockData, DataMessage};
use crate::geyser::GeyserClient;
use crate::Name;
use anyhow::{anyhow, bail};
use std::pin::{pin, Pin};
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tracing::debug;


pub struct Ingest {
    sources: Vec<(Name, GeyserClient)>
}


impl Ingest {
    pub fn new() -> Self {
        Self {
            sources: Vec::new()
        }
    }
    
    pub fn add_source(&mut self, name: Name, client: GeyserClient) {
        self.sources.push((name, client))
    }
    
    pub fn start(self, broadcast: Broadcast) -> IngestHandle {
        let (source_tx, source_rx) = tokio::sync::mpsc::channel::<SourceMessage>(20_000);

        let sources: Vec<_> = self.sources.into_iter().map(|(name, client)| {
            let task = tokio::spawn(
                source_loop(source_tx.clone(), name, client)
            );
            (name, false, task)
        }).collect();

        let processing = tokio::spawn(
            processing_loop(
                broadcast,
                ReceiverStream::new(source_rx)
            )
        );
        
        IngestHandle {
            terminated: false,
            sources,
            processing
        }
    }
}


pub struct IngestHandle {
    terminated: bool,
    sources: Vec<(Name, bool, JoinHandle<anyhow::Result<()>>)>,
    processing: JoinHandle<()>
}


impl Future for IngestHandle {
    type Output = anyhow::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.terminated {
            return Poll::Ready(Err(anyhow!("already terminated")))
        }
        
        for (name, terminated, handle) in self.sources.iter_mut() {
            let name = *name;
            if let Poll::Ready(res) = pin!(handle).poll(cx) {
                match res {
                    Ok(Ok(_)) => {
                        *terminated = true;
                    },
                    Ok(Err(err)) => {
                        self.abort();
                        return Poll::Ready(Err(
                            err.context(format!("{} data ingestion failed", name))
                        ))
                    },
                    Err(join_error) => {
                        self.abort();
                        return Poll::Ready(Err(
                            anyhow!(
                                "{} data ingestion terminated: {:?}",
                                name,
                                join_error
                            )
                        ))
                    }
                }
            }
        }
        
        self.sources.retain(|(_, is_terminated, _)| !*is_terminated);

        if let Poll::Ready(res) = pin!(&mut self.processing).poll(cx) {
            return match res {
                Ok(_) => {
                    self.abort();
                    Poll::Ready(Ok(()))
                },
                Err(join_error) => {
                    self.abort();
                    Poll::Ready(Err(
                        anyhow!("data processing task failed: {:?}", join_error)
                    ))
                }
            }
        }
        
        Poll::Pending
    }
}


impl IngestHandle {
    pub fn abort(&mut self) {
        if self.terminated {
            return;
        }
        self.terminated = true;
        self.processing.abort();
        for (_, _, handle) in self.sources.iter() {
            handle.abort();
        }
    }
}


impl Drop for IngestHandle {
    fn drop(&mut self) {
        self.abort()
    }
}
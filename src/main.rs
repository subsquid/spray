#![allow(unused)]
mod cli;
mod config;
mod data;
mod geyser;
mod ingest;
mod json_builder;
mod server;
mod query;


use crate::cli::CLI;
use crate::config::Config;
use crate::geyser::create_geyser_client;
use crate::ingest::{Broadcast, Ingest};
use anyhow::{ensure, Context};
use clap::Parser;


#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;


fn main() -> anyhow::Result<()> {
    let args = CLI::parse();
    let cfg = Config::read(args.config).context("failed to read config file")?;
    
    ensure!(!cfg.sources.is_empty(), "no data source was specified in config file");
    
    init_tracing();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(
            run(cfg)
        )
}


async fn run(cfg: Config) -> anyhow::Result<()> {
    let broadcast = Broadcast::new(20_000);
    
    let ingest = {
        let mut ingest = Ingest::new();
        for (name, src) in cfg.sources {
            let name: Name = name.leak();
            let client = create_geyser_client(src).await.with_context(|| {
                format!("{} connection failed", name)
            })?;
            ingest.add_source(name, client);
        }
        ingest.start(broadcast.clone())
    };
    
    ingest.await?;
    
    Ok(())
}


fn init_tracing() {
    use std::io::IsTerminal;
    
    let env_filter = tracing_subscriber::EnvFilter::builder().parse_lossy(
        std::env::var(tracing_subscriber::EnvFilter::DEFAULT_ENV)
            .unwrap_or("info".to_string()),
    );

    if std::io::stdout().is_terminal() {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .compact()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .with_current_span(false)
            .init();
    }
}


pub type Name = &'static str;
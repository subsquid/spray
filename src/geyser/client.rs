use super::api;
use super::auth_interceptor::AuthInterceptor;
use crate::config::GeyserConfig;
use anyhow::Context;
use tonic::codec::CompressionEncoding;
use tonic::codegen::InterceptedService;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};


pub type GeyserClient = api::geyser_client::GeyserClient<InterceptedService<Channel, AuthInterceptor>>;


pub async fn create_geyser_client(cfg: GeyserConfig) -> anyhow::Result<GeyserClient> {
    let channel = Endpoint::from(cfg.url)
        .tls_config(ClientTlsConfig::new().with_native_roots())
        .context("failed to configure TLS")?
        .connect()
        .await?;

    let auth = AuthInterceptor {
        x_token: cfg.x_token,
        x_access_token: cfg.x_access_token
    };

    let client = api::geyser_client::GeyserClient::with_interceptor(channel, auth)
        .max_decoding_message_size(32 * 1024 * 1024)
        .accept_compressed(CompressionEncoding::Zstd);

    Ok(client)
}
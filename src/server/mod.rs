mod rpc;


use self::rpc::build_rpc_module;
use crate::ingest::Broadcast;
use jsonrpsee::server::{Server, ServerConfig, ServerHandle};
use tracing::info;


pub struct RpcServer {
    broadcast: Broadcast,
    config: ServerConfig,
    port: u16
}


impl RpcServer {
    pub fn new(broadcast: Broadcast) -> Self {
        let config = ServerConfig::builder()
            .set_message_buffer_capacity(5)
            .max_response_body_size(4 * 1024 * 1024)
            .max_request_body_size(257 * 1024)
            .build();
        
        Self {
            broadcast,
            config,
            port: 3000
        }
    }
    
    pub fn set_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub async fn start(self) -> anyhow::Result<ServerHandle> {
        let server = Server::builder()
            .set_config(self.config)
            .build(("0.0.0.0", self.port))
            .await?;

        let module = build_rpc_module(self.broadcast);
        
        let addr = server.local_addr()?;
        let handle = server.start(module);
        info!("server is listening on port {}", addr.port());
        Ok(handle)
    }
}
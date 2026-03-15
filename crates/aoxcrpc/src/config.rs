/// RPC subsystem configuration.
#[derive(Debug, Clone)]
pub struct RpcConfig {
    pub http_bind_addr: String,
    pub websocket_bind_addr: String,
    pub grpc_bind_addr: String,
}

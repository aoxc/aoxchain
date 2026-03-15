/// RPC subsystem configuration.
#[derive(Debug, Clone)]
pub struct RpcConfig {
    pub http_bind_addr: String,
    pub websocket_bind_addr: String,
    pub grpc_bind_addr: String,
    pub tls_cert_path: String,
    pub tls_key_path: String,
    pub mtls_ca_cert_path: Option<String>,
    pub max_requests_per_minute: u64,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            http_bind_addr: "127.0.0.1:8080".to_string(),
            websocket_bind_addr: "127.0.0.1:8081".to_string(),
            grpc_bind_addr: "127.0.0.1:50051".to_string(),
            tls_cert_path: "./tls/server.crt".to_string(),
            tls_key_path: "./tls/server.key".to_string(),
            mtls_ca_cert_path: Some("./tls/ca.crt".to_string()),
            max_requests_per_minute: 600,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RpcClient;

impl RpcClient {
    pub fn endpoint() -> &'static str {
        "http://127.0.0.1:8545"
    }
}

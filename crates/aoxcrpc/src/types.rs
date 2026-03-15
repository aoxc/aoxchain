/// Generic health response returned by RPC endpoints.
#[derive(Debug, Clone)]
pub struct HealthResponse {
    pub status: &'static str,
}

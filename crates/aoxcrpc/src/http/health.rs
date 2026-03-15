use crate::types::HealthResponse;

/// Returns the current health status exposed by the HTTP RPC interface.
pub fn health() -> HealthResponse {
    HealthResponse { status: "ok" }
}

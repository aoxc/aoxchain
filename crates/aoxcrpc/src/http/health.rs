use crate::types::HealthResponse;

/// Returns the current health status exposed by the HTTP RPC interface.
#[must_use]
pub fn health() -> HealthResponse {
    HealthResponse { status: "ok" }
}

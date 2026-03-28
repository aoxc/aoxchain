use std::env;

/// Resolves the effective RPC target for AOXCHUB service integrations.
///
/// Operational Policy:
/// - Local development must work out of the box without requiring external AOXC
///   infrastructure.
/// - The default endpoint therefore targets a local JSON-RPC node.
/// - Controlled deployments may override the endpoint through environment
///   variables without modifying application code.
///
/// Resolution Order:
/// 1. `AOXCHUB_RPC_ENDPOINT`
/// 2. `AOXC_RPC_ENDPOINT`
/// 3. `http://127.0.0.1:8545`
#[derive(Debug, Clone, Copy)]
pub struct RpcClient;

impl RpcClient {
    /// Returns the effective JSON-RPC endpoint used by AOXCHUB services.
    ///
    /// The function applies a local-first policy so the desktop control plane
    /// remains usable in development environments even when public AOXC RPC
    /// infrastructure is unavailable.
    pub fn endpoint() -> String {
        Self::endpoint_from_env()
            .unwrap_or_else(|| "http://127.0.0.1:8545".to_string())
    }

    /// Returns a human-readable descriptor suitable for diagnostics or UI display.
    pub fn descriptor() -> String {
        format!("AOXCHUB RPC endpoint ({})", Self::endpoint())
    }

    fn endpoint_from_env() -> Option<String> {
        for key in ["AOXCHUB_RPC_ENDPOINT", "AOXC_RPC_ENDPOINT"] {
            if let Ok(value) = env::var(key) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }

        None
    }
}

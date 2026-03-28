use serde::Deserialize;
use serde_json::json;

use crate::services::rpc_client::RpcClient;

/// Represents the most recent control-plane telemetry snapshot derived from the
/// chain-access layer.
///
/// Security and Reliability Notes:
/// - The snapshot is intentionally minimal and deterministic.
/// - Health is inferred from a successful JSON-RPC round-trip and the presence
///   of a valid block number result.
/// - The source field always reflects the effective RPC endpoint used by the probe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetrySnapshot {
    pub healthy: bool,
    pub source: String,
    pub latest_block: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcSuccessResponse {
    result: String,
}

#[derive(Debug, Deserialize)]
struct JsonRpcErrorObject {
    code: i64,
    message: String,
}

#[derive(Debug, Deserialize)]
struct JsonRpcErrorResponse {
    error: JsonRpcErrorObject,
}

/// Retrieves the latest telemetry snapshot from the configured chain RPC.
///
/// Operational Behavior:
/// - Executes a lightweight `eth_blockNumber` probe against the configured endpoint.
/// - Marks the snapshot as healthy only when the endpoint returns a valid hex block number.
/// - Treats transport failures, malformed payloads, and RPC error objects as unhealthy.
///
/// Return Model:
/// - This function never propagates an error to callers.
/// - Failures are collapsed into a deterministic unhealthy snapshot so that UI surfaces
///   can render safely without introducing control-flow instability.
pub async fn latest_snapshot() -> TelemetrySnapshot {
    let endpoint = RpcClient::endpoint().to_string();

    let client = match reqwest::Client::builder().build() {
        Ok(client) => client,
        Err(_) => {
            return TelemetrySnapshot {
                healthy: false,
                source: endpoint,
                latest_block: None,
            };
        }
    };

    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });

    let response = match client
        .post(&endpoint)
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return TelemetrySnapshot {
                healthy: false,
                source: endpoint,
                latest_block: None,
            };
        }
    };

    if !response.status().is_success() {
        return TelemetrySnapshot {
            healthy: false,
            source: endpoint,
            latest_block: None,
        };
    }

    let body = match response.text().await {
        Ok(body) => body,
        Err(_) => {
            return TelemetrySnapshot {
                healthy: false,
                source: endpoint,
                latest_block: None,
            };
        }
    };

    if let Ok(success) = serde_json::from_str::<JsonRpcSuccessResponse>(&body) {
        let latest_block = success
            .result
            .strip_prefix("0x")
            .and_then(|value| u64::from_str_radix(value, 16).ok());

        return TelemetrySnapshot {
            healthy: latest_block.is_some(),
            source: endpoint,
            latest_block,
        };
    }

    if let Ok(error_response) = serde_json::from_str::<JsonRpcErrorResponse>(&body) {
        let _error_code = error_response.error.code;
        let _error_message = error_response.error.message;

        return TelemetrySnapshot {
            healthy: false,
            source: endpoint,
            latest_block: None,
        };
    }

    TelemetrySnapshot {
        healthy: false,
        source: endpoint,
        latest_block: None,
    }
}

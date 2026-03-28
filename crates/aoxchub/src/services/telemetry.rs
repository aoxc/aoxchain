use serde::Deserialize;
use serde_json::json;

use crate::services::rpc_client::RpcClient;

/// Represents the most recent control-plane telemetry snapshot derived from the
/// chain-access layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetrySnapshot {
    pub healthy: bool,
    pub source: String,
    pub latest_block: Option<u64>,
    pub peer_count: Option<usize>,
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
pub async fn latest_snapshot() -> TelemetrySnapshot {
    let endpoint = RpcClient::endpoint().to_string();

    let client = match reqwest::Client::builder().build() {
        Ok(client) => client,
        Err(_) => {
            return TelemetrySnapshot {
                healthy: false,
                source: endpoint,
                latest_block: None,
                peer_count: None,
            };
        }
    };

    let payload = json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });

    let response = match client.post(&endpoint).json(&payload).send().await {
        Ok(response) => response,
        Err(_) => {
            return TelemetrySnapshot {
                healthy: false,
                source: endpoint,
                latest_block: None,
                peer_count: None,
            };
        }
    };

    if !response.status().is_success() {
        return TelemetrySnapshot {
            healthy: false,
            source: endpoint,
            latest_block: None,
            peer_count: None,
        };
    }

    let body = match response.text().await {
        Ok(body) => body,
        Err(_) => {
            return TelemetrySnapshot {
                healthy: false,
                source: endpoint,
                latest_block: None,
                peer_count: None,
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
            source: endpoint.clone(),
            latest_block,
            peer_count: latest_peer_count(&client, &endpoint).await,
        };
    }

    if let Ok(error_response) = serde_json::from_str::<JsonRpcErrorResponse>(&body) {
        let _error_code = error_response.error.code;
        let _error_message = error_response.error.message;

        return TelemetrySnapshot {
            healthy: false,
            source: endpoint,
            latest_block: None,
            peer_count: None,
        };
    }

    TelemetrySnapshot {
        healthy: false,
        source: endpoint,
        latest_block: None,
        peer_count: None,
    }
}

async fn latest_peer_count(client: &reqwest::Client, endpoint: &str) -> Option<usize> {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "net_peerCount",
        "params": [],
        "id": 2
    });

    let response = client.post(endpoint).json(&payload).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }

    let body = response.text().await.ok()?;
    let success = serde_json::from_str::<JsonRpcSuccessResponse>(&body).ok()?;

    success
        .result
        .strip_prefix("0x")
        .and_then(|value| usize::from_str_radix(value, 16).ok())
}

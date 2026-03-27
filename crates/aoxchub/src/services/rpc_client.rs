use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub chain_id: String,
    pub readiness_score: u8,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Default)]
pub struct MetricsSnapshot {
    pub requests_total: f64,
    pub rejected_total: f64,
    pub rate_limited_total: f64,
    pub readiness_score: f64,
}

#[derive(Debug, Clone)]
pub struct ChainSnapshot {
    pub chain_id_hex: String,
    pub block_height: u64,
    pub health: HealthResponse,
    pub metrics: MetricsSnapshot,
}

#[derive(Debug, Clone)]
pub struct RpcClient {
    pub api_base: String,
    pub rpc_base: String,
    pub ws_base: String,
    http: Client,
}

impl RpcClient {
    pub fn from_env() -> Self {
        let api_base = env::var("AOXHUB_API_BASE")
            .unwrap_or_else(|_| "http://127.0.0.1:2626".to_string())
            .trim_end_matches('/')
            .to_string();
        let rpc_base = env::var("AOXHUB_RPC_BASE")
            .unwrap_or_else(|_| format!("{api_base}/rpc/v1"))
            .trim_end_matches('/')
            .to_string();
        let ws_base =
            env::var("AOXHUB_WS_BASE").unwrap_or_else(|_| "ws://127.0.0.1:3030/ws/v1".to_string());

        Self {
            api_base,
            rpc_base,
            ws_base,
            http: Client::new(),
        }
    }

    pub async fn fetch_dashboard(&self) -> Result<ChainSnapshot, String> {
        let health = self.fetch_health().await?;
        let chain_id_hex = self.eth_chain_id().await?;
        let block_height = self.eth_block_number().await?;
        let metrics = self.fetch_metrics().await.unwrap_or_default();

        Ok(ChainSnapshot {
            chain_id_hex,
            block_height,
            health,
            metrics,
        })
    }

    pub async fn fetch_health(&self) -> Result<HealthResponse, String> {
        let preferred = format!("{}/api/v1/health", self.api_base);
        let fallback = format!("{}/health", self.api_base);

        let response = self.get_with_fallback(&preferred, &fallback).await?;

        response
            .error_for_status()
            .map_err(|e| format!("health endpoint HTTP hatası: {e}"))?
            .json::<HealthResponse>()
            .await
            .map_err(|e| format!("health JSON çözümlenemedi: {e}"))
    }

    pub async fn fetch_metrics(&self) -> Result<MetricsSnapshot, String> {
        let preferred = format!("{}/api/v1/metrics", self.api_base);
        let fallback = format!("{}/metrics", self.api_base);

        let text = self
            .get_with_fallback(&preferred, &fallback)
            .await?
            .error_for_status()
            .map_err(|e| format!("metrics endpoint HTTP hatası: {e}"))?
            .text()
            .await
            .map_err(|e| format!("metrics body okunamadı: {e}"))?;

        Ok(MetricsSnapshot {
            requests_total: parse_metric(&text, "aox_rpc_requests_total"),
            rejected_total: parse_metric(&text, "aox_rpc_rejected_total"),
            rate_limited_total: parse_metric(&text, "aox_rpc_rate_limited_total"),
            readiness_score: parse_metric(&text, "aox_rpc_health_readiness_score"),
        })
    }

    pub async fn eth_chain_id(&self) -> Result<String, String> {
        self.rpc_call_text("eth_chainId", json!([])).await
    }

    pub async fn eth_block_number(&self) -> Result<u64, String> {
        let hex = self.rpc_call_text("eth_blockNumber", json!([])).await?;
        parse_hex_u64(&hex)
    }

    pub async fn eth_get_transaction_receipt(&self, tx_hash: &str) -> Result<Value, String> {
        self.rpc_call_value("eth_getTransactionReceipt", json!([tx_hash]))
            .await
    }

    pub async fn eth_call(&self, to: &str, data: &str) -> Result<String, String> {
        self.rpc_call_text("eth_call", json!([{ "to": to, "data": data }, "latest"]))
            .await
    }

    pub async fn eth_estimate_gas(&self, to: &str, data: &str) -> Result<String, String> {
        self.rpc_call_text("eth_estimateGas", json!([{ "to": to, "data": data }]))
            .await
    }

    pub async fn submit_zkp_tx(
        &self,
        actor_id: &str,
        tx_payload: Vec<u8>,
        zkp_proof: Vec<u8>,
    ) -> Result<Value, String> {
        let submit_url = format!("{}/api/v1/tx/submit", self.api_base);
        self.http
            .post(submit_url)
            .json(&json!({
                "actor_id": actor_id,
                "tx_payload": tx_payload,
                "zkp_proof": zkp_proof,
            }))
            .send()
            .await
            .map_err(|e| format!("Submit endpoint erişilemedi: {e}"))?
            .error_for_status()
            .map_err(|e| format!("Submit HTTP hatası: {e}"))?
            .json::<Value>()
            .await
            .map_err(|e| format!("Submit JSON cevabı çözümlenemedi: {e}"))
    }

    async fn get_with_fallback(
        &self,
        preferred: &str,
        fallback: &str,
    ) -> Result<reqwest::Response, String> {
        match self.http.get(preferred).send().await {
            Ok(response) => Ok(response),
            Err(primary_err) => self
                .http
                .get(fallback)
                .send()
                .await
                .map_err(|fallback_err| {
                    format!(
                        "endpoint erişilemedi (primary: {primary_err}; fallback: {fallback_err})"
                    )
                }),
        }
    }

    pub async fn rpc_call_value(&self, method: &str, params: Value) -> Result<Value, String> {
        let payload = RpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method,
            params,
        };

        let envelope = self
            .http
            .post(&self.rpc_base)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("RPC endpoint erişilemedi: {e}"))?
            .error_for_status()
            .map_err(|e| format!("RPC HTTP hatası: {e}"))?
            .json::<RpcResponse<Value>>()
            .await
            .map_err(|e| format!("RPC cevabı JSON çözümlenemedi: {e}"))?;

        if let Some(err) = envelope.error {
            return Err(format!("{}: {}", err.code, err.message));
        }

        envelope
            .result
            .ok_or_else(|| "RPC result alanı boş geldi".to_string())
    }

    pub async fn rpc_call_text(&self, method: &str, params: Value) -> Result<String, String> {
        let value = self.rpc_call_value(method, params).await?;
        value
            .as_str()
            .map(ToString::to_string)
            .ok_or_else(|| format!("{method} sonucu string değil"))
    }
}

#[derive(Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'a str,
    id: u64,
    method: &'a str,
    params: Value,
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

fn parse_metric(body: &str, metric: &str) -> f64 {
    body.lines()
        .find_map(|line| {
            if line.starts_with(metric) {
                line.split_whitespace().last()?.parse::<f64>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0.0)
}

fn parse_hex_u64(value: &str) -> Result<u64, String> {
    let raw = value.trim_start_matches("0x");
    u64::from_str_radix(raw, 16).map_err(|e| format!("hex sayı parse edilemedi ({value}): {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn parse_metric_extracts_numeric_value() {
        let payload = "aox_rpc_requests_total 42\n";
        assert_eq!(parse_metric(payload, "aox_rpc_requests_total"), 42.0);
    }

    #[test]
    fn parse_metric_returns_zero_when_missing() {
        let payload = "# HELP aox_rpc_requests_total Requests\n";
        assert_eq!(parse_metric(payload, "aox_rpc_rejected_total"), 0.0);
    }

    #[test]
    fn parse_metric_ignores_comments_and_picks_actual_row() {
        let payload =
            "# TYPE aox_rpc_health_readiness_score gauge\naox_rpc_health_readiness_score 98\n";
        assert_eq!(
            parse_metric(payload, "aox_rpc_health_readiness_score"),
            98.0
        );
    }

    #[test]
    fn parse_hex_u64_accepts_prefixed_hex() {
        assert_eq!(parse_hex_u64("0x2a").expect("valid hex"), 42);
    }

    #[test]
    fn parse_hex_u64_accepts_plain_hex() {
        assert_eq!(parse_hex_u64("2a").expect("valid hex"), 42);
    }

    #[test]
    fn parse_hex_u64_rejects_invalid_hex() {
        let err = parse_hex_u64("0xzz").expect_err("invalid hex should fail");
        assert!(err.contains("hex sayı parse edilemedi"));
    }

    #[test]
    fn from_env_uses_defaults_when_env_is_missing() {
        let _guard = env_lock().lock().expect("env mutex poisoned");

        std::env::remove_var("AOXHUB_API_BASE");
        std::env::remove_var("AOXHUB_RPC_BASE");
        std::env::remove_var("AOXHUB_WS_BASE");

        let client = RpcClient::from_env();
        assert_eq!(client.api_base, "http://127.0.0.1:2626");
        assert_eq!(client.rpc_base, "http://127.0.0.1:2626/rpc/v1");
        assert_eq!(client.ws_base, "ws://127.0.0.1:3030/ws/v1");
    }

    #[test]
    fn from_env_respects_overrides_and_trims_trailing_slash() {
        let _guard = env_lock().lock().expect("env mutex poisoned");

        std::env::set_var("AOXHUB_API_BASE", "https://api.example.com/");
        std::env::set_var("AOXHUB_RPC_BASE", "https://api.example.com/rpc/custom/");
        std::env::set_var("AOXHUB_WS_BASE", "wss://ws.example.com/ws/v1");

        let client = RpcClient::from_env();
        assert_eq!(client.api_base, "https://api.example.com");
        assert_eq!(client.rpc_base, "https://api.example.com/rpc/custom");
        assert_eq!(client.ws_base, "wss://ws.example.com/ws/v1");

        std::env::remove_var("AOXHUB_API_BASE");
        std::env::remove_var("AOXHUB_RPC_BASE");
        std::env::remove_var("AOXHUB_WS_BASE");
    }
}

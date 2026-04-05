// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::config::RpcConfig;
use crate::contracts::ContractHttpApi;
use crate::http::{health::health_with_context, metrics::prometheus_metrics_snapshot};
use crate::types::RpcErrorResponse;

#[derive(Debug, Clone)]
pub struct HttpRpcResponse {
    pub status: u16,
    pub content_type: &'static str,
    pub body: String,
}

#[derive(Debug, Default)]
pub struct HttpRpcServer {
    pub config: RpcConfig,
    pub contract_api: ContractHttpApi,
    pub uptime_secs: u64,
    pub total_requests: u64,
    pub rejected_requests: u64,
    pub rate_limited_requests: u64,
    pub active_rate_limiter_keys: u64,
}

impl HttpRpcServer {
    #[must_use]
    pub fn new(config: RpcConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    pub fn handle_json(
        &mut self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Result<HttpRpcResponse, HttpRpcResponse> {
        self.total_requests = self.total_requests.saturating_add(1);

        match (method, path) {
            ("GET", "/health") => {
                self.ok_json(&health_with_context(&self.config, self.uptime_secs))
            }
            ("GET", "/metrics") => Ok(HttpRpcResponse {
                status: 200,
                content_type: "text/plain; version=0.0.4",
                body: prometheus_metrics_snapshot(
                    self.total_requests,
                    self.rejected_requests,
                    self.rate_limited_requests,
                    self.active_rate_limiter_keys,
                    health_with_context(&self.config, self.uptime_secs).readiness_score,
                ),
            }),
            ("GET", "/quantum/profile") => {
                self.ok_json(&crate::http::quantum::quantum_crypto_profile())
            }
            ("GET", "/quantum/profile/full") => {
                self.ok_json(&crate::http::quantum::quantum_full_profile())
            }
            ("GET", "/quantum/posture/runtime") => {
                self.ok_json(&crate::http::quantum::quantum_runtime_posture(
                    &self.config,
                    self.uptime_secs,
                    self.total_requests,
                    self.rejected_requests,
                    self.rate_limited_requests,
                    self.active_rate_limiter_keys,
                ))
            }
            ("POST", "/contracts/validate") => {
                self.contract_post(body, |api, request| api.validate_manifest(request))
            }
            ("POST", "/contracts/register") => {
                self.contract_post(body, |api, request| api.register_contract(request))
            }
            ("POST", "/contracts/get") => {
                self.contract_post(body, |api, request| api.get_contract(request))
            }
            ("POST", "/contracts/list") => {
                self.contract_post(body, |api, request| api.list_contracts(request))
            }
            ("POST", "/contracts/activate") => {
                self.contract_post(body, |api, request| api.activate_contract(request))
            }
            ("POST", "/contracts/deprecate") => {
                self.contract_post(body, |api, request| api.deprecate_contract(request))
            }
            ("POST", "/contracts/revoke") => {
                self.contract_post(body, |api, request| api.revoke_contract(request))
            }
            ("POST", "/contracts/runtime-binding") => {
                self.contract_post(body, |api, request| api.resolve_runtime_binding(request))
            }
            _ => Err(self.error_response(
                404,
                RpcErrorResponse {
                    code: "METHOD_NOT_FOUND",
                    message: format!("unsupported route {method} {path}"),
                    retry_after_ms: None,
                    request_id: None,
                    user_hint: Some("Use a supported HTTP method and API route.".to_string()),
                },
            )),
        }
    }

    fn contract_post<T, R>(
        &mut self,
        body: Option<&str>,
        handler: impl FnOnce(&mut ContractHttpApi, T) -> Result<R, crate::contracts::ContractRpcError>,
    ) -> Result<HttpRpcResponse, HttpRpcResponse>
    where
        T: serde::de::DeserializeOwned,
        R: serde::Serialize,
    {
        let Some(body) = body else {
            self.rejected_requests = self.rejected_requests.saturating_add(1);
            return Err(self.error_response(400, parse_error("request body is required")));
        };

        let request: T = match serde_json::from_str(body) {
            Ok(value) => value,
            Err(error) => {
                self.rejected_requests = self.rejected_requests.saturating_add(1);
                return Err(self.error_response(400, parse_error(&error.to_string())));
            }
        };

        match handler(&mut self.contract_api, request) {
            Ok(response) => self.ok_json(&response),
            Err(error) => {
                self.rejected_requests = self.rejected_requests.saturating_add(1);
                Err(self.error_response(
                    error.http_status(),
                    error.to_response("http-contract".to_string()),
                ))
            }
        }
    }

    fn ok_json(&self, payload: &impl serde::Serialize) -> Result<HttpRpcResponse, HttpRpcResponse> {
        match serde_json::to_string(payload) {
            Ok(body) => Ok(HttpRpcResponse {
                status: 200,
                content_type: "application/json",
                body,
            }),
            Err(_) => Err(self.error_response(
                500,
                RpcErrorResponse {
                    code: "INTERNAL_ERROR",
                    message: "failed to serialize response payload".to_string(),
                    retry_after_ms: None,
                    request_id: None,
                    user_hint: None,
                },
            )),
        }
    }

    fn error_response(&self, status: u16, error: RpcErrorResponse) -> HttpRpcResponse {
        HttpRpcResponse {
            status,
            content_type: "application/json",
            body: serde_json::to_string(&error).unwrap_or_else(|_| {
                "{\"code\":\"INTERNAL_ERROR\",\"message\":\"error serialization failed\"}".into()
            }),
        }
    }
}

fn parse_error(message: &str) -> RpcErrorResponse {
    RpcErrorResponse {
        code: "INVALID_REQUEST",
        message: format!("invalid JSON request: {message}"),
        retry_after_ms: None,
        request_id: None,
        user_hint: Some("Ensure body is valid JSON and matches route schema.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_route_returns_json() {
        let mut server = HttpRpcServer::default();
        let response = server
            .handle_json("GET", "/health", None)
            .expect("health route should return success");
        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "application/json");
        assert!(response.body.contains("\"status\""));
    }

    #[test]
    fn unsupported_route_returns_method_not_found() {
        let mut server = HttpRpcServer::default();
        let response = server
            .handle_json("PATCH", "/unknown", None)
            .expect_err("unknown route should be rejected");
        assert_eq!(response.status, 404);
        assert!(response.body.contains("METHOD_NOT_FOUND"));
    }

    #[test]
    fn quantum_full_profile_route_returns_json() {
        let mut server = HttpRpcServer::default();
        let response = server
            .handle_json("GET", "/quantum/profile/full", None)
            .expect("quantum full profile route should return success");
        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "application/json");
        assert!(response.body.contains("hybrid-post-quantum-hardening"));
    }

    #[test]
    fn quantum_runtime_posture_route_returns_runtime_fields() {
        let mut server = HttpRpcServer::default();
        server.uptime_secs = 77;
        server.total_requests = 3;

        let response = server
            .handle_json("GET", "/quantum/posture/runtime", None)
            .expect("runtime posture route should return success");

        assert_eq!(response.status, 200);
        assert_eq!(response.content_type, "application/json");
        assert!(response.body.contains("\"runtime_counters\""));
        assert!(response.body.contains("\"chain_id\""));
    }
}

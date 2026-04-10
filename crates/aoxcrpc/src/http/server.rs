// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::config::RpcConfig;
use crate::contracts::ContractHttpApi;
use crate::error::RpcError;
use crate::http::{health::health_with_context, metrics::prometheus_metrics_snapshot};
use crate::middleware::{mtls_auth::MtlsPolicy, rate_limiter::RateLimiter};
use crate::types::RpcErrorResponse;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HttpRpcResponse {
    pub status: u16,
    pub content_type: &'static str,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct HttpRequestContext {
    pub request_id: Option<String>,
    pub client_key: String,
    pub content_type: Option<String>,
    pub mtls_fingerprint: Option<String>,
}

impl Default for HttpRequestContext {
    fn default() -> Self {
        Self {
            request_id: None,
            client_key: "anonymous".to_string(),
            content_type: None,
            mtls_fingerprint: None,
        }
    }
}

#[derive(Debug)]
pub struct HttpRpcServer {
    pub config: RpcConfig,
    pub contract_api: ContractHttpApi,
    pub uptime_secs: u64,
    pub total_requests: u64,
    pub rejected_requests: u64,
    pub rate_limited_requests: u64,
    pub active_rate_limiter_keys: u64,
    rate_limiter: RateLimiter,
    mtls_policy: Option<MtlsPolicy>,
}

impl Default for HttpRpcServer {
    fn default() -> Self {
        Self::new(RpcConfig::default())
    }
}

impl HttpRpcServer {
    #[must_use]
    pub fn new(config: RpcConfig) -> Self {
        let rate_limiter = RateLimiter::with_limits(
            usize::try_from(config.max_requests_per_minute).unwrap_or(usize::MAX),
            Duration::from_secs(config.rate_limiter_window_secs),
            config.rate_limiter_max_tracked_keys,
        );

        Self {
            config,
            contract_api: ContractHttpApi::default(),
            uptime_secs: 0,
            total_requests: 0,
            rejected_requests: 0,
            rate_limited_requests: 0,
            active_rate_limiter_keys: 0,
            rate_limiter,
            mtls_policy: None,
        }
    }

    pub fn set_mtls_policy(&mut self, policy: Option<MtlsPolicy>) {
        self.mtls_policy = policy;
    }

    pub fn handle_json(
        &mut self,
        method: &str,
        path: &str,
        body: Option<&str>,
    ) -> Result<HttpRpcResponse, HttpRpcResponse> {
        self.handle_json_with_context(method, path, body, HttpRequestContext::default())
    }

    pub fn handle_json_with_context(
        &mut self,
        method: &str,
        path: &str,
        body: Option<&str>,
        context: HttpRequestContext,
    ) -> Result<HttpRpcResponse, HttpRpcResponse> {
        self.total_requests = self.total_requests.saturating_add(1);

        let request_id = context
            .request_id
            .clone()
            .unwrap_or_else(|| format!("req-{}", self.total_requests));

        if let Err(error) = self.rate_limiter.check(&context.client_key) {
            self.rate_limited_requests = self.rate_limited_requests.saturating_add(1);
            self.active_rate_limiter_keys = self.rate_limiter.active_key_count() as u64;
            return Err(self.error_from_rpc_error(429, &request_id, error));
        }
        self.active_rate_limiter_keys = self.rate_limiter.active_key_count() as u64;

        if let Err(error) = self.guard_request(method, path, body, &context) {
            self.rejected_requests = self.rejected_requests.saturating_add(1);
            let status = status_for_guard_error(&error);
            return Err(self.error_from_rpc_error(status, &request_id, error));
        }

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
                    request_id: Some(request_id),
                    user_hint: Some("Use a supported HTTP method and API route.".to_string()),
                },
            )),
        }
    }

    fn guard_request(
        &self,
        method: &str,
        path: &str,
        body: Option<&str>,
        context: &HttpRequestContext,
    ) -> Result<(), RpcError> {
        if method == "POST" {
            match context.content_type.as_deref() {
                Some("application/json") => {}
                _ => return Err(RpcError::InvalidRequest),
            }

            let Some(payload) = body else {
                return Err(RpcError::InvalidRequest);
            };

            if payload.len() > self.config.max_json_body_bytes {
                return Err(RpcError::InvalidRequest);
            }
        }

        if self.requires_mtls(path) {
            let fingerprint = context
                .mtls_fingerprint
                .as_deref()
                .ok_or(RpcError::MtlsAuthFailed)?;

            match &self.mtls_policy {
                Some(policy) => policy.validate_client_fingerprint(fingerprint)?,
                None => return Err(RpcError::MtlsAuthFailed),
            }
        }

        Ok(())
    }

    fn requires_mtls(&self, path: &str) -> bool {
        matches!(
            path,
            "/contracts/register"
                | "/contracts/activate"
                | "/contracts/deprecate"
                | "/contracts/revoke"
        )
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
            return Err(self.error_response(400, parse_error("request body is required", None)));
        };

        let request: T = match serde_json::from_str(body) {
            Ok(value) => value,
            Err(error) => {
                self.rejected_requests = self.rejected_requests.saturating_add(1);
                return Err(self.error_response(400, parse_error(&error.to_string(), None)));
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

    fn error_from_rpc_error(&self, status: u16, request_id: &str, error: RpcError) -> HttpRpcResponse {
        self.error_response(status, error.to_response(Some(request_id.to_string())))
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

fn status_for_guard_error(error: &RpcError) -> u16 {
    match error {
        RpcError::MtlsAuthFailed => 401,
        RpcError::RateLimitExceeded { .. } => 429,
        RpcError::InvalidRequest => 400,
        _ => 403,
    }
}

fn parse_error(message: &str, request_id: Option<String>) -> RpcErrorResponse {
    RpcErrorResponse {
        code: "INVALID_REQUEST",
        message: format!("invalid JSON request: {message}"),
        retry_after_ms: None,
        request_id,
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
    fn post_route_requires_application_json_content_type() {
        let mut server = HttpRpcServer::default();
        let response = server
            .handle_json_with_context(
                "POST",
                "/contracts/get",
                Some("{}"),
                HttpRequestContext::default(),
            )
            .expect_err("missing content-type must be rejected");

        assert_eq!(response.status, 400);
        assert!(response.body.contains("INVALID_REQUEST"));
    }

    #[test]
    fn privileged_route_requires_mtls() {
        let mut server = HttpRpcServer::default();
        let response = server
            .handle_json_with_context(
                "POST",
                "/contracts/register",
                Some("{}"),
                HttpRequestContext {
                    content_type: Some("application/json".to_string()),
                    ..HttpRequestContext::default()
                },
            )
            .expect_err("missing mTLS must be rejected");

        assert_eq!(response.status, 401);
        assert!(response.body.contains("MTLS_AUTH_FAILED"));
    }

    #[test]
    fn rate_limiter_rejects_excessive_requests() {
        let config = RpcConfig {
            max_requests_per_minute: 1,
            ..RpcConfig::default()
        };
        let mut server = HttpRpcServer::new(config);

        let context = HttpRequestContext {
            client_key: "client-a".to_string(),
            ..HttpRequestContext::default()
        };

        let first = server.handle_json_with_context("GET", "/health", None, context.clone());
        assert!(first.is_ok());

        let second = server
            .handle_json_with_context("GET", "/health", None, context)
            .expect_err("second request in same window must be limited");

        assert_eq!(second.status, 429);
        assert!(second.body.contains("RATE_LIMIT_EXCEEDED"));
    }
}

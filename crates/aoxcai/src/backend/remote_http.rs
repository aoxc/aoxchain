// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    error::AiError,
    manifest::{AuthMode, HttpMethod, ModelManifest},
    model::{InferenceRequest, ModelOutput},
    traits::InferenceBackend,
};
use reqwest::Url;
use std::{net::IpAddr, time::Duration};

/// Remote HTTP backend that submits normalized inference requests to a hardened
/// JSON inference endpoint.
///
/// This implementation intentionally enforces manifest-declared endpoint policy,
/// retry limits, and output validation before a response is allowed to affect
/// node behavior.
#[derive(Debug)]
pub struct RemoteHttpBackendRuntime {
    client: reqwest::Client,
    endpoint: Url,
    auth_env: Option<String>,
    headers: Vec<(String, String)>,
    max_retries: u32,
    retry_backoff_ms: u64,
}

impl RemoteHttpBackendRuntime {
    pub fn new(manifest: &ModelManifest) -> Result<Self, AiError> {
        let cfg = manifest.spec.backend.remote_http.as_ref().ok_or_else(|| {
            AiError::ManifestValidation(
                "remote_http backend requires spec.backend.remote_http".to_owned(),
            )
        })?;

        let endpoint = Url::parse(&cfg.endpoint).map_err(|err| {
            AiError::ManifestValidation(format!("invalid remote_http endpoint: {err}"))
        })?;

        validate_endpoint_policy(manifest, &endpoint)?;
        validate_http_method(cfg.method)?;
        validate_tls_policy(cfg.tls.enabled, cfg.tls.verify_peer, &endpoint)?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(manifest.spec.backend.timeout_ms))
            .https_only(cfg.tls.enabled)
            .no_proxy()
            .build()
            .map_err(|err| AiError::Http(err.to_string()))?;

        let auth_env = match cfg.auth.mode {
            AuthMode::None => None,
            AuthMode::BearerEnv => Some(cfg.auth.env_key.clone()),
        };

        let headers = cfg
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(Self {
            client,
            endpoint,
            auth_env,
            headers,
            max_retries: manifest.spec.backend.max_retries,
            retry_backoff_ms: manifest.spec.backend.retry_backoff_ms,
        })
    }
}

#[async_trait::async_trait]
impl InferenceBackend for RemoteHttpBackendRuntime {
    fn name(&self) -> &'static str {
        "remote_http"
    }

    async fn infer(
        &self,
        manifest: &ModelManifest,
        request: &InferenceRequest,
    ) -> Result<ModelOutput, AiError> {
        let total_attempts = self.max_retries.saturating_add(1);

        for attempt in 0..total_attempts {
            let response = self.send_once(request).await;

            match response {
                Ok(output) => {
                    validate_output(manifest, &output)?;
                    return Ok(output);
                }
                Err(err) => {
                    let is_last_attempt = attempt + 1 >= total_attempts;
                    if is_last_attempt {
                        return Err(err);
                    }

                    tokio::time::sleep(Duration::from_millis(self.retry_backoff_ms)).await;
                }
            }
        }

        Err(AiError::BackendFailure(
            "remote_http execution exhausted without producing a final result".to_owned(),
        ))
    }
}

impl RemoteHttpBackendRuntime {
    async fn send_once(&self, request: &InferenceRequest) -> Result<ModelOutput, AiError> {
        let mut builder = self.client.post(self.endpoint.clone());

        for (key, value) in &self.headers {
            builder = builder.header(key, value);
        }

        if let Some(env_key) = &self.auth_env {
            let token =
                std::env::var(env_key).map_err(|_| AiError::MissingEnvironment(env_key.clone()))?;
            builder = builder.bearer_auth(token);
        }

        let response = builder
            .json(request)
            .send()
            .await
            .map_err(classify_reqwest_error)?;

        if !response.status().is_success() {
            return Err(AiError::BackendFailure(format!(
                "non-success status code: {}",
                response.status()
            )));
        }

        response
            .json::<ModelOutput>()
            .await
            .map_err(|err| AiError::BackendSchema(err.to_string()))
    }
}

fn validate_http_method(method: HttpMethod) -> Result<(), AiError> {
    match method {
        HttpMethod::Post => Ok(()),
    }
}

fn validate_tls_policy(
    tls_enabled: bool,
    verify_peer: bool,
    endpoint: &Url,
) -> Result<(), AiError> {
    if tls_enabled && endpoint.scheme() != "https" {
        return Err(AiError::ManifestValidation(
            "TLS-enabled remote_http backend requires an https endpoint".to_owned(),
        ));
    }

    if verify_peer && endpoint.scheme() != "https" {
        return Err(AiError::ManifestValidation(
            "verify_peer requires an https endpoint".to_owned(),
        ));
    }

    Ok(())
}

fn validate_endpoint_policy(manifest: &ModelManifest, endpoint: &Url) -> Result<(), AiError> {
    let endpoint_str = endpoint.as_str();
    let is_allowed = manifest
        .spec
        .security
        .allowed_endpoints
        .iter()
        .any(|allowed| endpoint_str.starts_with(allowed));

    if !is_allowed {
        return Err(AiError::ManifestValidation(format!(
            "remote_http endpoint '{}' is not permitted by security.allowed_endpoints",
            endpoint_str
        )));
    }

    if !manifest.spec.security.allow_private_networks
        && let Some(host) = endpoint.host_str()
        && is_private_network_host(host)
    {
        return Err(AiError::ManifestValidation(format!(
            "remote_http endpoint '{}' targets a private network host while allow_private_networks=false",
            endpoint_str
        )));
    }

    Ok(())
}

fn is_private_network_host(host: &str) -> bool {
    let normalized_host = host.trim_matches(['[', ']']);

    if normalized_host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    if let Ok(ip) = normalized_host.parse::<IpAddr>() {
        return match ip {
            IpAddr::V4(v4) => v4.is_private() || v4.is_loopback() || v4.is_link_local(),
            IpAddr::V6(v6) => {
                v6.is_loopback() || v6.is_unique_local() || v6.is_unicast_link_local()
            }
        };
    }

    false
}

fn classify_reqwest_error(err: reqwest::Error) -> AiError {
    if err.is_timeout() {
        return AiError::BackendTimeout(err.to_string());
    }

    if err.is_connect() {
        return AiError::BackendUnreachable(err.to_string());
    }

    AiError::Http(err.to_string())
}

fn validate_output(manifest: &ModelManifest, output: &ModelOutput) -> Result<(), AiError> {
    let validation = &manifest.spec.output.validation;

    if !validation.allowed_labels.contains(&output.label) {
        return Err(AiError::BackendSchema(format!(
            "label '{:?}' is not allowed by manifest",
            output.label
        )));
    }

    if output.risk_bps < validation.risk_bps_min || output.risk_bps > validation.risk_bps_max {
        return Err(AiError::BackendSchema(format!(
            "risk_bps '{}' is out of manifest bounds",
            output.risk_bps
        )));
    }

    if output.confidence_bps < validation.confidence_bps_min
        || output.confidence_bps > validation.confidence_bps_max
    {
        return Err(AiError::BackendSchema(format!(
            "confidence_bps '{}' is out of manifest bounds",
            output.confidence_bps
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::AiError,
        model::OutputLabel,
        test_support::{bearer_remote_http_manifest, empty_request, remote_http_manifest},
    };
    use httpmock::prelude::*;
    use serde_json::json;

    #[test]
    fn new_rejects_private_network_when_disallowed() {
        let manifest = remote_http_manifest("https://127.0.0.1:8443/infer");
        let err = RemoteHttpBackendRuntime::new(&manifest).expect_err("private network must fail");
        match err {
            AiError::ManifestValidation(message) => assert!(message.contains("private network")),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn new_rejects_localhost_when_private_networks_are_disallowed() {
        let manifest = remote_http_manifest("https://localhost:8443/infer");
        let err = RemoteHttpBackendRuntime::new(&manifest).expect_err("localhost must be blocked");
        assert!(
            matches!(err, AiError::ManifestValidation(message) if message.contains("private network"))
        );
    }

    #[test]
    fn new_rejects_ipv6_private_and_loopback_hosts_when_disallowed() {
        for endpoint in [
            "https://[::1]:8443/infer",
            "https://[fd00::1234]:8443/infer",
        ] {
            let manifest = remote_http_manifest(endpoint);
            let err = RemoteHttpBackendRuntime::new(&manifest)
                .expect_err("ipv6 loopback/private hosts must be blocked");
            assert!(
                matches!(err, AiError::ManifestValidation(ref message) if message.contains("private network")),
                "unexpected error for endpoint {endpoint}: {err:?}"
            );
        }
    }

    #[tokio::test]
    async fn infer_returns_output_when_remote_response_is_valid() {
        let server = MockServer::start();

        let response_body = json!({
            "backend": "remote_http",
            "model_id": "validator-risk-v1",
            "label": "review",
            "risk_bps": 4200,
            "confidence_bps": 8100,
            "rationale": "Remote model evaluated the supplied request.",
            "recommended_action": "review",
            "attributes": {
                "trace_id": "trace-001"
            }
        });

        let mock = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(200)
                .header("content-type", "application/json")
                .json_body_obj(&response_body);
        });

        let mut manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec![server.base_url()];

        let backend =
            RemoteHttpBackendRuntime::new(&manifest).expect("remote backend must be constructed");

        let output = backend
            .infer(&manifest, &empty_request())
            .await
            .expect("remote inference must succeed");

        mock.assert();
        assert_eq!(output.label, OutputLabel::Review);
        assert_eq!(output.risk_bps, 4200);
    }

    #[tokio::test]
    async fn infer_rejects_missing_bearer_environment_variable() {
        let server = MockServer::start();

        let mut manifest = bearer_remote_http_manifest(
            format!("{}/infer", server.base_url()),
            "AOXC_TEST_MISSING_REMOTE_HTTP_TOKEN_8F6A9C0B",
        );
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec![server.base_url()];

        let backend =
            RemoteHttpBackendRuntime::new(&manifest).expect("remote backend must be constructed");

        let err = backend
            .infer(&manifest, &empty_request())
            .await
            .expect_err("missing environment variable must fail");

        assert_eq!(
            err,
            AiError::MissingEnvironment("AOXC_TEST_MISSING_REMOTE_HTTP_TOKEN_8F6A9C0B".to_owned())
        );
    }

    #[test]
    fn new_rejects_non_https_endpoint_when_tls_enabled() {
        let mut manifest = remote_http_manifest("http://example.com/infer");
        let cfg = manifest
            .spec
            .backend
            .remote_http
            .as_mut()
            .expect("remote http config must exist");
        cfg.tls.enabled = true;
        cfg.tls.verify_peer = false;
        manifest.spec.security.allowed_endpoints = vec!["http://example.com".to_owned()];

        let err = RemoteHttpBackendRuntime::new(&manifest).expect_err("must reject non-https");
        assert!(
            matches!(err, AiError::ManifestValidation(message) if message.contains("TLS-enabled"))
        );
    }

    #[test]
    fn new_rejects_verify_peer_when_endpoint_is_not_https() {
        let mut manifest = remote_http_manifest("http://example.com/infer");
        let cfg = manifest
            .spec
            .backend
            .remote_http
            .as_mut()
            .expect("remote http config must exist");
        cfg.tls.enabled = false;
        cfg.tls.verify_peer = true;
        manifest.spec.security.allowed_endpoints = vec!["http://example.com".to_owned()];

        let err = RemoteHttpBackendRuntime::new(&manifest).expect_err("must reject verify_peer");
        assert!(
            matches!(err, AiError::ManifestValidation(message) if message.contains("verify_peer"))
        );
    }

    #[test]
    fn new_rejects_endpoint_outside_allowed_endpoints() {
        let mut manifest = remote_http_manifest("https://example.com/infer");
        manifest.spec.security.allowed_endpoints = vec!["https://api.example.com".to_owned()];

        let err = RemoteHttpBackendRuntime::new(&manifest).expect_err("endpoint must be blocked");
        assert!(
            matches!(err, AiError::ManifestValidation(message) if message.contains("not permitted"))
        );
    }

    #[tokio::test]
    async fn infer_returns_error_for_non_success_http_status() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(503).body("backend overloaded");
        });

        let mut manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec![server.base_url()];
        let backend = RemoteHttpBackendRuntime::new(&manifest).expect("backend must construct");

        let err = backend
            .infer(&manifest, &empty_request())
            .await
            .expect_err("non-success status must fail");
        mock.assert();

        assert!(
            matches!(err, AiError::BackendFailure(message) if message.contains("non-success status code"))
        );
    }

    #[tokio::test]
    async fn infer_returns_schema_error_for_invalid_json_response() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(200)
                .header("content-type", "application/json")
                .body("{not-json");
        });

        let mut manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec![server.base_url()];
        let backend = RemoteHttpBackendRuntime::new(&manifest).expect("backend must construct");

        let err = backend
            .infer(&manifest, &empty_request())
            .await
            .expect_err("invalid json must fail");
        mock.assert();

        assert!(matches!(err, AiError::BackendSchema(_)));
    }

    #[tokio::test]
    async fn infer_rejects_disallowed_output_label() {
        let server = MockServer::start();
        let response_body = json!({
            "backend": "remote_http",
            "model_id": "validator-risk-v1",
            "label": "unknown",
            "risk_bps": 100,
            "confidence_bps": 8000,
            "rationale": "label not allowed for this manifest",
            "recommended_action": "review",
            "attributes": {}
        });
        let mock = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(200)
                .header("content-type", "application/json")
                .json_body_obj(&response_body);
        });

        let mut manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec![server.base_url()];
        manifest.spec.output.validation.allowed_labels = vec![OutputLabel::Review];
        let backend = RemoteHttpBackendRuntime::new(&manifest).expect("backend must construct");

        let err = backend
            .infer(&manifest, &empty_request())
            .await
            .expect_err("disallowed label must fail");
        mock.assert();

        assert!(matches!(err, AiError::BackendSchema(message) if message.contains("not allowed")));
    }

    #[tokio::test]
    async fn infer_rejects_out_of_bounds_risk_bps() {
        let server = MockServer::start();
        let response_body = json!({
            "backend": "remote_http",
            "model_id": "validator-risk-v1",
            "label": "review",
            "risk_bps": 9000,
            "confidence_bps": 8000,
            "rationale": "risk too high for manifest policy",
            "recommended_action": "review",
            "attributes": {}
        });
        let mock = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(200)
                .header("content-type", "application/json")
                .json_body_obj(&response_body);
        });

        let mut manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec![server.base_url()];
        manifest.spec.output.validation.risk_bps_max = 1000;
        let backend = RemoteHttpBackendRuntime::new(&manifest).expect("backend must construct");

        let err = backend
            .infer(&manifest, &empty_request())
            .await
            .expect_err("out-of-bounds risk must fail");
        mock.assert();

        assert!(matches!(err, AiError::BackendSchema(message) if message.contains("risk_bps")));
    }

    #[tokio::test]
    async fn infer_rejects_out_of_bounds_confidence_bps() {
        let server = MockServer::start();
        let response_body = json!({
            "backend": "remote_http",
            "model_id": "validator-risk-v1",
            "label": "review",
            "risk_bps": 400,
            "confidence_bps": 100,
            "rationale": "confidence below policy floor",
            "recommended_action": "review",
            "attributes": {}
        });
        let mock = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(200)
                .header("content-type", "application/json")
                .json_body_obj(&response_body);
        });

        let mut manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec![server.base_url()];
        manifest.spec.output.validation.confidence_bps_min = 1000;
        let backend = RemoteHttpBackendRuntime::new(&manifest).expect("backend must construct");

        let err = backend
            .infer(&manifest, &empty_request())
            .await
            .expect_err("out-of-bounds confidence must fail");
        mock.assert();

        assert!(
            matches!(err, AiError::BackendSchema(message) if message.contains("confidence_bps"))
        );
    }

    #[tokio::test]
    async fn infer_maps_connection_failures_to_backend_unreachable() {
        let mut manifest = remote_http_manifest("http://127.0.0.1:9/infer");
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec!["http://127.0.0.1:9".to_owned()];
        let backend = RemoteHttpBackendRuntime::new(&manifest).expect("backend must construct");

        let err = backend
            .infer(&manifest, &empty_request())
            .await
            .expect_err("connection failure must fail");

        assert!(matches!(err, AiError::BackendUnreachable(_)));
    }

    #[tokio::test]
    async fn infer_retries_and_then_succeeds_after_transient_failure() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("listener must bind");
        let address = listener
            .local_addr()
            .expect("listener address must resolve");

        let server = std::thread::spawn(move || {
            for attempt in 0..2 {
                let (mut stream, _) = listener.accept().expect("request must be accepted");

                let mut buffer = [0_u8; 2048];
                let _ = stream.read(&mut buffer);

                if attempt == 0 {
                    stream
                        .write_all(
                            b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                        )
                        .expect("transient response must be written");
                } else {
                    let body = r#"{"backend":"remote_http","model_id":"validator-risk-v1","label":"review","risk_bps":2100,"confidence_bps":7600,"rationale":"recovered after transient backend issue","recommended_action":"review","attributes":{}}"#;
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream
                        .write_all(response.as_bytes())
                        .expect("success response must be written");
                }
                stream.flush().expect("stream must flush");
            }
        });

        let base_url = format!("http://{address}");
        let mut manifest = remote_http_manifest(format!("{base_url}/infer"));
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec![base_url];
        manifest.spec.backend.max_retries = 1;
        manifest.spec.backend.retry_backoff_ms = 1;
        let backend = RemoteHttpBackendRuntime::new(&manifest).expect("backend must construct");

        let output = backend
            .infer(&manifest, &empty_request())
            .await
            .expect("retry should eventually succeed");

        server.join().expect("mock server thread must complete");
        assert_eq!(output.label, OutputLabel::Review);
    }

    #[tokio::test]
    async fn infer_returns_final_error_after_retry_exhaustion() {
        let server = MockServer::start();
        let failure = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(500).body("persistent failure");
        });

        let mut manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec![server.base_url()];
        manifest.spec.backend.max_retries = 2;
        manifest.spec.backend.retry_backoff_ms = 1;
        let backend = RemoteHttpBackendRuntime::new(&manifest).expect("backend must construct");

        let err = backend
            .infer(&manifest, &empty_request())
            .await
            .expect_err("persistent failure must surface");

        assert!(
            matches!(err, AiError::BackendFailure(message) if message.contains("non-success status code"))
        );
        assert_eq!(failure.calls(), 3);
    }
}

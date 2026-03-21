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
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
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
}

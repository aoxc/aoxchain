use crate::{
    error::AiError,
    manifest::ModelManifest,
    model::{InferenceRequest, ModelOutput},
    traits::InferenceBackend,
};

/// Remote HTTP backend that submits normalized inference requests to an
/// OpenAI-compatible or equivalent JSON inference endpoint.
pub struct RemoteHttpBackendRuntime {
    client: reqwest::Client,
    endpoint: String,
    auth_env: Option<String>,
    headers: Vec<(String, String)>,
}

impl RemoteHttpBackendRuntime {
    /// Constructs a new remote HTTP backend from manifest configuration.
    pub fn new(manifest: &ModelManifest) -> Result<Self, AiError> {
        let cfg = manifest.spec.backend.remote_http.as_ref().ok_or_else(|| {
            AiError::ManifestValidation(
                "remote_http backend requires spec.backend.remote_http".to_owned(),
            )
        })?;

        let timeout = std::time::Duration::from_millis(manifest.spec.backend.timeout_ms);

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .no_proxy()
            .build()
            .map_err(|err| AiError::Http(err.to_string()))?;

        let auth_env = match cfg.auth.mode.as_str() {
            "bearer_env" => Some(cfg.auth.env_key.clone()),
            "none" => None,
            other => {
                return Err(AiError::ManifestValidation(format!(
                    "unsupported auth mode '{}'",
                    other
                )));
            }
        };

        let headers = cfg
            .headers
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        Ok(Self {
            client,
            endpoint: cfg.endpoint.clone(),
            auth_env,
            headers,
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
        let mut builder = self.client.post(&self.endpoint);

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
            .map_err(|err| AiError::Http(err.to_string()))?;

        if !response.status().is_success() {
            return Err(AiError::Http(format!(
                "non-success status code: {}",
                response.status()
            )));
        }

        let output = response
            .json::<ModelOutput>()
            .await
            .map_err(|err| AiError::Json(err.to_string()))?;

        validate_output(manifest, &output)?;
        Ok(output)
    }
}

fn validate_output(manifest: &ModelManifest, output: &ModelOutput) -> Result<(), AiError> {
    let validation = &manifest.spec.output.validation;

    if !validation
        .allowed_labels
        .iter()
        .any(|label| label == &output.label)
    {
        return Err(AiError::BackendFailure(format!(
            "label '{}' is not allowed by manifest",
            output.label
        )));
    }

    if output.risk_bps < validation.risk_bps_min || output.risk_bps > validation.risk_bps_max {
        return Err(AiError::BackendFailure(format!(
            "risk_bps '{}' is out of manifest bounds",
            output.risk_bps
        )));
    }

    if output.confidence_bps < validation.confidence_bps_min
        || output.confidence_bps > validation.confidence_bps_max
    {
        return Err(AiError::BackendFailure(format!(
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
        test_support::{
            base_manifest, bearer_remote_http_manifest, empty_request, remote_http_manifest,
        },
    };
    use httpmock::prelude::*;
    use serde_json::json;

    #[test]
    fn new_rejects_missing_remote_http_configuration() {
        let mut manifest = base_manifest();
        manifest.spec.backend.r#type = "remote_http".to_owned();
        manifest.spec.backend.heuristic = None;
        manifest.spec.backend.remote_http = None;

        let err = match RemoteHttpBackendRuntime::new(&manifest) {
            Ok(_) => panic!("missing remote_http configuration must fail"),
            Err(err) => err,
        };

        match err {
            AiError::ManifestValidation(message) => {
                assert_eq!(
                    message,
                    "remote_http backend requires spec.backend.remote_http"
                );
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn infer_returns_output_when_remote_backend_response_is_valid() {
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

        let manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        let backend =
            RemoteHttpBackendRuntime::new(&manifest).expect("remote backend must be constructed");
        let request = empty_request();

        let output = backend
            .infer(&manifest, &request)
            .await
            .expect("remote inference must succeed");

        mock.assert();

        assert_eq!(output.backend, "remote_http");
        assert_eq!(output.label, "review");
        assert_eq!(output.risk_bps, 4200);
        assert_eq!(output.confidence_bps, 8100);
        assert_eq!(
            output.attributes.get("trace_id"),
            Some(&"trace-001".to_owned())
        );
    }

    #[tokio::test]
    async fn infer_returns_http_error_on_non_success_status() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(503)
                .header("content-type", "text/plain")
                .body("service unavailable");
        });

        let manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        let backend =
            RemoteHttpBackendRuntime::new(&manifest).expect("remote backend must be constructed");
        let request = empty_request();

        let err = backend
            .infer(&manifest, &request)
            .await
            .expect_err("non-success status must fail");

        mock.assert();

        match err {
            AiError::Http(message) => {
                assert!(message.contains("non-success status code"));
                assert!(message.contains("503"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn infer_rejects_output_with_invalid_label() {
        let server = MockServer::start();

        let response_body = json!({
            "backend": "remote_http",
            "model_id": "validator-risk-v1",
            "label": "forbidden_label",
            "risk_bps": 2000,
            "confidence_bps": 7000,
            "rationale": "Invalid label for validation test.",
            "recommended_action": null,
            "attributes": {}
        });

        let mock = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(200)
                .header("content-type", "application/json")
                .json_body_obj(&response_body);
        });

        let manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        let backend =
            RemoteHttpBackendRuntime::new(&manifest).expect("remote backend must be constructed");
        let request = empty_request();

        let err = backend
            .infer(&manifest, &request)
            .await
            .expect_err("invalid label must fail output validation");

        mock.assert();

        match err {
            AiError::BackendFailure(message) => {
                assert!(message.contains("label"));
                assert!(message.contains("not allowed"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn infer_rejects_output_with_out_of_range_risk() {
        let server = MockServer::start();

        let response_body = json!({
            "backend": "remote_http",
            "model_id": "validator-risk-v1",
            "label": "review",
            "risk_bps": 10001,
            "confidence_bps": 7000,
            "rationale": "Out-of-range risk for validation test.",
            "recommended_action": null,
            "attributes": {}
        });

        let mock = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(200)
                .header("content-type", "application/json")
                .json_body_obj(&response_body);
        });

        let manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        let backend =
            RemoteHttpBackendRuntime::new(&manifest).expect("remote backend must be constructed");
        let request = empty_request();

        let err = backend
            .infer(&manifest, &request)
            .await
            .expect_err("out-of-range risk must fail output validation");

        mock.assert();

        match err {
            AiError::BackendFailure(message) => {
                assert!(message.contains("risk_bps"));
                assert!(message.contains("out of manifest bounds"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn infer_rejects_output_with_out_of_range_confidence() {
        let server = MockServer::start();

        let response_body = json!({
            "backend": "remote_http",
            "model_id": "validator-risk-v1",
            "label": "review",
            "risk_bps": 5000,
            "confidence_bps": 10001,
            "rationale": "Out-of-range confidence for validation test.",
            "recommended_action": null,
            "attributes": {}
        });

        let mock = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(200)
                .header("content-type", "application/json")
                .json_body_obj(&response_body);
        });

        let manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        let backend =
            RemoteHttpBackendRuntime::new(&manifest).expect("remote backend must be constructed");
        let request = empty_request();

        let err = backend
            .infer(&manifest, &request)
            .await
            .expect_err("out-of-range confidence must fail output validation");

        mock.assert();

        match err {
            AiError::BackendFailure(message) => {
                assert!(message.contains("confidence_bps"));
                assert!(message.contains("out of manifest bounds"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn infer_rejects_missing_bearer_environment_variable() {
        let server = MockServer::start();

        let manifest = bearer_remote_http_manifest(
            format!("{}/infer", server.base_url()),
            "AOXC_TEST_MISSING_REMOTE_HTTP_TOKEN_8F6A9C0B",
        );
        let backend =
            RemoteHttpBackendRuntime::new(&manifest).expect("remote backend must be constructed");
        let request = empty_request();

        let err = backend
            .infer(&manifest, &request)
            .await
            .expect_err("missing environment variable must fail");

        match err {
            AiError::MissingEnvironment(name) => {
                assert_eq!(name, "AOXC_TEST_MISSING_REMOTE_HTTP_TOKEN_8F6A9C0B");
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}

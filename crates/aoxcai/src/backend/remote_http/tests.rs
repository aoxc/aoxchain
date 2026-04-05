use super::*;
use crate::{
    error::AiError,
    model::OutputLabel,
    test_support::{bearer_remote_http_manifest, empty_request, remote_http_manifest},
    traits::InferenceBackend,
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
    assert!(matches!(err, AiError::ManifestValidation(message) if message.contains("TLS-enabled")));
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
    assert!(matches!(err, AiError::ManifestValidation(message) if message.contains("verify_peer")));
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

    assert!(matches!(err, AiError::BackendSchema(message) if message.contains("confidence_bps")));
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

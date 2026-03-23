use crate::{
    manifest::{
        ActionMap, Audit, Auth, AuthMode, Backend, BackendFailureAction, BackendType, Bindings,
        CircuitBreaker, Compatibility, Confidence, Decision, DeterministicOverrides, Fallback,
        Fusion, FusionStrategy, FusionWeights, HeuristicBackend, HttpMethod, Input,
        InvalidOutputBehavior, Lifecycle, ManifestSignature, Metadata, Metrics, ModelManifest,
        Observability, Output, OutputMapping, OutputValidation, Prompt, RateLimit,
        RemoteHttpBackend, Security, Spec, SubjectShape, Thresholds, Tls, Tracing,
        TruncateStrategy,
    },
    model::{
        ActionName, AiMode, AiTask, InferenceContext, InferenceFinding, InferenceRequest,
        InferenceSignal, ModelOutput, OutputLabel,
    },
};
use std::collections::BTreeMap;

pub fn base_manifest() -> ModelManifest {
    ModelManifest {
        api_version: "aoxc.ai/v1".to_owned(),
        kind: "ModelManifest".to_owned(),
        metadata: Metadata {
            id: "test-model".to_owned(),
            name: "test".to_owned(),
            version: "1.0.0".to_owned(),
            description: "test manifest".to_owned(),
            owner: "test".to_owned(),
            tags: vec![],
            created_at: "2026-03-15T00:00:00Z".to_owned(),
            updated_at: "2026-03-15T00:00:00Z".to_owned(),
        },
        spec: Spec {
            enabled: true,
            lifecycle: Lifecycle {
                hot_reload: false,
                reload_strategy: "none".to_owned(),
                immutable_fields: vec![],
            },
            bindings: Bindings {
                default_for_tasks: vec![AiTask::ValidatorAdmission],
                optional_tasks: vec![AiTask::PeerScreening],
                node_scopes: vec![],
                tenants: vec![],
            },
            backend: Backend {
                r#type: BackendType::Heuristic,
                driver: "builtin".to_owned(),
                priority: 1,
                timeout_ms: 1_000,
                max_retries: 0,
                retry_backoff_ms: 0,
                circuit_breaker: CircuitBreaker {
                    enabled: false,
                    failure_threshold: 0,
                    cooldown_ms: 0,
                },
                remote_http: None,
                local_candle: None,
                local_onnx: None,
                heuristic: Some(HeuristicBackend {
                    ruleset: "default".to_owned(),
                    anomaly_keywords: vec!["revoked".to_owned(), "anomaly".to_owned()],
                }),
                ensemble: None,
            },
            input: Input {
                format: "structured_json".to_owned(),
                include_narrative: true,
                include_signals: true,
                include_findings: true,
                include_context_metadata: true,
                include_evidence_refs: false,
                max_signal_count: 64,
                max_finding_count: 32,
                max_evidence_refs: 0,
                truncate_strategy: TruncateStrategy::WeightDesc,
                signal_encoding: "typed".to_owned(),
                subject: SubjectShape {
                    required_fields: vec![
                        "subject_id".to_owned(),
                        "subject_kind".to_owned(),
                        "task".to_owned(),
                        "mode".to_owned(),
                    ],
                    optional_fields: vec![],
                },
                prompt: Prompt {
                    style: "plain".to_owned(),
                    system_instructions: String::new(),
                    narrative_template: String::new(),
                },
            },
            output: Output {
                format: "normalized_json".to_owned(),
                required_fields: vec![
                    "label".to_owned(),
                    "risk_bps".to_owned(),
                    "confidence_bps".to_owned(),
                    "rationale".to_owned(),
                ],
                optional_fields: vec!["recommended_action".to_owned(), "attributes".to_owned()],
                mapping: OutputMapping {
                    label_field: "label".to_owned(),
                    risk_bps_field: "risk_bps".to_owned(),
                    confidence_bps_field: "confidence_bps".to_owned(),
                    rationale_field: "rationale".to_owned(),
                    recommended_action_field: "recommended_action".to_owned(),
                    attributes_field: "attributes".to_owned(),
                },
                validation: OutputValidation {
                    allowed_labels: vec![
                        OutputLabel::Trusted,
                        OutputLabel::Review,
                        OutputLabel::Suspicious,
                        OutputLabel::Malicious,
                        OutputLabel::Unknown,
                    ],
                    risk_bps_min: 0,
                    risk_bps_max: 10_000,
                    confidence_bps_min: 0,
                    confidence_bps_max: 10_000,
                    on_invalid_output: InvalidOutputBehavior::Reject,
                },
            },
            decision: Decision {
                mode: AiMode::Enforced,
                thresholds: Thresholds {
                    allow_max_risk_bps: 2_499,
                    review_max_risk_bps: 6_999,
                    deny_min_risk_bps: 7_000,
                },
                confidence: Confidence {
                    minimum_confidence_bps: 3_500,
                    low_confidence_action: ActionName::Review,
                },
                fusion: Fusion {
                    strategy: FusionStrategy::Weighted,
                    weights: FusionWeights {
                        model_risk_bps: 6_000,
                        deterministic_risk_bps: 4_000,
                    },
                    deterministic_overrides: DeterministicOverrides {
                        deny_on_critical_finding: true,
                        deny_on_revoked_identity: true,
                        deny_on_invalid_quorum_proof: true,
                        review_on_missing_context: true,
                        review_on_model_timeout: true,
                    },
                },
                actions: ActionMap {
                    trusted: ActionName::Allow,
                    review: ActionName::Review,
                    suspicious: ActionName::Review,
                    malicious: ActionName::Deny,
                    unknown: ActionName::Review,
                },
            },
            fallback: Fallback {
                enabled: true,
                backend: None,
                action_on_backend_error: BackendFailureAction::Review,
                action_on_timeout: BackendFailureAction::Review,
                action_on_schema_error: BackendFailureAction::Review,
                action_on_unreachable_backend: BackendFailureAction::Review,
                action_on_empty_response: BackendFailureAction::Review,
            },
            observability: Observability {
                metrics: Metrics {
                    enabled: false,
                    namespace: "test".to_owned(),
                    labels: Default::default(),
                },
                tracing: Tracing {
                    enabled: false,
                    include_request_hash: false,
                    include_subject_id: false,
                    include_backend_latency: false,
                },
                audit: Audit {
                    enabled: false,
                    log_decision_input: false,
                    log_decision_output: false,
                    redact_fields: vec![],
                    persist_reports: false,
                    persist_path: String::new(),
                },
            },
            security: Security {
                manifest_signature: ManifestSignature {
                    required: false,
                    algorithm: "none".to_owned(),
                    public_key_path: None,
                    signature_field: String::new(),
                },
                allowed_endpoints: vec!["https://inference.aoxc.local".to_owned()],
                allow_private_networks: false,
                allow_file_backends: false,
            },
            compatibility: Compatibility {
                min_node_version: "0.0.0-alpha.1".to_owned(),
                max_node_version: None,
                supported_tasks: vec![
                    AiTask::ValidatorAdmission,
                    AiTask::PeerScreening,
                    AiTask::TransactionScreening,
                    AiTask::ArtifactInspection,
                ],
                supported_modes: vec![AiMode::Advisory, AiMode::Enforced],
            },
            extensions: Default::default(),
        },
    }
}

pub fn heuristic_manifest() -> ModelManifest {
    base_manifest()
}

pub fn remote_http_manifest(endpoint: impl Into<String>) -> ModelManifest {
    let endpoint = endpoint.into();
    let mut manifest = base_manifest();
    manifest.spec.backend.r#type = BackendType::RemoteHttp;
    manifest.spec.backend.driver = "openai_compatible_json".to_owned();
    manifest.spec.backend.heuristic = None;
    manifest.spec.backend.remote_http = Some(RemoteHttpBackend {
        endpoint: endpoint.clone(),
        method: HttpMethod::Post,
        headers: {
            let mut headers = BTreeMap::new();
            headers.insert("Content-Type".to_owned(), "application/json".to_owned());
            headers
        },
        auth: Auth {
            mode: AuthMode::None,
            env_key: String::new(),
        },
        tls: Tls {
            enabled: endpoint.starts_with("https://"),
            verify_peer: endpoint.starts_with("https://"),
        },
        rate_limit: RateLimit {
            requests_per_minute: 60,
            burst: 10,
        },
    });

    if let Some((prefix, _)) = endpoint.rsplit_once('/') {
        manifest.spec.security.allowed_endpoints = vec![prefix.to_owned()];
    } else {
        manifest.spec.security.allowed_endpoints = vec![endpoint];
    }

    manifest
}

pub fn bearer_remote_http_manifest(
    endpoint: impl Into<String>,
    env_key: impl Into<String>,
) -> ModelManifest {
    let mut manifest = remote_http_manifest(endpoint);
    if let Some(cfg) = manifest.spec.backend.remote_http.as_mut() {
        cfg.auth.mode = AuthMode::BearerEnv;
        cfg.auth.env_key = env_key.into();
    }
    manifest
}

pub fn request_with(
    signals: Vec<InferenceSignal>,
    findings: Vec<InferenceFinding>,
) -> InferenceRequest {
    InferenceRequest {
        task: AiTask::ValidatorAdmission,
        mode: AiMode::Enforced,
        context: InferenceContext::new("validator-001", "validator"),
        signals,
        findings,
        narrative: Some("Deterministic unit test narrative.".to_owned()),
    }
}

pub fn empty_request() -> InferenceRequest {
    request_with(vec![], vec![])
}

pub fn model_output(label: OutputLabel, risk_bps: u16, confidence_bps: u16) -> ModelOutput {
    ModelOutput {
        backend: "heuristic".to_owned(),
        model_id: "test-model".to_owned(),
        label,
        risk_bps,
        confidence_bps,
        rationale: "Unit test model output.".to_owned(),
        recommended_action: None,
        attributes: Default::default(),
    }
}

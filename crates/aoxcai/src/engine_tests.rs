#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ModelRegistry,
        model::{InferenceContext, InferenceSignal, OutputLabel},
        test_support::{base_manifest, remote_http_manifest},
        traits::{ContextProvider, DecisionPolicy, SignalProvider},
    };
    use httpmock::prelude::*;
    use serde_json::json;

    struct StaticContextProvider;

    #[async_trait::async_trait]
    impl ContextProvider for StaticContextProvider {
        fn name(&self) -> &'static str {
            "static-context"
        }

        async fn build(
            &self,
            _task: AiTask,
            subject_id: &str,
        ) -> Result<InferenceContext, AiError> {
            Ok(InferenceContext::new(subject_id, "validator"))
        }
    }

    struct StaticSignalProvider {
        signals: Vec<crate::model::InferenceSignal>,
    }

    #[async_trait::async_trait]
    impl SignalProvider for StaticSignalProvider {
        fn name(&self) -> &'static str {
            "static-signals"
        }

        async fn collect(
            &self,
            _task: AiTask,
            _subject_id: &str,
        ) -> Result<Vec<InferenceSignal>, AiError> {
            Ok(self.signals.clone())
        }
    }

    struct PassthroughPolicy;

    #[async_trait::async_trait]
    impl DecisionPolicy for PassthroughPolicy {
        fn name(&self) -> &'static str {
            "passthrough"
        }

        async fn decide(
            &self,
            _manifest: &ModelManifest,
            _request: &InferenceRequest,
            output: &crate::model::ModelOutput,
            findings: &[InferenceFinding],
        ) -> Result<Assessment, AiError> {
            Ok(Assessment {
                action: if findings
                    .iter()
                    .any(|finding| finding.severity == FindingSeverity::Critical)
                {
                    DecisionAction::Deny
                } else {
                    DecisionAction::Review
                },
                effective_risk_bps: output.risk_bps,
                confidence_bps: output.confidence_bps,
                rationale: format!("passthrough findings={}", findings.len()),
            })
        }
    }

    struct FailingPolicy;

    #[async_trait::async_trait]
    impl DecisionPolicy for FailingPolicy {
        fn name(&self) -> &'static str {
            "failing-policy"
        }

        async fn decide(
            &self,
            _manifest: &ModelManifest,
            _request: &InferenceRequest,
            _output: &crate::model::ModelOutput,
            _findings: &[InferenceFinding],
        ) -> Result<Assessment, AiError> {
            Err(AiError::PolicyFailure("policy execution failed".to_owned()))
        }
    }

    fn registry_with(mut manifest: ModelManifest) -> ModelRegistry {
        let mut registry = ModelRegistry::new();
        if manifest.spec.bindings.default_for_tasks.is_empty() {
            manifest
                .spec
                .bindings
                .default_for_tasks
                .push(AiTask::ValidatorAdmission);
        }
        registry.register(manifest).expect("manifest must register");
        registry
    }

    #[test]
    fn ensure_task_supported_rejects_unsupported_mode() {
        let mut manifest = base_manifest();
        manifest.spec.compatibility.supported_modes = vec![AiMode::Advisory];

        let err = ensure_task_supported(&manifest, AiTask::ValidatorAdmission, AiMode::Enforced)
            .expect_err("unsupported mode must fail");
        assert!(matches!(err, AiError::ManifestValidation(message) if message.contains("mode")));
    }

    #[test]
    fn ensure_task_supported_rejects_unsupported_task() {
        let mut manifest = base_manifest();
        manifest.spec.compatibility.supported_tasks = vec![AiTask::PeerScreening];

        let err = ensure_task_supported(&manifest, AiTask::ValidatorAdmission, AiMode::Enforced)
            .expect_err("unsupported task must fail");
        assert!(matches!(err, AiError::ManifestValidation(message) if message.contains("task")));
    }

    #[test]
    fn truncate_signals_keeps_highest_weight_entries() {
        let mut manifest = base_manifest();
        manifest.spec.input.max_signal_count = 2;
        let mut signals = vec![
            InferenceSignal::new("a", "1", 100, "test"),
            InferenceSignal::new("b", "1", 5_000, "test"),
            InferenceSignal::new("c", "1", 2_500, "test"),
        ];

        truncate_signals(&manifest, &mut signals);
        assert_eq!(signals.len(), 2);
        assert_eq!(signals[0].name, "b");
        assert_eq!(signals[1].name, "c");
    }

    #[test]
    fn truncate_signals_preserves_order_for_equal_weights() {
        let mut manifest = base_manifest();
        manifest.spec.input.max_signal_count = 2;
        let mut signals = vec![
            InferenceSignal::new("first", "v", 1_000, "test"),
            InferenceSignal::new("second", "v", 1_000, "test"),
            InferenceSignal::new("third", "v", 900, "test"),
        ];

        truncate_signals(&manifest, &mut signals);
        assert_eq!(signals.len(), 2);
        assert_eq!(signals[0].name, "first");
        assert_eq!(signals[1].name, "second");
    }

    #[test]
    fn truncate_signals_with_zero_limit_drops_all_signals() {
        let mut manifest = base_manifest();
        manifest.spec.input.max_signal_count = 0;
        let mut signals = vec![
            InferenceSignal::new("a", "v", 100, "test"),
            InferenceSignal::new("b", "v", 200, "test"),
        ];

        truncate_signals(&manifest, &mut signals);
        assert!(signals.is_empty());
    }

    #[test]
    fn truncate_signals_does_not_truncate_when_under_limit() {
        let mut manifest = base_manifest();
        manifest.spec.input.max_signal_count = 5;
        let mut signals = vec![
            InferenceSignal::new("a", "v", 100, "test"),
            InferenceSignal::new("b", "v", 200, "test"),
        ];

        truncate_signals(&manifest, &mut signals);
        assert_eq!(signals.len(), 2);
        assert_eq!(signals[0].name, "b");
        assert_eq!(signals[1].name, "a");
    }

    #[test]
    fn deterministic_findings_recognize_expected_signal_patterns() {
        let signals = vec![
            InferenceSignal::new("identity", "revoked_key", 100, "test"),
            InferenceSignal::new("quorum", "invalid_quorum_signature", 100, "test"),
            InferenceSignal::new("runtime", "timeout_anomaly", 100, "test"),
        ];

        let findings = deterministic_findings(&signals);
        assert_eq!(findings.len(), 3);
        assert_eq!(findings[0].code, "revoked_identity");
        assert_eq!(findings[1].code, "invalid_quorum_proof");
        assert_eq!(findings[2].code, "runtime_anomaly");
    }

    #[test]
    fn deterministic_findings_cover_all_target_patterns_and_no_match_case() {
        let revoked = deterministic_findings(&[InferenceSignal::new(
            "identity",
            "revoked_credential",
            100,
            "test",
        )]);
        assert_eq!(revoked.len(), 1);
        assert_eq!(revoked[0].code, "revoked_identity");

        let invalid_quorum = deterministic_findings(&[InferenceSignal::new(
            "quorum",
            "invalid_quorum_signature",
            100,
            "test",
        )]);
        assert_eq!(invalid_quorum.len(), 1);
        assert_eq!(invalid_quorum[0].code, "invalid_quorum_proof");

        let timeout =
            deterministic_findings(&[InferenceSignal::new("runtime", "timeout", 100, "test")]);
        assert_eq!(timeout.len(), 1);
        assert_eq!(timeout[0].code, "runtime_anomaly");

        let anomaly = deterministic_findings(&[InferenceSignal::new(
            "runtime",
            "anomaly_detected",
            100,
            "test",
        )]);
        assert_eq!(anomaly.len(), 1);
        assert_eq!(anomaly[0].code, "runtime_anomaly");

        let no_match =
            deterministic_findings(&[InferenceSignal::new("health", "healthy", 100, "test")]);
        assert!(no_match.is_empty());
    }

    #[test]
    fn build_narrative_respects_manifest_toggle() {
        let mut manifest = base_manifest();
        manifest.spec.input.include_narrative = false;
        let narrative = build_narrative(
            &manifest,
            AiTask::ValidatorAdmission,
            AiMode::Enforced,
            "validator",
            "validator-1",
            &[],
            &[],
        );
        assert!(narrative.is_none());
    }

    #[test]
    fn build_narrative_contains_expected_content_when_enabled() {
        let manifest = base_manifest();
        let signals = vec![InferenceSignal::new(
            "identity",
            "revoked_identity",
            6_000,
            "test",
        )];
        let findings = vec![InferenceFinding::new(
            "revoked_identity",
            "detected revoked identity",
            FindingSeverity::Critical,
        )];

        let narrative = build_narrative(
            &manifest,
            AiTask::ValidatorAdmission,
            AiMode::Enforced,
            "validator",
            "validator-narrative",
            &signals,
            &findings,
        )
        .expect("narrative must be generated");

        assert!(narrative.contains("Task: ValidatorAdmission"));
        assert!(narrative.contains("Mode: Enforced"));
        assert!(narrative.contains("SubjectId: validator-narrative"));
        assert!(narrative.contains("identity=revoked_identity (weight_bps=6000)"));
        assert!(narrative.contains("revoked_identity [Critical]"));
    }

    #[test]
    fn fallback_assessment_maps_timeout_and_unreachable_errors() {
        let manifest = base_manifest();
        let findings = vec![InferenceFinding::new(
            "runtime_anomaly",
            "warn",
            FindingSeverity::Warning,
        )];

        let timeout = fallback_assessment(
            &manifest,
            &AiError::BackendTimeout("slow".into()),
            &findings,
        );
        assert_eq!(timeout.action, DecisionAction::Review);
        assert_eq!(timeout.effective_risk_bps, 5_000);

        let unreachable = fallback_assessment(
            &manifest,
            &AiError::BackendUnreachable("down".into()),
            &findings,
        );
        assert_eq!(unreachable.action, DecisionAction::Review);
    }

    #[tokio::test]
    async fn evaluate_rejects_empty_subject_identifier() {
        let engine = AiEngine::new(
            registry_with(base_manifest()),
            Box::new(StaticContextProvider),
            vec![],
            Box::new(PassthroughPolicy),
        );

        let err = engine
            .evaluate(AiTask::ValidatorAdmission, AiMode::Enforced, "   ")
            .await
            .expect_err("empty subject id must fail");
        assert_eq!(
            err,
            AiError::InvalidInput("subject identifier must not be empty".to_owned())
        );
    }

    #[tokio::test]
    async fn evaluate_truncates_signals_and_produces_findings() {
        let mut manifest = base_manifest();
        manifest.spec.input.max_signal_count = 2;
        let engine = AiEngine::new(
            registry_with(manifest),
            Box::new(StaticContextProvider),
            vec![Box::new(StaticSignalProvider {
                signals: vec![
                    InferenceSignal::new("runtime", "healthy", 100, "test"),
                    InferenceSignal::new("identity", "revoked_identity", 7_000, "test"),
                    InferenceSignal::new("runtime", "timeout_anomaly", 5_000, "test"),
                ],
            })],
            Box::new(PassthroughPolicy),
        );

        let report = engine
            .evaluate(
                AiTask::ValidatorAdmission,
                AiMode::Enforced,
                "validator-007",
            )
            .await
            .expect("evaluation must succeed");

        assert_eq!(report.request.signals.len(), 2);
        assert_eq!(report.request.signals[0].name, "identity");
        assert_eq!(report.request.signals[1].name, "runtime");
        assert_eq!(report.request.findings.len(), 2);
        assert_eq!(report.assessment.action, DecisionAction::Deny);
    }

    #[tokio::test]
    async fn evaluate_returns_backend_error_when_fallback_disabled() {
        let mut manifest = remote_http_manifest("http://127.0.0.1:9/infer");
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec!["http://127.0.0.1:9".to_owned()];
        manifest.spec.fallback.enabled = false;

        let engine = AiEngine::new(
            registry_with(manifest),
            Box::new(StaticContextProvider),
            vec![],
            Box::new(PassthroughPolicy),
        );

        let err = engine
            .evaluate(
                AiTask::ValidatorAdmission,
                AiMode::Enforced,
                "validator-fallback-off",
            )
            .await
            .expect_err("backend error must be returned directly");

        assert!(matches!(err, AiError::BackendUnreachable(_)));
    }

    #[tokio::test]
    async fn evaluate_applies_fallback_mapping_for_schema_errors() {
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
        manifest.spec.fallback.enabled = true;
        manifest.spec.fallback.action_on_schema_error = BackendFailureAction::Deny;

        let engine = AiEngine::new(
            registry_with(manifest),
            Box::new(StaticContextProvider),
            vec![],
            Box::new(PassthroughPolicy),
        );

        let report = engine
            .evaluate(
                AiTask::ValidatorAdmission,
                AiMode::Enforced,
                "validator-schema-fallback",
            )
            .await
            .expect("fallback must convert schema error into report");
        mock.assert();

        assert_eq!(report.model_output.backend, "fallback");
        assert_eq!(report.assessment.action, DecisionAction::Deny);
    }

    #[tokio::test]
    async fn evaluate_applies_fallback_mapping_for_unreachable_backend_errors() {
        let mut manifest = remote_http_manifest("http://127.0.0.1:9/infer");
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec!["http://127.0.0.1:9".to_owned()];
        manifest.spec.fallback.enabled = true;
        manifest.spec.fallback.action_on_unreachable_backend = BackendFailureAction::Allow;

        let engine = AiEngine::new(
            registry_with(manifest),
            Box::new(StaticContextProvider),
            vec![],
            Box::new(PassthroughPolicy),
        );

        let report = engine
            .evaluate(
                AiTask::ValidatorAdmission,
                AiMode::Enforced,
                "validator-unreachable-fallback",
            )
            .await
            .expect("fallback must convert unreachable error into report");

        assert_eq!(report.model_output.backend, "fallback");
        assert_eq!(report.assessment.action, DecisionAction::Allow);
    }

    #[tokio::test]
    async fn evaluate_applies_fallback_mapping_for_generic_backend_failures() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(500).body("error");
        });

        let mut manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec![server.base_url()];
        manifest.spec.fallback.enabled = true;
        manifest.spec.fallback.action_on_backend_error = BackendFailureAction::Deny;

        let engine = AiEngine::new(
            registry_with(manifest),
            Box::new(StaticContextProvider),
            vec![],
            Box::new(PassthroughPolicy),
        );

        let report = engine
            .evaluate(
                AiTask::ValidatorAdmission,
                AiMode::Enforced,
                "validator-generic-fallback",
            )
            .await
            .expect("fallback must convert generic backend error into report");
        mock.assert();

        assert_eq!(report.model_output.backend, "fallback");
        assert_eq!(report.assessment.action, DecisionAction::Deny);
    }

    #[tokio::test]
    async fn evaluate_propagates_policy_decision_failures() {
        let engine = AiEngine::new(
            registry_with(base_manifest()),
            Box::new(StaticContextProvider),
            vec![],
            Box::new(FailingPolicy),
        );

        let err = engine
            .evaluate(
                AiTask::ValidatorAdmission,
                AiMode::Enforced,
                "validator-policy-fail",
            )
            .await
            .expect_err("policy failure must propagate");

        assert_eq!(
            err,
            AiError::PolicyFailure("policy execution failed".to_owned())
        );
    }

    #[tokio::test]
    async fn evaluate_merges_multiple_signal_providers_deterministically() {
        let mut manifest = base_manifest();
        manifest.spec.input.max_signal_count = 3;
        let engine = AiEngine::new(
            registry_with(manifest),
            Box::new(StaticContextProvider),
            vec![
                Box::new(StaticSignalProvider {
                    signals: vec![
                        InferenceSignal::new("a", "value-a", 300, "provider-a"),
                        InferenceSignal::new("c", "value-c", 800, "provider-a"),
                    ],
                }),
                Box::new(StaticSignalProvider {
                    signals: vec![
                        InferenceSignal::new("b", "value-b", 1200, "provider-b"),
                        InferenceSignal::new("d", "value-d", 100, "provider-b"),
                    ],
                }),
            ],
            Box::new(PassthroughPolicy),
        );

        let report = engine
            .evaluate(
                AiTask::ValidatorAdmission,
                AiMode::Enforced,
                "validator-signal-merge",
            )
            .await
            .expect("evaluation must succeed");

        let names = report
            .request
            .signals
            .iter()
            .map(|signal| signal.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["b", "c", "a"]);
    }

    #[tokio::test]
    async fn evaluate_preserves_decision_report_metadata_and_integrity() {
        let server = MockServer::start();
        let response_body = json!({
            "backend": "remote_http",
            "model_id": "validator-risk-v1",
            "label": "review",
            "risk_bps": 3333,
            "confidence_bps": 7777,
            "rationale": "metadata integrity probe",
            "recommended_action": "review",
            "attributes": {"trace_id": "trace-integrity"}
        });
        let mock = server.mock(|when, then| {
            when.method(POST).path("/infer");
            then.status(200)
                .header("content-type", "application/json")
                .json_body_obj(&response_body);
        });

        let mut manifest = remote_http_manifest(format!("{}/infer", server.base_url()));
        manifest.metadata.id = "manifest-integrity-id".to_owned();
        manifest.spec.security.allow_private_networks = true;
        manifest.spec.security.allowed_endpoints = vec![server.base_url()];
        let expected_backend_type = format!("{:?}", manifest.spec.backend.r#type);

        let engine = AiEngine::new(
            registry_with(manifest),
            Box::new(StaticContextProvider),
            vec![Box::new(StaticSignalProvider {
                signals: vec![InferenceSignal::new(
                    "identity",
                    "revoked_identity",
                    9_500,
                    "provider-integrity",
                )],
            })],
            Box::new(PassthroughPolicy),
        );

        let report = engine
            .evaluate(
                AiTask::ValidatorAdmission,
                AiMode::Enforced,
                "validator-integrity",
            )
            .await
            .expect("evaluation must succeed");
        mock.assert();

        assert_eq!(report.manifest_id, "manifest-integrity-id");
        assert_eq!(report.backend_type, expected_backend_type);
        assert_eq!(report.request.context.subject_id, "validator-integrity");
        assert_eq!(report.request.task, AiTask::ValidatorAdmission);
        assert_eq!(report.model_output.backend, "remote_http");
        assert_eq!(report.model_output.risk_bps, 3333);
        assert_eq!(report.model_output.confidence_bps, 7777);
    }

    #[test]
    fn fallback_model_output_uses_backend_error_recommendation_mapping() {
        let mut manifest = base_manifest();
        manifest.spec.fallback.action_on_backend_error = BackendFailureAction::Deny;
        let output = fallback_model_output(&manifest, &AiError::BackendFailure("oops".into()));
        assert_eq!(output.backend, "fallback");
        assert_eq!(output.label, OutputLabel::Unknown);
        assert_eq!(
            output.recommended_action,
            Some(crate::model::ActionName::Deny)
        );
    }
}

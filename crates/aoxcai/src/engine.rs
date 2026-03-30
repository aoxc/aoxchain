// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::{
    backend::factory::BackendFactory,
    error::AiError,
    manifest::{BackendFailureAction, ModelManifest},
    model::{
        AiMode, AiTask, Assessment, DecisionAction, DecisionReport, FindingSeverity,
        InferenceFinding, InferenceRequest,
    },
    registry::ModelRegistry,
    traits::{ContextProvider, DecisionPolicy, SignalProvider},
};

/// Runtime AI engine.
///
/// The engine is intentionally manifest-driven. It resolves a task binding,
/// normalizes context and signals, executes a validated backend, and fuses the
/// result using a deterministic policy.
///
/// Failure handling is conservative. Backend failures may be mapped into
/// manifest-declared fallback actions rather than being silently ignored.
pub struct AiEngine {
    registry: ModelRegistry,
    context_provider: Box<dyn ContextProvider>,
    signal_providers: Vec<Box<dyn SignalProvider>>,
    policy: Box<dyn DecisionPolicy>,
}

impl AiEngine {
    pub fn new(
        registry: ModelRegistry,
        context_provider: Box<dyn ContextProvider>,
        signal_providers: Vec<Box<dyn SignalProvider>>,
        policy: Box<dyn DecisionPolicy>,
    ) -> Self {
        Self {
            registry,
            context_provider,
            signal_providers,
            policy,
        }
    }

    pub async fn evaluate(
        &self,
        task: AiTask,
        mode: AiMode,
        subject_id: impl Into<String>,
    ) -> Result<DecisionReport, AiError> {
        let subject_id = subject_id.into();

        if subject_id.trim().is_empty() {
            return Err(AiError::InvalidInput(
                "subject identifier must not be empty".to_owned(),
            ));
        }

        let manifest = self.registry.resolve_for_task(task)?;
        ensure_task_supported(manifest, task, mode)?;

        let context = self.context_provider.build(task, &subject_id).await?;
        let mut signals = Vec::new();

        for provider in &self.signal_providers {
            let mut batch = provider.collect(task, &subject_id).await?;
            signals.append(&mut batch);
        }

        truncate_signals(manifest, &mut signals);
        let findings = deterministic_findings(&signals);
        let narrative = build_narrative(
            manifest,
            task,
            mode,
            &context.subject_kind,
            &subject_id,
            &signals,
            &findings,
        );

        let request = InferenceRequest {
            task,
            mode,
            context,
            signals,
            findings: findings.clone(),
            narrative,
        };

        let backend = BackendFactory::build(manifest)?;
        match backend.infer(manifest, &request).await {
            Ok(output) => {
                let assessment = self
                    .policy
                    .decide(manifest, &request, &output, &findings)
                    .await?;

                Ok(DecisionReport {
                    request,
                    model_output: output,
                    assessment,
                    manifest_id: manifest.metadata.id.clone(),
                    backend_type: format!("{:?}", manifest.spec.backend.r#type),
                })
            }
            Err(err) => {
                if !manifest.spec.fallback.enabled {
                    return Err(err);
                }

                let assessment = fallback_assessment(manifest, &err, &findings);
                let output = fallback_model_output(manifest, &err);

                Ok(DecisionReport {
                    request,
                    model_output: output,
                    assessment,
                    manifest_id: manifest.metadata.id.clone(),
                    backend_type: format!("{:?}", manifest.spec.backend.r#type),
                })
            }
        }
    }
}

fn ensure_task_supported(
    manifest: &ModelManifest,
    task: AiTask,
    mode: AiMode,
) -> Result<(), AiError> {
    if !manifest.spec.compatibility.supported_tasks.contains(&task) {
        return Err(AiError::ManifestValidation(format!(
            "task '{task:?}' is not supported by manifest '{}'",
            manifest.metadata.id
        )));
    }

    if !manifest.spec.compatibility.supported_modes.contains(&mode) {
        return Err(AiError::ManifestValidation(format!(
            "mode '{mode:?}' is not supported by manifest '{}'",
            manifest.metadata.id
        )));
    }

    Ok(())
}

fn truncate_signals(manifest: &ModelManifest, signals: &mut Vec<crate::model::InferenceSignal>) {
    signals.sort_by(|a, b| b.weight_bps.cmp(&a.weight_bps));
    if signals.len() > manifest.spec.input.max_signal_count {
        signals.truncate(manifest.spec.input.max_signal_count);
    }
}

fn deterministic_findings(signals: &[crate::model::InferenceSignal]) -> Vec<InferenceFinding> {
    let mut findings = Vec::new();

    for signal in signals {
        let value = signal.value.to_ascii_lowercase();

        if value.contains("revoked") {
            findings.push(InferenceFinding::new(
                "revoked_identity",
                "Subject exhibits a revoked identity signal.",
                FindingSeverity::Critical,
            ));
        } else if value.contains("invalid_quorum") {
            findings.push(InferenceFinding::new(
                "invalid_quorum_proof",
                "Subject exhibits an invalid quorum proof signal.",
                FindingSeverity::Critical,
            ));
        } else if value.contains("timeout") || value.contains("anomaly") {
            findings.push(InferenceFinding::new(
                "runtime_anomaly",
                "Subject exhibits a runtime anomaly signal.",
                FindingSeverity::Warning,
            ));
        }
    }

    findings
}

fn build_narrative(
    manifest: &ModelManifest,
    task: AiTask,
    mode: AiMode,
    subject_kind: &str,
    subject_id: &str,
    signals: &[crate::model::InferenceSignal],
    findings: &[InferenceFinding],
) -> Option<String> {
    if !manifest.spec.input.include_narrative {
        return None;
    }

    let signals_narrative = signals
        .iter()
        .map(|signal| {
            format!(
                "{}={} (weight_bps={})",
                signal.name, signal.value, signal.weight_bps
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let findings_narrative = findings
        .iter()
        .map(|finding| {
            format!(
                "{} [{:?}] {}",
                finding.code, finding.severity, finding.message
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    Some(format!(
        "Task: {task:?}\nMode: {mode:?}\nSubjectKind: {subject_kind}\nSubjectId: {subject_id}\n\nSignals:\n{signals_narrative}\n\nFindings:\n{findings_narrative}"
    ))
}

fn fallback_assessment(
    manifest: &ModelManifest,
    err: &AiError,
    findings: &[InferenceFinding],
) -> Assessment {
    let action_name = if err.is_timeout() {
        manifest.spec.fallback.action_on_timeout
    } else if err.is_schema_error() {
        manifest.spec.fallback.action_on_schema_error
    } else if err.is_backend_unreachable() {
        manifest.spec.fallback.action_on_unreachable_backend
    } else {
        manifest.spec.fallback.action_on_backend_error
    };

    let action: DecisionAction = action_name.into();
    let effective_risk_bps = match action {
        DecisionAction::Allow => 0,
        DecisionAction::Review => 5_000,
        DecisionAction::Deny => 10_000,
    };

    Assessment {
        action,
        effective_risk_bps,
        confidence_bps: 0,
        rationale: format!(
            "Fallback assessment applied due to backend failure: {}. deterministic_findings={}.",
            err,
            findings.len()
        ),
    }
}

fn fallback_model_output(manifest: &ModelManifest, err: &AiError) -> crate::model::ModelOutput {
    crate::model::ModelOutput {
        backend: "fallback".to_owned(),
        model_id: manifest.metadata.id.clone(),
        label: crate::model::OutputLabel::Unknown,
        risk_bps: 0,
        confidence_bps: 0,
        rationale: format!(
            "Fallback model output synthesized due to backend failure: {}.",
            err
        ),
        recommended_action: Some(match manifest.spec.fallback.action_on_backend_error {
            BackendFailureAction::Allow => crate::model::ActionName::Allow,
            BackendFailureAction::Review => crate::model::ActionName::Review,
            BackendFailureAction::Deny => crate::model::ActionName::Deny,
        }),
        attributes: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ModelRegistry,
        model::{InferenceContext, InferenceSignal, OutputLabel},
        test_support::base_manifest,
        traits::{ContextProvider, DecisionPolicy, SignalProvider},
    };

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

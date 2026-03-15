use crate::{
    backend::factory::BackendFactory,
    error::AiError,
    manifest::ModelManifest,
    model::{AiMode, AiTask, DecisionReport, InferenceFinding, InferenceRequest},
    registry::ModelRegistry,
    traits::{ContextProvider, DecisionPolicy, SignalProvider},
};

/// Runtime AI engine.
///
/// This engine is manifest-driven. It resolves a task binding to a concrete
/// model manifest, constructs a normalized inference request, invokes the
/// selected backend, and applies a deterministic fusion policy to the result.
pub struct AiEngine {
    registry: ModelRegistry,
    context_provider: Box<dyn ContextProvider>,
    signal_providers: Vec<Box<dyn SignalProvider>>,
    policy: Box<dyn DecisionPolicy>,
}

impl AiEngine {
    /// Creates a new AI engine with the supplied registry and providers.
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

    /// Evaluates the supplied subject under the given task and mode.
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
        self.ensure_task_supported(manifest, task, mode)?;

        let context = self.context_provider.build(task, &subject_id).await?;
        let mut signals = Vec::new();

        for provider in &self.signal_providers {
            let mut batch = provider.collect(task, &subject_id).await?;
            signals.append(&mut batch);
        }

        truncate_signals(manifest, &mut signals);

        let findings = deterministic_findings(&signals);
        let narrative = build_narrative(manifest, task, mode, &context.subject_kind, &subject_id, &signals, &findings);

        let request = InferenceRequest {
            task,
            mode,
            context,
            signals,
            findings: findings.clone(),
            narrative,
        };

        let backend = BackendFactory::build(manifest)?;
        let output = backend.infer(manifest, &request).await?;
        let assessment = self
            .policy
            .decide(manifest, &request, &output, &findings)
            .await?;

        Ok(DecisionReport {
            request,
            model_output: output,
            assessment,
            manifest_id: manifest.metadata.id.clone(),
            backend_type: manifest.spec.backend.r#type.clone(),
        })
    }

    fn ensure_task_supported(
        &self,
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
}

fn truncate_signals(manifest: &ModelManifest, signals: &mut Vec<crate::model::InferenceSignal>) {
    signals.sort_by(|a, b| b.weight_bps.cmp(&a.weight_bps));

    if signals.len() > manifest.spec.input.max_signal_count {
        signals.truncate(manifest.spec.input.max_signal_count);
    }
}

fn deterministic_findings(
    signals: &[crate::model::InferenceSignal],
) -> Vec<InferenceFinding> {
    let mut findings = Vec::new();

    for signal in signals {
        let value = signal.value.to_ascii_lowercase();

        if value.contains("revoked") {
            findings.push(InferenceFinding::new(
                "revoked_identity",
                "Subject exhibits a revoked identity signal.",
                "critical",
            ));
        } else if value.contains("invalid_quorum") {
            findings.push(InferenceFinding::new(
                "invalid_quorum_proof",
                "Subject exhibits an invalid quorum proof signal.",
                "critical",
            ));
        } else if value.contains("timeout") || value.contains("anomaly") {
            findings.push(InferenceFinding::new(
                "runtime_anomaly",
                "Subject exhibits a runtime anomaly signal.",
                "warning",
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
        .map(|signal| format!("{}={} (weight_bps={})", signal.name, signal.value, signal.weight_bps))
        .collect::<Vec<_>>()
        .join("\n");

    let findings_narrative = findings
        .iter()
        .map(|finding| format!("{} [{}] {}", finding.code, finding.severity, finding.message))
        .collect::<Vec<_>>()
        .join("\n");

    Some(format!(
        "Task: {task:?}\nMode: {mode:?}\nSubjectKind: {subject_kind}\nSubjectId: {subject_id}\n\nSignals:\n{signals_narrative}\n\nFindings:\n{findings_narrative}"
    ))
}

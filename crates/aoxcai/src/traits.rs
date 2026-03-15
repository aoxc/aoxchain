use crate::{
    error::AiError,
    manifest::ModelManifest,
    model::{
        Assessment, InferenceContext, InferenceFinding, InferenceRequest, InferenceSignal,
        ModelOutput,
    },
};

/// Supplies normalized signals for a given request domain.
///
/// Providers must return deterministic signals suitable for both heuristic and
/// model-backed backends.
#[async_trait::async_trait]
pub trait SignalProvider: Send + Sync {
    /// Returns a stable provider name for auditability.
    fn name(&self) -> &'static str;

    /// Collects signals for the supplied subject identifier and task.
    async fn collect(
        &self,
        task: crate::model::AiTask,
        subject_id: &str,
    ) -> Result<Vec<InferenceSignal>, AiError>;
}

/// Supplies contextual metadata for the subject under evaluation.
#[async_trait::async_trait]
pub trait ContextProvider: Send + Sync {
    /// Returns a stable provider name for auditability.
    fn name(&self) -> &'static str;

    /// Builds contextual metadata for the supplied task and subject.
    async fn build(
        &self,
        task: crate::model::AiTask,
        subject_id: &str,
    ) -> Result<InferenceContext, AiError>;
}

/// Executes a model or heuristic backend selected by manifest.
#[async_trait::async_trait]
pub trait InferenceBackend: Send + Sync {
    /// Returns the backend identifier used for telemetry and reporting.
    fn name(&self) -> &'static str;

    /// Executes inference for the supplied request using the provided manifest.
    async fn infer(
        &self,
        manifest: &ModelManifest,
        request: &InferenceRequest,
    ) -> Result<ModelOutput, AiError>;
}

/// Produces the final decision by combining model output and deterministic state.
#[async_trait::async_trait]
pub trait DecisionPolicy: Send + Sync {
    /// Returns a stable policy name.
    fn name(&self) -> &'static str;

    /// Produces the final assessment from the normalized model output.
    async fn decide(
        &self,
        manifest: &ModelManifest,
        request: &InferenceRequest,
        output: &ModelOutput,
        findings: &[InferenceFinding],
    ) -> Result<Assessment, AiError>;
}

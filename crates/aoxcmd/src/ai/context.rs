use aoxcai::{
    error::AiError,
    model::{AiTask, InferenceContext},
    traits::ContextProvider,
};
use async_trait::async_trait;

/// Minimal no-op context provider used when the AI runtime is disabled.
///
/// This provider exists solely to satisfy engine construction in disabled mode.
/// It must never participate in active inference flows.
pub struct NoopContextProvider;

impl NoopContextProvider {
    /// Creates a new no-op context provider instance.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ContextProvider for NoopContextProvider {
    fn name(&self) -> &'static str {
        "noop_context_provider"
    }

    async fn build(
        &self,
        _task: AiTask,
        subject_id: &str,
    ) -> Result<InferenceContext, AiError> {
        Ok(InferenceContext::new(subject_id, "unknown"))
    }
}

/// Node-backed context provider.
///
/// This initial implementation is intentionally conservative and returns a
/// normalized subject shell. It can later be extended with deterministic
/// metadata derived from `aoxcore` state, identity material, peer context,
/// transaction attributes, or artifact descriptors.
pub struct NodeContextProvider;

impl NodeContextProvider {
    /// Creates a new node-backed context provider instance.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ContextProvider for NodeContextProvider {
    fn name(&self) -> &'static str {
        "node_context_provider"
    }

    async fn build(
        &self,
        task: AiTask,
        subject_id: &str,
    ) -> Result<InferenceContext, AiError> {
        Ok(InferenceContext::new(subject_id, subject_kind_for_task(task)))
    }
}

/// Resolves the normalized subject kind associated with a task.
///
/// This mapping provides a stable classification boundary between node-level
/// orchestration and AI inference input construction.
fn subject_kind_for_task(task: AiTask) -> &'static str {
    match task {
        AiTask::ValidatorAdmission => "validator",
        AiTask::PeerScreening => "peer",
        AiTask::TransactionScreening => "transaction",
        AiTask::ArtifactInspection => "artifact",
    }
}


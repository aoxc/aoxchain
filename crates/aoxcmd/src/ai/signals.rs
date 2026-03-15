use aoxcai::{
    error::AiError,
    model::{AiTask, InferenceSignal},
    traits::SignalProvider,
};
use async_trait::async_trait;

/// Node-backed signal provider.
///
/// This initial implementation intentionally emits no inference signals.
/// Future iterations should derive deterministic evidence from validated
/// node state, handshake metadata, revocation registries, quorum proofs,
/// mempool state, and other authenticated local inputs.
pub struct NodeSignalProvider;

impl NodeSignalProvider {
    /// Creates a new node-backed signal provider instance.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl SignalProvider for NodeSignalProvider {
    fn name(&self) -> &'static str {
        "node_signal_provider"
    }

    async fn collect(
        &self,
        _task: AiTask,
        _subject_id: &str,
    ) -> Result<Vec<InferenceSignal>, AiError> {
        Ok(Vec::new())
    }
}


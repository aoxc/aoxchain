use aoxcai::{
    engine::AiEngine,
    error::AiError,
    model::{AiMode, AiTask, DecisionReport},
    policy::fusion::FusionPolicy,
    registry::ModelRegistry,
    traits::{ContextProvider, DecisionPolicy, SignalProvider},
};

use crate::ai::{
    context::{NodeContextProvider, NoopContextProvider},
    signals::NodeSignalProvider,
};

/// Node-local AOXC AI runtime adapter.
///
/// This adapter composes manifest loading, provider wiring, and policy
/// initialization into a single entry point consumable by `aoxcmd`.
pub struct AoxcAiRuntime {
    engine: AiEngine,
    enabled: bool,
}

impl AoxcAiRuntime {
    /// Creates a fully enabled AI runtime.
    pub fn bootstrap(manifest_dir: impl AsRef<std::path::Path>) -> Result<Self, AiError> {
        let registry = ModelRegistry::new().load_dir(manifest_dir)?;
        let context_provider: Box<dyn ContextProvider> = Box::new(NodeContextProvider::new());
        let signal_providers: Vec<Box<dyn SignalProvider>> =
            vec![Box::new(NodeSignalProvider::new())];
        let policy: Box<dyn DecisionPolicy> = Box::new(FusionPolicy::new());

        let engine = AiEngine::new(registry, context_provider, signal_providers, policy);

        Ok(Self {
            engine,
            enabled: true,
        })
    }

    /// Creates a disabled runtime for nodes that do not opt into AI features.
    pub fn disabled() -> Self {
        let registry = ModelRegistry::new();
        let context_provider: Box<dyn ContextProvider> = Box::new(NoopContextProvider::new());
        let signal_providers: Vec<Box<dyn SignalProvider>> = Vec::new();
        let policy: Box<dyn DecisionPolicy> = Box::new(FusionPolicy::new());

        let engine = AiEngine::new(registry, context_provider, signal_providers, policy);

        Self {
            engine,
            enabled: false,
        }
    }

    /// Returns whether the runtime is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Evaluates a subject when the runtime is enabled.
    ///
    /// Disabled runtimes return `Ok(None)` and perform no AI work.
    pub async fn evaluate(
        &self,
        task: AiTask,
        mode: AiMode,
        subject_id: impl Into<String>,
    ) -> Result<Option<DecisionReport>, AiError> {
        if !self.enabled {
            return Ok(None);
        }

        let report = self.engine.evaluate(task, mode, subject_id).await?;
        Ok(Some(report))
    }

    /// Returns the composed engine reference.
    pub fn engine(&self) -> &AiEngine {
        &self.engine
    }
}

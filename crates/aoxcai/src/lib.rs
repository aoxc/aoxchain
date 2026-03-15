//! AOXC AI runtime.
//!
//! This crate provides a runtime-loadable AI orchestration layer driven by
//! declarative model manifests. Backends are selected through configuration
//! rather than compile-time coupling, allowing the node to adopt new models
//! with minimal operational friction.

pub mod backend;
pub mod engine;
pub mod error;
pub mod manifest;
pub mod model;
pub mod policy;
pub mod registry;
pub mod traits;

pub use engine::AiEngine;
pub use error::AiError;
pub use manifest::ModelManifest;
pub use model::{
    AiMode, AiTask, Assessment, DecisionAction, DecisionReport, InferenceContext, InferenceFinding,
    InferenceRequest, InferenceSignal, ModelOutput,
};
pub use registry::ModelRegistry;
pub use traits::{ContextProvider, DecisionPolicy, InferenceBackend, SignalProvider};

#[cfg(test)]
mod test_support;

//! AOXC AI runtime.
//!
//! This crate provides a manifest-driven AI orchestration layer for AOXChain.
//! The design goal is operational determinism, explicit policy enforcement, and
//! conservative failure handling suitable for security-sensitive node workflows.

pub mod backend;
pub mod engine;
pub mod error;
pub mod manifest;
pub mod model;
pub mod policy;
pub mod registry;
pub mod traits;

#[cfg(test)]
mod test_support;

pub use engine::AiEngine;
pub use error::AiError;
pub use manifest::ModelManifest;
pub use model::{
    ActionName, AiMode, AiTask, Assessment, DecisionAction, DecisionReport, FindingSeverity,
    InferenceContext, InferenceFinding, InferenceRequest, InferenceSignal, ModelOutput,
    OutputLabel,
};
pub use policy::fusion::FusionPolicy;
pub use registry::ModelRegistry;
pub use traits::{ContextProvider, DecisionPolicy, InferenceBackend, SignalProvider};

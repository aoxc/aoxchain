//! AOXC intelligence extension plane.
//!
//! # Purpose
//! `aoxcai` provides a policy-constrained, auditable, capability-scoped
//! extension framework for AI-assisted workflows inside AOXChain. It exists to
//! support advisory and guarded-preparation use cases such as diagnostics
//! explanation, incident summarization, remediation planning, contract review,
//! and other non-authoritative operator workflows.
//!
//! # Non-goals
//! `aoxcai` does not execute chain logic, participate in consensus, mutate
//! canonical state, accept or reject contracts as authority, validate blocks as
//! canonical truth, or replace any native kernel decision path.
//!
//! # Trust model
//! Upstream callers must perform their own native validation before invoking AI.
//! AI output is treated as an untrusted advisory artifact and never as
//! constitutional truth.
//!
//! # Security model
//! Every invocation must be capability-scoped, zone-aware, action-class-bound,
//! policy-gated, and auditable. Denied invocations produce explicit audit traces.
//! AI failures are expected to degrade assistance only; they must not affect
//! deterministic correctness.
//!
//! # Architectural position
//! `aoxcai` is an extension plane. It is not a kernel substrate, not a root
//! authority layer, and not a required dependency for kernel correctness. Kernel
//! crates remain sovereign and AI-independent.

pub mod adapter;
pub mod audit;
pub mod backend;
pub mod capability;
pub mod constitution;
pub mod engine;
pub mod error;
pub mod extension;
pub mod manifest;
pub mod model;
pub mod policy;
pub mod registry;
pub mod traits;

#[cfg(test)]
mod test_support;

pub use adapter::AdapterInvocation;
pub use audit::{
    AiAuditSink, AiInvocationAuditRecord, InvocationDisposition, MemoryAuditSink, NoopAuditSink,
};
pub use engine::AiEngine;
pub use error::AiError;
pub use extension::{AuthorizedInvocation, ExecutionBudget, ExtensionDescriptor};
pub use manifest::ModelManifest;
pub use model::{
    ActionName, AiMode, AiTask, Assessment, DecisionAction, DecisionReport, FindingSeverity,
    InferenceContext, InferenceFinding, InferenceRequest, InferenceSignal, ModelOutput,
    OutputLabel,
};
pub use policy::fusion::FusionPolicy;
pub use registry::ModelRegistry;
pub use traits::{ContextProvider, DecisionPolicy, InferenceBackend, SignalProvider};

pub use capability::{AiActionClass, AiCapability, CapabilityGrant, InvocationPolicy, KernelZone};
pub use constitution::{CONSTITUTIONAL_RULES, authorize_invocation};

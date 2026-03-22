//! Audit-grade invocation records and pluggable audit sinks for AI extension calls.
//!
//! # Purpose
//! This module defines the canonical audit artifact for AI-assisted extension
//! invocations within AOXChain.
//!
//! It provides:
//! - a structured invocation audit record,
//! - a disposition model for allowed, denied, and fallback outcomes, and
//! - pluggable audit sinks that allow invocation evidence to be surfaced to
//!   operator tooling, diagnostics, and future persistence layers.
//!
//! # Security posture
//! AI invocation activity is security-relevant.
//! Both successful and denied invocations must be auditable so that operator,
//! policy, and forensic workflows can reconstruct what occurred.
//!
//! # Design intent
//! The audit structures in this module are designed to remain lightweight,
//! serializable, and stable enough to serve as first-class evidence artifacts
//! across test, runtime, and operator-facing workflows.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::capability::{AiActionClass, AiCapability, KernelZone};

/// Final disposition of an AI invocation authorization or execution path.
///
/// # Interpretation
/// - `Allowed`: the invocation was explicitly permitted.
/// - `Denied`: the invocation was explicitly rejected.
/// - `Fallback`: the invocation did not proceed as normal and a bounded fallback
///   path was used instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvocationDisposition {
    Allowed,
    Denied,
    Fallback,
}

/// Structured audit artifact for a single AI invocation.
///
/// # Purpose
/// This record captures the authorization and execution context needed to
/// understand what was requested, who requested it, under which policy it was
/// evaluated, and how the invocation concluded.
///
/// # Security note
/// This record is evidence, not authority. It must not be treated as canonical
/// truth about the kernel state; rather, it is an auditable trace of AI-related
/// system behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiInvocationAuditRecord {
    pub invocation_id: String,
    pub caller_crate: String,
    pub caller_component: String,
    pub requested_action: String,
    pub provider_name: String,
    pub capability: AiCapability,
    pub action_class: AiActionClass,
    pub kernel_zone: KernelZone,
    pub policy_id: String,
    pub input_class: String,
    pub output_class: String,
    pub confidence_bps: u16,
    pub duration_ms: u64,
    pub timeout_hit: bool,
    pub side_effect_intent: bool,
    pub approval_state: String,
    pub final_disposition: InvocationDisposition,
    pub recorded_at: DateTime<Utc>,
}

impl AiInvocationAuditRecord {
    /// Constructs a new audit record with conservative default metadata.
    ///
    /// # Defaults
    /// The default shape intentionally assumes:
    /// - structured input,
    /// - advisory artifact output,
    /// - zero confidence until explicitly set,
    /// - no timeout,
    /// - no side-effect intent unless declared otherwise.
    ///
    /// # Usage guidance
    /// Callers should enrich the returned record with final execution metadata
    /// where appropriate before persisting or emitting it.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        invocation_id: impl Into<String>,
        caller_crate: impl Into<String>,
        caller_component: impl Into<String>,
        requested_action: impl Into<String>,
        provider_name: impl Into<String>,
        capability: AiCapability,
        action_class: AiActionClass,
        kernel_zone: KernelZone,
        policy_id: impl Into<String>,
    ) -> Self {
        Self {
            invocation_id: invocation_id.into(),
            caller_crate: caller_crate.into(),
            caller_component: caller_component.into(),
            requested_action: requested_action.into(),
            provider_name: provider_name.into(),
            capability,
            action_class,
            kernel_zone,
            policy_id: policy_id.into(),
            input_class: "structured".into(),
            output_class: "advisory_artifact".into(),
            confidence_bps: 0,
            duration_ms: 0,
            timeout_hit: false,
            side_effect_intent: false,
            approval_state: "not_required".into(),
            final_disposition: InvocationDisposition::Allowed,
            recorded_at: Utc::now(),
        }
    }
}

/// Sink abstraction for AI invocation audit artifacts.
///
/// # Design intent
/// This trait decouples audit record production from audit record storage or
/// emission. It allows AOXChain subsystems to route audit evidence into
/// memory-backed testing sinks, no-op sinks, or future persistence/event layers
/// without changing authorization logic.
///
/// # Security note
/// The existence of a sink does not weaken the requirement to emit audit
/// artifacts. It only abstracts where those artifacts are sent.
pub trait AiAuditSink: Send + Sync {
    fn record(&self, record: AiInvocationAuditRecord);
}

/// In-memory audit sink primarily intended for tests and bounded runtime capture.
///
/// # Concurrency note
/// Internal storage is protected by a mutex because multiple authorization paths
/// may emit audit records concurrently.
#[derive(Debug, Default, Clone)]
pub struct MemoryAuditSink {
    records: Arc<Mutex<Vec<AiInvocationAuditRecord>>>,
}

impl MemoryAuditSink {
    /// Returns a point-in-time snapshot of all captured records.
    ///
    /// # Panic behavior
    /// This method will panic if the internal audit mutex is poisoned.
    /// Such a condition indicates a serious internal synchronization failure and
    /// should be treated as a runtime integrity problem rather than silently ignored.
    #[must_use]
    pub fn snapshot(&self) -> Vec<AiInvocationAuditRecord> {
        self.records.lock().expect("audit mutex poisoned").clone()
    }
}

impl AiAuditSink for MemoryAuditSink {
    fn record(&self, record: AiInvocationAuditRecord) {
        self.records
            .lock()
            .expect("audit mutex poisoned")
            .push(record);
    }
}

/// No-op audit sink for explicitly silent capture environments.
///
/// # Usage guidance
/// This sink should be used only where dropping audit records is an intentional
/// architectural decision, such as in narrow test scaffolding or explicitly
/// non-persistent environments.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopAuditSink;

impl AiAuditSink for NoopAuditSink {
    fn record(&self, _record: AiInvocationAuditRecord) {}
}

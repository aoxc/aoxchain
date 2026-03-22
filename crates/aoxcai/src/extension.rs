//! Stable extension-plane descriptors and policy-bound registration.
//!
//! # Purpose
//! This module defines the stable authorization boundary between AOXChain
//! callers and AI-backed extension providers.
//!
//! It provides:
//! - bounded execution-budget descriptors,
//! - extension metadata describing what an AI-backed module is allowed to do,
//! - an authorization result type for approved invocations, and
//! - explicit authorization flows that preserve audit evidence for both allowed
//!   and denied requests.
//!
//! # Design intent
//! The extension plane must remain policy-constrained and auditable.
//! AI-backed extensions may be integrated broadly across AOXChain, but they must
//! never operate outside explicit authorization boundaries.
//!
//! # Security posture
//! Authorization must produce reviewable evidence.
//! In particular, denied invocations are operationally significant and should
//! produce explicit audit records whenever possible rather than disappearing as
//! plain errors.

use serde::{Deserialize, Serialize};

use crate::{
    audit::{AiInvocationAuditRecord, InvocationDisposition},
    capability::{AiActionClass, AiCapability, InvocationPolicy, KernelZone},
    constitution::authorize_invocation,
    error::AiError,
};

/// Bounded execution constraints for an AI-backed extension invocation.
///
/// # Security note
/// These limits are part of the extension control plane and should be treated
/// as enforceable runtime constraints rather than advisory metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionBudget {
    /// Maximum execution time budget in milliseconds.
    pub timeout_ms: u64,

    /// Maximum memory budget available to the invocation.
    pub max_memory_bytes: u64,

    /// Maximum serialized output size permitted for the invocation.
    pub max_output_bytes: u64,
}

impl Default for ExecutionBudget {
    fn default() -> Self {
        Self {
            timeout_ms: 1_000,
            max_memory_bytes: 8 * 1024 * 1024,
            max_output_bytes: 64 * 1024,
        }
    }
}

/// Describes a policy-bound AI extension registration.
///
/// # Design intent
/// This structure identifies a concrete extension/provider pair together with
/// the kernel zone, capability, action class, and budget under which it may be
/// considered for authorization.
///
/// # Security note
/// Presence of an `ExtensionDescriptor` does not itself grant permission to run.
/// Authorization remains policy-gated and constitutionally constrained.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionDescriptor {
    /// Stable logical identifier for the extension.
    pub id: String,

    /// Provider name backing the extension implementation.
    pub provider_name: String,

    /// Kernel zone in which this extension is intended to operate.
    pub zone: KernelZone,

    /// Capability requested by this extension.
    pub capability: AiCapability,

    /// Constitutional sensitivity of the requested action.
    pub action_class: AiActionClass,

    /// Bounded execution budget for the invocation.
    pub budget: ExecutionBudget,
}

/// Represents a successfully authorized AI invocation.
///
/// # Security note
/// Authorization is represented explicitly so downstream execution paths can
/// require prior approval as a typed precondition rather than re-checking
/// authorization ad hoc.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedInvocation {
    pub descriptor: ExtensionDescriptor,
    pub audit_record: AiInvocationAuditRecord,
}

impl ExtensionDescriptor {
    /// Attempts to authorize this extension invocation and always produces an
    /// explicit audit outcome.
    ///
    /// # Behavior
    /// - On success, returns an `AuthorizedInvocation` carrying an audit record
    ///   whose final disposition is `Allowed`.
    /// - On failure, returns a boxed audit record whose final disposition is
    ///   `Denied`, preserving a structured failure trace for operator and audit use.
    ///
    /// # Rationale
    /// Denied authorization events are security-relevant and operationally
    /// meaningful. Returning an audit record on denial preserves evidence that
    /// a prohibited invocation was attempted and explicitly blocked.
    ///
    /// # Security note
    /// This method does not execute the extension. It only performs
    /// authorization and emits the corresponding audit artifact.
    pub fn attempt_authorize(
        &self,
        policy: &InvocationPolicy,
        caller_crate: impl Into<String>,
        caller_component: impl Into<String>,
        requested_action: impl Into<String>,
    ) -> Result<AuthorizedInvocation, Box<AiInvocationAuditRecord>> {
        let caller_crate = caller_crate.into();
        let caller_component = caller_component.into();
        let requested_action = requested_action.into();

        match authorize_invocation(policy, self.zone, self.capability, self.action_class) {
            Ok(()) => {
                let mut audit = AiInvocationAuditRecord::new(
                    format!("{}:{}", self.id, self.provider_name),
                    caller_crate,
                    caller_component,
                    requested_action,
                    self.provider_name.clone(),
                    self.capability,
                    self.action_class,
                    self.zone,
                    policy.policy_id.clone(),
                );
                audit.final_disposition = InvocationDisposition::Allowed;

                Ok(AuthorizedInvocation {
                    descriptor: self.clone(),
                    audit_record: audit,
                })
            }
            Err(reason) => {
                let mut audit = AiInvocationAuditRecord::new(
                    format!("denied:{}:{}", self.id, self.provider_name),
                    caller_crate,
                    caller_component,
                    requested_action,
                    self.provider_name.clone(),
                    self.capability,
                    self.action_class,
                    self.zone,
                    policy.policy_id.clone(),
                );
                audit.final_disposition = InvocationDisposition::Denied;
                audit.approval_state = reason;

                Err(Box::new(audit))
            }
        }
    }

    /// Authorizes this extension invocation and returns a simplified domain error on failure.
    ///
    /// # Behavior
    /// This method is a convenience wrapper over `attempt_authorize()`.
    /// It preserves the successful authorization path while collapsing denied
    /// authorization into `AiError::PolicyFailure`.
    ///
    /// # Usage guidance
    /// Use this method where the caller needs a simple error-oriented control
    /// flow. Use `attempt_authorize()` where denied attempts must remain visible
    /// as first-class audit artifacts.
    pub fn authorize(
        &self,
        policy: &InvocationPolicy,
        caller_crate: impl Into<String>,
        caller_component: impl Into<String>,
        requested_action: impl Into<String>,
    ) -> Result<AuthorizedInvocation, AiError> {
        self.attempt_authorize(policy, caller_crate, caller_component, requested_action)
            .map_err(|audit| AiError::PolicyFailure(audit.approval_state.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{AiActionClass, AiCapability, InvocationPolicy, KernelZone};

    #[test]
    fn authorization_respects_policy_boundary() {
        let descriptor = ExtensionDescriptor {
            id: "contract-review".into(),
            provider_name: "heuristic".into(),
            zone: KernelZone::Contract,
            capability: AiCapability::ManifestReview,
            action_class: AiActionClass::Advisory,
            budget: ExecutionBudget::default(),
        };

        let authorized = descriptor
            .authorize(
                &InvocationPolicy::kernel_default(),
                "aoxcontract",
                "review-adapter",
                "manifest_review",
            )
            .unwrap();

        assert_eq!(authorized.audit_record.caller_crate, "aoxcontract");
        assert_eq!(authorized.audit_record.final_disposition, InvocationDisposition::Allowed);
    }

    #[test]
    fn denied_authorization_emits_explicit_failure_trace() {
        let descriptor = ExtensionDescriptor {
            id: "consensus-review".into(),
            provider_name: "heuristic".into(),
            zone: KernelZone::Consensus,
            capability: AiCapability::RunbookGenerate,
            action_class: AiActionClass::GuardedPreparation,
            budget: ExecutionBudget::default(),
        };

        let denied = descriptor
            .attempt_authorize(
                &InvocationPolicy::kernel_default(),
                "aoxcmd",
                "doctor",
                "consensus_repair",
            )
            .unwrap_err();

        assert_eq!(denied.final_disposition, InvocationDisposition::Denied);
        assert!(denied.approval_state.contains("does not allow"));
    }

    #[test]
    fn budget_defaults_serialize_stably() {
        let encoded = serde_json::to_string(&ExecutionBudget::default()).unwrap();
        assert!(encoded.contains("timeout_ms"));
        assert!(encoded.contains("max_memory_bytes"));
        assert!(encoded.contains("max_output_bytes"));
    }
}
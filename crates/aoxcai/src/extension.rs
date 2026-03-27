// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! Stable extension-plane descriptors and policy-bound registration.
//!
//! # Purpose
//! This module defines the stable authorization boundary between AOXChain
//! callers and AI-backed extension providers.
//!
//! It provides:
//! - execution budgets,
//! - extension descriptors,
//! - explicit authorization flows,
//! - audit-aware invocation handling.
//!
//! # Security posture
//! AI invocation must always be:
//! - policy-checked,
//! - capability-scoped,
//! - auditable (allowed AND denied).
//!
//! This module does NOT execute logic. It only authorizes.

use serde::{Deserialize, Serialize};

use crate::{
    audit::{AiInvocationAuditRecord, InvocationDisposition},
    capability::{AiActionClass, AiCapability, InvocationPolicy, KernelZone},
    constitution::authorize_invocation,
    error::AiError,
};

/// Execution limits for AI invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionBudget {
    pub timeout_ms: u64,
    pub max_memory_bytes: u64,
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

/// AI extension descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionDescriptor {
    pub id: String,
    pub provider_name: String,
    pub zone: KernelZone,
    pub capability: AiCapability,
    pub action_class: AiActionClass,
    pub budget: ExecutionBudget,
}

/// Authorized invocation wrapper.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedInvocation {
    pub descriptor: ExtensionDescriptor,
    pub audit_record: AiInvocationAuditRecord,
}

impl ExtensionDescriptor {
    /// Attempts authorization and ALWAYS produces audit evidence.
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

    /// Ergonomic wrapper.
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

        assert_eq!(
            authorized.audit_record.final_disposition,
            InvocationDisposition::Allowed
        );
        assert_eq!(authorized.audit_record.policy_id, "aoxcai-kernel-default");
        assert_eq!(authorized.audit_record.output_class, "advisory_artifact");
        assert_eq!(authorized.audit_record.kernel_zone, KernelZone::Contract);
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
        assert!(!denied.approval_state.is_empty());
        assert_eq!(denied.policy_id, "aoxcai-kernel-default");
    }
}

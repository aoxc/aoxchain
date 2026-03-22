//! Stable extension-plane descriptors and policy-bound registration.

use serde::{Deserialize, Serialize};

use crate::{
    audit::{AiInvocationAuditRecord, InvocationDisposition},
    capability::{AiActionClass, AiCapability, InvocationPolicy, KernelZone},
    constitution::authorize_invocation,
    error::AiError,
};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtensionDescriptor {
    pub id: String,
    pub provider_name: String,
    pub zone: KernelZone,
    pub capability: AiCapability,
    pub action_class: AiActionClass,
    pub budget: ExecutionBudget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedInvocation {
    pub descriptor: ExtensionDescriptor,
    pub audit_record: AiInvocationAuditRecord,
}

impl ExtensionDescriptor {
    pub fn authorize(
        &self,
        policy: &InvocationPolicy,
        caller_crate: impl Into<String>,
        caller_component: impl Into<String>,
        requested_action: impl Into<String>,
    ) -> Result<AuthorizedInvocation, AiError> {
        authorize_invocation(policy, self.zone, self.capability, self.action_class)
            .map_err(AiError::PolicyFailure)?;

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
    }
}

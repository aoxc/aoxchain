// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! Constitutional rules for keeping AOXC kernel-first and AI-extensible.

use serde::{Deserialize, Serialize};

use crate::capability::{AiActionClass, AiCapability, InvocationPolicy, KernelZone};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstitutionalRule {
    pub id: &'static str,
    pub description: &'static str,
}

pub const CONSTITUTIONAL_RULES: &[ConstitutionalRule] = &[
    ConstitutionalRule {
        id: "AI_NOT_ROOT_AUTHORITY",
        description: "AI may advise broadly but never becomes the default sovereign authority.",
    },
    ConstitutionalRule {
        id: "DETERMINISTIC_KERNEL",
        description: "Deterministic kernel paths must remain valid without AI availability.",
    },
    ConstitutionalRule {
        id: "AI_OUTPUT_NOT_CANONICAL_TRUTH",
        description: "AI output is an evaluated artifact, never canonical truth by itself.",
    },
    ConstitutionalRule {
        id: "CAPABILITY_SCOPED",
        description: "Every AI invocation must be capability-scoped.",
    },
    ConstitutionalRule {
        id: "POLICY_GATED_SIDE_EFFECTS",
        description: "Every AI side effect must be policy-gated.",
    },
    ConstitutionalRule {
        id: "AUDIT_REQUIRED",
        description: "Every AI invocation must emit an audit record.",
    },
    ConstitutionalRule {
        id: "FAIL_OPEN_ON_ASSISTANCE",
        description: "Loss of AI should reduce assistance, not kernel correctness.",
    },
    ConstitutionalRule {
        id: "CONSTITUTIONAL_ZONES_RESTRICTED",
        description: "Constitutional zones permit advisory behavior only unless explicitly policy-bound.",
    },
];

pub fn authorize_invocation(
    policy: &InvocationPolicy,
    zone: KernelZone,
    capability: AiCapability,
    action_class: AiActionClass,
) -> Result<(), String> {
    if action_class == AiActionClass::RestrictedConstitutional {
        return Err("restricted constitutional actions are never AI-authoritative".to_string());
    }

    if !policy.allows(zone, capability, action_class) {
        return Err(format!(
            "policy '{}' does not allow capability '{capability:?}' in zone '{zone:?}' for action class '{action_class:?}'",
            policy.policy_id
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{AiActionClass, AiCapability, InvocationPolicy, KernelZone};

    #[test]
    fn restricted_constitutional_action_is_rejected() {
        let err = authorize_invocation(
            &InvocationPolicy::kernel_default(),
            KernelZone::Core,
            AiCapability::Explain,
            AiActionClass::RestrictedConstitutional,
        )
        .unwrap_err();
        assert!(err.contains("never AI-authoritative"));
    }

    #[test]
    fn advisory_contract_review_is_allowed() {
        authorize_invocation(
            &InvocationPolicy::kernel_default(),
            KernelZone::Contract,
            AiCapability::ManifestReview,
            AiActionClass::Advisory,
        )
        .unwrap();
    }
}

//! Capability and policy descriptors for the AOXC intelligence extension plane.
//!
//! # Purpose
//! This module defines the authorization vocabulary used to constrain AI
//! behavior inside AOXChain.
//!
//! It provides:
//! - capability identifiers describing bounded assistance types,
//! - kernel zone identifiers describing invocation origin,
//! - action classes describing constitutional sensitivity, and
//! - policy structures binding capabilities to zones and action classes.
//!
//! # Security model
//! AI access is never implicit. Every invocation must be:
//! - explicitly granted,
//! - policy-checked,
//! - capability-scoped,
//! - auditable.
//!
//! # Design intent
//! These types are stable, serializable policy artifacts forming the core
//! authorization layer of the AOXC AI extension plane.

use serde::{Deserialize, Serialize};

/// Bounded AI capabilities.
///
/// # Important
/// Capabilities define *assistance scope*, not authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiCapability {
    Explain,
    ValidateAssist,
    InvariantCheckAssist,

    ManifestReview,
    CompatibilityLint,
    RiskSummary,

    DiagnosticsAssist,
    IncidentSummary,

    ConfigReview,
    RemediationPlan,
    RunbookGenerate,
}

/// Kernel zones representing invocation origin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelZone {
    Core,
    Consensus,
    Network,
    Contract,
    Operator,
}

/// Constitutional sensitivity class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiActionClass {
    Advisory,
    GuardedPreparation,
    RestrictedConstitutional,
}

/// Explicit policy grant.
///
/// Authorization requires exact match:
/// zone + capability + action_class
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub zone: KernelZone,
    pub capability: AiCapability,
    pub action_class: AiActionClass,
}

/// Invocation policy definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvocationPolicy {
    pub policy_id: String,
    pub grants: Vec<CapabilityGrant>,
    pub audit_required: bool,
}

impl InvocationPolicy {
    #[must_use]
    pub fn kernel_default() -> Self {
        Self {
            policy_id: "aoxcai-kernel-default".to_string(),
            grants: vec![
                // Core
                CapabilityGrant {
                    zone: KernelZone::Core,
                    capability: AiCapability::Explain,
                    action_class: AiActionClass::Advisory,
                },
                // Consensus
                CapabilityGrant {
                    zone: KernelZone::Consensus,
                    capability: AiCapability::InvariantCheckAssist,
                    action_class: AiActionClass::Advisory,
                },
                // Contract
                CapabilityGrant {
                    zone: KernelZone::Contract,
                    capability: AiCapability::ManifestReview,
                    action_class: AiActionClass::Advisory,
                },
                CapabilityGrant {
                    zone: KernelZone::Contract,
                    capability: AiCapability::CompatibilityLint,
                    action_class: AiActionClass::Advisory,
                },
                // Network
                CapabilityGrant {
                    zone: KernelZone::Network,
                    capability: AiCapability::RiskSummary,
                    action_class: AiActionClass::Advisory,
                },
                // Operator - advisory
                CapabilityGrant {
                    zone: KernelZone::Operator,
                    capability: AiCapability::DiagnosticsAssist,
                    action_class: AiActionClass::Advisory,
                },
                CapabilityGrant {
                    zone: KernelZone::Operator,
                    capability: AiCapability::IncidentSummary,
                    action_class: AiActionClass::Advisory,
                },
                CapabilityGrant {
                    zone: KernelZone::Operator,
                    capability: AiCapability::ConfigReview,
                    action_class: AiActionClass::Advisory,
                },
                // Operator - guarded
                CapabilityGrant {
                    zone: KernelZone::Operator,
                    capability: AiCapability::RemediationPlan,
                    action_class: AiActionClass::GuardedPreparation,
                },
                CapabilityGrant {
                    zone: KernelZone::Operator,
                    capability: AiCapability::RunbookGenerate,
                    action_class: AiActionClass::GuardedPreparation,
                },
            ],
            audit_required: true,
        }
    }

    #[must_use]
    pub fn allows(
        &self,
        zone: KernelZone,
        capability: AiCapability,
        action_class: AiActionClass,
    ) -> bool {
        self.grants
            .iter()
            .any(|g| g.zone == zone && g.capability == capability && g.action_class == action_class)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_CAPABILITIES: [AiCapability; 11] = [
        AiCapability::Explain,
        AiCapability::ValidateAssist,
        AiCapability::InvariantCheckAssist,
        AiCapability::ManifestReview,
        AiCapability::CompatibilityLint,
        AiCapability::RiskSummary,
        AiCapability::DiagnosticsAssist,
        AiCapability::IncidentSummary,
        AiCapability::ConfigReview,
        AiCapability::RemediationPlan,
        AiCapability::RunbookGenerate,
    ];

    const ALL_ZONES: [KernelZone; 5] = [
        KernelZone::Core,
        KernelZone::Consensus,
        KernelZone::Network,
        KernelZone::Contract,
        KernelZone::Operator,
    ];

    const ALL_ACTION_CLASSES: [AiActionClass; 3] = [
        AiActionClass::Advisory,
        AiActionClass::GuardedPreparation,
        AiActionClass::RestrictedConstitutional,
    ];

    #[test]
    fn kernel_default_policy_requires_audit_and_has_stable_grants() {
        let policy = InvocationPolicy::kernel_default();
        assert!(policy.audit_required);
        assert_eq!(policy.policy_id, "aoxcai-kernel-default");
        assert_eq!(policy.grants.len(), 10);
    }

    #[test]
    fn kernel_default_policy_allows_every_declared_grant() {
        let policy = InvocationPolicy::kernel_default();
        for grant in &policy.grants {
            assert!(policy.allows(grant.zone, grant.capability, grant.action_class));
        }
    }

    #[test]
    fn kernel_default_policy_denies_every_non_granted_combination() {
        let policy = InvocationPolicy::kernel_default();

        for zone in ALL_ZONES {
            for capability in ALL_CAPABILITIES {
                for action_class in ALL_ACTION_CLASSES {
                    let expected = policy.grants.iter().any(|grant| {
                        grant.zone == zone
                            && grant.capability == capability
                            && grant.action_class == action_class
                    });
                    assert_eq!(policy.allows(zone, capability, action_class), expected);
                }
            }
        }
    }

    #[test]
    fn exact_match_semantics_reject_capability_zone_and_action_mutations() {
        let policy = InvocationPolicy::kernel_default();

        assert!(policy.allows(
            KernelZone::Operator,
            AiCapability::RunbookGenerate,
            AiActionClass::GuardedPreparation,
        ));
        assert!(!policy.allows(
            KernelZone::Operator,
            AiCapability::RunbookGenerate,
            AiActionClass::Advisory,
        ));
        assert!(!policy.allows(
            KernelZone::Consensus,
            AiCapability::RunbookGenerate,
            AiActionClass::GuardedPreparation,
        ));
        assert!(!policy.allows(
            KernelZone::Operator,
            AiCapability::DiagnosticsAssist,
            AiActionClass::GuardedPreparation,
        ));
    }
}

// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! Capability and policy descriptors for the AOXC intelligence extension plane.
//!
//! # Purpose
//! This module defines the canonical authorization vocabulary used to constrain
//! AI-assisted operations within AOXChain.
//!
//! It provides:
//! - capability identifiers describing bounded assistance scopes,
//! - kernel zone identifiers describing invocation origin,
//! - action classes describing constitutional sensitivity, and
//! - policy structures binding capabilities to zones and action classes.
//!
//! # Security Model
//! AI access is never implicit. Every invocation must be:
//! - explicitly granted,
//! - policy-validated,
//! - capability-scoped, and
//! - auditable.
//!
//! # Design Intent
//! These types are intended to be stable, serializable policy artifacts that
//! form the core authorization layer of the AOXC AI extension plane.
//!
//! The authorization model is intentionally exact-match based. A request is
//! authorized only when the tuple `(zone, capability, action_class)` is granted
//! by policy without wildcard expansion, inheritance, or implicit escalation.

use serde::{Deserialize, Serialize};

/// Enumerates bounded AI assistance capabilities.
///
/// # Security Consideration
/// Capability values define the permitted *scope of assistance* only.
/// They do not independently confer execution authority, privilege
/// elevation, or constitutional legitimacy.
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

/// Identifies the kernel zone from which an AI-assisted invocation originates.
///
/// # Security Consideration
/// Zone classification is part of the authorization boundary. The same
/// capability may be acceptable in one zone and denied in another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelZone {
    Core,
    Consensus,
    Network,
    Contract,
    Operator,
}

/// Defines the constitutional sensitivity of an AI-assisted action.
///
/// # Classification Semantics
/// - `Advisory`: non-authoritative assistance with no direct constitutional effect.
/// - `GuardedPreparation`: preparatory assistance requiring heightened control.
/// - `RestrictedConstitutional`: constitutionally sensitive actions requiring
///   stronger restriction and explicit governance treatment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiActionClass {
    Advisory,
    GuardedPreparation,
    RestrictedConstitutional,
}

/// Represents an explicit authorization grant for a single exact-match tuple.
///
/// # Authorization Rule
/// A request is authorized only when all three dimensions match exactly:
/// - zone
/// - capability
/// - action_class
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub zone: KernelZone,
    pub capability: AiCapability,
    pub action_class: AiActionClass,
}

/// Defines an invocation policy consisting of explicit grants and audit controls.
///
/// # Security Consideration
/// Policies are deny-by-default. Any tuple not explicitly present in `grants`
/// must be treated as unauthorized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvocationPolicy {
    pub policy_id: String,
    pub grants: Vec<CapabilityGrant>,
    pub audit_required: bool,
}

impl InvocationPolicy {
    /// Returns the default kernel policy for the AOXC AI extension plane.
    ///
    /// # Policy Characteristics
    /// This policy is intentionally conservative:
    /// - advisory permissions are narrowly scoped by zone,
    /// - operator preparation flows are separated into guarded grants,
    /// - no constitutional action is implicitly authorized, and
    /// - audit logging is mandatory.
    #[must_use]
    pub fn kernel_default() -> Self {
        Self {
            policy_id: "aoxcai-kernel-default".to_string(),
            grants: vec![
                CapabilityGrant {
                    zone: KernelZone::Core,
                    capability: AiCapability::Explain,
                    action_class: AiActionClass::Advisory,
                },
                CapabilityGrant {
                    zone: KernelZone::Consensus,
                    capability: AiCapability::InvariantCheckAssist,
                    action_class: AiActionClass::Advisory,
                },
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
                CapabilityGrant {
                    zone: KernelZone::Network,
                    capability: AiCapability::RiskSummary,
                    action_class: AiActionClass::Advisory,
                },
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

    /// Returns `true` only if the provided authorization tuple is explicitly granted.
    ///
    /// # Security Semantics
    /// This function implements strict exact-match authorization. It does not
    /// support partial matching, wildcarding, fallback behavior, or privilege
    /// inheritance across zones, capabilities, or action classes.
    #[must_use]
    pub fn allows(
        &self,
        zone: KernelZone,
        capability: AiCapability,
        action_class: AiActionClass,
    ) -> bool {
        self.grants.iter().any(|grant| {
            grant.zone == zone
                && grant.capability == capability
                && grant.action_class == action_class
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Full capability universe used to verify deny-by-default behavior for all
    /// non-granted combinations.
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

    /// Full zone universe used to validate exact-match authorization semantics.
    const ALL_ZONES: [KernelZone; 5] = [
        KernelZone::Core,
        KernelZone::Consensus,
        KernelZone::Network,
        KernelZone::Contract,
        KernelZone::Operator,
    ];

    /// Full action-class universe used to confirm that ungranted constitutional
    /// classes remain denied.
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

//! Capability and policy descriptors for the AOXC intelligence extension plane.
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
        self.grants.iter().any(|g| {
            g.zone == zone
                && g.capability == capability
                && g.action_class == action_class
        })
    }
}

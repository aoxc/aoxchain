//! Capability and policy descriptors for the AOXC intelligence extension plane.

use serde::{Deserialize, Serialize};

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
    RunbookGenerate,
    RemediationPlan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelZone {
    Core,
    Consensus,
    Network,
    Contract,
    Operator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiActionClass {
    Advisory,
    GuardedPreparation,
    RestrictedConstitutional,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub zone: KernelZone,
    pub capability: AiCapability,
    pub action_class: AiActionClass,
}

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
        self.grants.iter().any(|grant| {
            grant.zone == zone
                && grant.capability == capability
                && grant.action_class == action_class
        })
    }
}

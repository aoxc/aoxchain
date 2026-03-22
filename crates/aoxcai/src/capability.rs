//! Capability and policy descriptors for the AOXC intelligence extension plane.
//!
//! # Purpose
//! This module defines the authorization vocabulary used to constrain AI
//! behavior inside AOXChain.
//!
//! It provides:
//! - capability identifiers describing what kind of assistance may be requested,
//! - kernel zone identifiers describing where the request originates,
//! - action classes describing the constitutional sensitivity of the request, and
//! - policy structures that explicitly bind allowed capabilities to specific
//!   zones and action classes.
//!
//! # Security model
//! AI access in AOXChain must never be implicit.
//! Every permitted AI action must be represented as an explicit, reviewable, and
//! auditable policy grant.
//!
//! # Design intent
//! The types in this module are deliberately small, serializable, and stable so
//! they can serve as policy artifacts across configuration, audit, testing, and
//! runtime authorization flows.

use serde::{Deserialize, Serialize};

/// Enumerates the bounded AI capabilities that may be requested by AOXChain
/// subsystems.
///
/// # Design note
/// These capabilities describe *classes of assistance*, not authority.
/// Granting a capability permits a bounded AI function to be invoked; it does
/// not delegate constitutional decision-making to AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiCapability {
    /// Produces human-readable explanation or interpretation of existing system state.
    Explain,

    /// Assists with validation-oriented reasoning without becoming the
    /// canonical validation authority.
    ValidateAssist,

    /// Assists with invariant analysis or invariant-oriented review.
    InvariantCheckAssist,

    /// Reviews a contract manifest and produces a non-authoritative assessment.
    ManifestReview,

    /// Produces compatibility-focused findings for manifests or integration targets.
    CompatibilityLint,

    /// Produces a bounded risk-oriented summary for operator or subsystem review.
    RiskSummary,

    /// Assists operators during diagnostics and incident inspection workflows.
    DiagnosticsAssist,

    /// Generates or drafts operator-facing runbook material.
    RunbookGenerate,

    /// Produces remediation proposals or corrective action drafts for review.
    RemediationPlan,
}

/// Identifies the kernel zone in which an AI invocation is requested.
///
/// # Security note
/// Zone classification is part of the authorization boundary. The same
/// capability may be acceptable in one zone and prohibited in another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelZone {
    /// Core protocol or domain-adjacent logic.
    Core,

    /// Consensus and finality-sensitive logic.
    Consensus,

    /// Network, admission, or communication-related logic.
    Network,

    /// Contract, manifest, registry, or lifecycle-related logic.
    Contract,

    /// Operator-facing tooling and control-plane workflows.
    Operator,
}

/// Describes the constitutional sensitivity of an AI-assisted action.
///
/// # Interpretation
/// - `Advisory`: AI may explain, summarize, assess, or recommend without
///   preparing a state-changing artifact.
/// - `GuardedPreparation`: AI may prepare an artifact or plan, but downstream
///   approval or explicit execution control remains mandatory.
/// - `RestrictedConstitutional`: AI must never be treated as an authority for
///   the underlying action. At most, inspection and explanatory behavior are allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiActionClass {
    Advisory,
    GuardedPreparation,
    RestrictedConstitutional,
}

/// A single explicit policy grant authorizing one AI capability in one zone
/// for one action class.
///
/// # Security note
/// This structure is intentionally narrow. Authorization is granted only when
/// all three dimensions match:
/// - zone,
/// - capability,
/// - action class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub zone: KernelZone,
    pub capability: AiCapability,
    pub action_class: AiActionClass,
}

/// Defines the explicit authorization policy applied to AI invocation requests.
///
/// # Design intent
/// Policies are intended to be reviewable artifacts that can be reasoned about
/// during audit, testing, and operational sign-off.
///
/// # Security note
/// `audit_required` is modeled as policy state because the requirement to emit
/// audit evidence is itself part of the authorization contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvocationPolicy {
    pub policy_id: String,
    pub grants: Vec<CapabilityGrant>,
    pub audit_required: bool,
}

impl InvocationPolicy {
    /// Returns the default kernel-safe AI invocation policy.
    ///
    /// # Policy goals
    /// This default policy is intentionally conservative:
    /// - advisory access is limited to low-risk explanatory and review-oriented use cases,
    /// - guarded preparation is limited to operator-plane runbook generation, and
    /// - no restricted constitutional action is granted here.
    ///
    /// # Security posture
    /// This policy is a baseline allowlist, not a broad entitlement model.
    /// Callers requiring additional permissions should introduce them explicitly
    /// and review them as part of architectural governance.
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
                    capability: AiCapability::Explain,
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

    /// Returns `true` if the policy explicitly authorizes the supplied tuple.
    ///
    /// # Authorization semantics
    /// Authorization is exact-match based. A request is allowed only if a grant
    /// exists for the same:
    /// - kernel zone,
    /// - AI capability, and
    /// - action class.
    ///
    /// # Security note
    /// This method does not apply constitutional overrides by itself. Higher
    /// layers may impose stricter rules even when a matching grant exists.
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
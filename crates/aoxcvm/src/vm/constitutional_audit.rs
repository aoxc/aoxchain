//! Constitutional telemetry and audit-decision primitives for Phase-3 runtime law.
//!
//! This module defines deterministic, structured records for policy/governance/
//! auth-profile decisions so upstream telemetry, receipt, and governance systems
//! can consume one canonical audit surface.

use crate::{
    auth::signer::SignerClass,
    policy::governance::GovernanceLane,
    vm::{
        admission::ActiveAuthProfile,
        constitutional_runtime::{ConstitutionalProvenance, RuntimeSurface},
    },
};

/// Decision outcome recorded by constitutional telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditDecisionOutcome {
    Allowed,
    Denied,
}

/// Typed constitutional decision record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstitutionalDecision {
    pub surface: RuntimeSurface,
    pub outcome: AuditDecisionOutcome,
    pub reason_code: &'static str,
    pub signer_class: &'static str,
    pub governance_lane: &'static str,
    pub auth_profile_id: Option<u32>,
    pub auth_profile_version: Option<u16>,
}

impl ConstitutionalDecision {
    /// Creates an "allowed" decision from execution-time provenance.
    pub fn allowed(provenance: &ConstitutionalProvenance, reason_code: &'static str) -> Self {
        Self {
            surface: provenance.surface,
            outcome: AuditDecisionOutcome::Allowed,
            reason_code,
            signer_class: provenance.signer_class.wire_id(),
            governance_lane: lane_wire_id(provenance.governance_lane),
            auth_profile_id: provenance.auth_profile_id,
            auth_profile_version: provenance.auth_profile_version,
        }
    }

    /// Creates a "denied" decision from explicit authority/profile context.
    pub fn denied(
        surface: RuntimeSurface,
        signer_class: SignerClass,
        lane: GovernanceLane,
        active_profile: Option<&ActiveAuthProfile>,
        reason_code: &'static str,
    ) -> Self {
        Self {
            surface,
            outcome: AuditDecisionOutcome::Denied,
            reason_code,
            signer_class: signer_class.wire_id(),
            governance_lane: lane_wire_id(lane),
            auth_profile_id: active_profile.map(|p| p.profile_id.as_u32()),
            auth_profile_version: active_profile.map(|p| p.profile_version),
        }
    }
}

/// Deterministic in-memory recorder for constitutional audit decisions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConstitutionalAuditLog {
    decisions: Vec<ConstitutionalDecision>,
}

impl ConstitutionalAuditLog {
    pub fn record(&mut self, decision: ConstitutionalDecision) {
        self.decisions.push(decision);
    }

    pub fn decisions(&self) -> &[ConstitutionalDecision] {
        &self.decisions
    }

    pub fn into_decisions(self) -> Vec<ConstitutionalDecision> {
        self.decisions
    }
}

const fn lane_wire_id(lane: GovernanceLane) -> &'static str {
    match lane {
        GovernanceLane::Constitutional => "constitutional",
        GovernanceLane::Operations => "operations",
        GovernanceLane::Emergency => "emergency",
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::{registry::AuthProfileId, signer::SignerClass},
        host::capability_check::HostOperation,
        policy::governance::{GovernanceAction, GovernanceLane},
        vm::{
            admission::ActiveAuthProfile,
            constitutional_audit::{
                AuditDecisionOutcome, ConstitutionalAuditLog, ConstitutionalDecision,
            },
            constitutional_runtime::{ConstitutionalProvenance, RuntimeSurface},
        },
    };

    #[test]
    fn allowed_decision_inherits_provenance_identity() {
        let provenance = ConstitutionalProvenance {
            surface: RuntimeSurface::GovernanceAction(GovernanceAction::UpgradeProtocol),
            signer_class: SignerClass::Governance,
            governance_lane: GovernanceLane::Constitutional,
            auth_profile_id: Some(5),
            auth_profile_version: Some(2),
        };

        let decision = ConstitutionalDecision::allowed(&provenance, "governance_action_authorized");

        assert_eq!(decision.outcome, AuditDecisionOutcome::Allowed);
        assert_eq!(decision.signer_class, "governance");
        assert_eq!(decision.governance_lane, "constitutional");
        assert_eq!(decision.auth_profile_id, Some(5));
    }

    #[test]
    fn denied_decision_captures_context_and_profile() {
        let active_profile = ActiveAuthProfile {
            profile_id: AuthProfileId::new(9),
            profile_version: 3,
            profile_name: "ops-v1".to_string(),
            signer_class: SignerClass::Operations,
        };

        let decision = ConstitutionalDecision::denied(
            RuntimeSurface::HostOperation(HostOperation::RegistryWrite),
            SignerClass::Operations,
            GovernanceLane::Operations,
            Some(&active_profile),
            "capability_denied",
        );

        assert_eq!(decision.outcome, AuditDecisionOutcome::Denied);
        assert_eq!(decision.signer_class, "operations");
        assert_eq!(decision.governance_lane, "operations");
        assert_eq!(decision.auth_profile_id, Some(9));
    }

    #[test]
    fn audit_log_preserves_decision_order() {
        let mut log = ConstitutionalAuditLog::default();
        log.record(ConstitutionalDecision::denied(
            RuntimeSurface::UpgradeTrigger,
            SignerClass::Application,
            GovernanceLane::Operations,
            None,
            "governance_lane_violation",
        ));
        log.record(ConstitutionalDecision::denied(
            RuntimeSurface::RegistryMutation,
            SignerClass::Operations,
            GovernanceLane::Operations,
            None,
            "capability_denied",
        ));

        let decisions = log.into_decisions();
        assert_eq!(decisions.len(), 2);
        assert_eq!(decisions[0].reason_code, "governance_lane_violation");
        assert_eq!(decisions[1].reason_code, "capability_denied");
    }
}

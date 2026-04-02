//! Upgrade constitution runtime policy.

use crate::{
    errors::{AoxcvmError, AoxcvmResult},
    policy::governance::{GovernanceAuthority, GovernanceLane},
};

/// Typed upgrade action surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpgradeAction {
    ProtocolBinary,
    StateSchema,
    RuntimeParameter,
    EmergencyRollback,
}

impl UpgradeAction {
    /// Minimum governance lane required to authorize this upgrade action.
    pub const fn required_lane(self) -> GovernanceLane {
        match self {
            Self::ProtocolBinary | Self::StateSchema => GovernanceLane::Constitutional,
            Self::RuntimeParameter => GovernanceLane::Operations,
            Self::EmergencyRollback => GovernanceLane::Emergency,
        }
    }
}

/// Quorum vote summary for an upgrade decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpgradeApproval {
    pub approvals: u16,
    pub rejections: u16,
    pub vetoed: bool,
    pub min_approvals: u16,
}

impl UpgradeApproval {
    pub const fn approved(self) -> bool {
        !self.vetoed && self.approvals >= self.min_approvals
    }
}

/// Compatibility check outcome before upgrade execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompatibilityReport {
    pub backward_compatible: bool,
    pub migration_plan_attached: bool,
    pub rollback_plan_attached: bool,
}

/// Runtime upgrade policy engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UpgradePolicy;

impl UpgradePolicy {
    /// Full constitutional authorization for upgrade execution.
    pub fn authorize(
        self,
        authority: GovernanceAuthority,
        action: UpgradeAction,
        approval: UpgradeApproval,
        compatibility: CompatibilityReport,
    ) -> AoxcvmResult<()> {
        if authority.lane != action.required_lane()
            && authority.lane != GovernanceLane::Constitutional
        {
            return Err(AoxcvmError::GovernanceLaneViolation(
                "upgrade action lane requirement not satisfied",
            ));
        }

        if !approval.approved() {
            return Err(AoxcvmError::PolicyViolation(
                "upgrade approval quorum or veto policy failed",
            ));
        }

        if !compatibility.backward_compatible && !compatibility.migration_plan_attached {
            return Err(AoxcvmError::PolicyViolation(
                "non-compatible upgrade requires migration plan",
            ));
        }

        if !compatibility.rollback_plan_attached {
            return Err(AoxcvmError::PolicyViolation(
                "upgrade rollback plan is required",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::signer::SignerClass,
        policy::{
            governance::{GovernanceAuthority, GovernanceLane},
            upgrade::{CompatibilityReport, UpgradeAction, UpgradeApproval, UpgradePolicy},
        },
    };

    #[test]
    fn constitutional_upgrade_requires_quorum_compatibility_and_rollback_plan() {
        let policy = UpgradePolicy;
        let authority = GovernanceAuthority {
            signer_class: SignerClass::Governance,
            lane: GovernanceLane::Constitutional,
        };

        let result = policy.authorize(
            authority,
            UpgradeAction::ProtocolBinary,
            UpgradeApproval {
                approvals: 7,
                rejections: 1,
                vetoed: false,
                min_approvals: 5,
            },
            CompatibilityReport {
                backward_compatible: true,
                migration_plan_attached: false,
                rollback_plan_attached: true,
            },
        );

        assert!(result.is_ok());
    }

    #[test]
    fn non_compatible_upgrade_without_migration_is_denied() {
        let policy = UpgradePolicy;
        let authority = GovernanceAuthority {
            signer_class: SignerClass::Governance,
            lane: GovernanceLane::Constitutional,
        };

        let result = policy.authorize(
            authority,
            UpgradeAction::StateSchema,
            UpgradeApproval {
                approvals: 9,
                rejections: 0,
                vetoed: false,
                min_approvals: 6,
            },
            CompatibilityReport {
                backward_compatible: false,
                migration_plan_attached: false,
                rollback_plan_attached: true,
            },
        );

        assert!(result.is_err());
    }
}

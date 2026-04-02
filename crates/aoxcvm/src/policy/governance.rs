//! Governance constitution primitives: lanes, actions, and authority checks.

use crate::{
    auth::signer::SignerClass,
    errors::{AoxcvmError, AoxcvmResult},
};

/// Constitutional governance lanes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GovernanceLane {
    Constitutional,
    Operations,
    Emergency,
}

/// Typed governance action families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GovernanceAction {
    UpgradeProtocol,
    MutateRegistry,
    ActivateFeature,
    EmergencyHalt,
}

impl GovernanceAction {
    /// Required minimum lane for this action.
    pub const fn required_lane(self) -> GovernanceLane {
        match self {
            Self::UpgradeProtocol => GovernanceLane::Constitutional,
            Self::MutateRegistry => GovernanceLane::Constitutional,
            Self::ActivateFeature => GovernanceLane::Operations,
            Self::EmergencyHalt => GovernanceLane::Emergency,
        }
    }
}

/// Runtime governance authority binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GovernanceAuthority {
    pub signer_class: SignerClass,
    pub lane: GovernanceLane,
}

impl GovernanceAuthority {
    /// Verifies whether an authority can execute a governance action.
    pub fn authorize(self, action: GovernanceAction) -> AoxcvmResult<()> {
        // Lane must satisfy action requirements.
        if !lane_satisfies(self.lane, action.required_lane()) {
            return Err(AoxcvmError::GovernanceLaneViolation(
                "governance lane is insufficient for action",
            ));
        }

        // Signer class must satisfy lane privilege model.
        let class_allowed = match self.lane {
            GovernanceLane::Constitutional => matches!(
                self.signer_class,
                SignerClass::Governance | SignerClass::System
            ),
            GovernanceLane::Operations => matches!(
                self.signer_class,
                SignerClass::Governance | SignerClass::Operations | SignerClass::System
            ),
            GovernanceLane::Emergency => {
                matches!(
                    self.signer_class,
                    SignerClass::System | SignerClass::Operations
                )
            }
        };

        if !class_allowed {
            return Err(AoxcvmError::GovernanceLaneViolation(
                "signer class is not authorized for lane",
            ));
        }

        Ok(())
    }
}

const fn lane_satisfies(active: GovernanceLane, required: GovernanceLane) -> bool {
    matches!(
        (active, required),
        (GovernanceLane::Constitutional, _)
            | (GovernanceLane::Operations, GovernanceLane::Operations)
            | (GovernanceLane::Emergency, GovernanceLane::Emergency)
    )
}

#[cfg(test)]
mod tests {
    use crate::{auth::signer::SignerClass, policy::governance::GovernanceAction};

    use super::{GovernanceAuthority, GovernanceLane};

    #[test]
    fn constitutional_lane_can_authorize_protocol_upgrade() {
        let authority = GovernanceAuthority {
            signer_class: SignerClass::Governance,
            lane: GovernanceLane::Constitutional,
        };

        assert!(
            authority
                .authorize(GovernanceAction::UpgradeProtocol)
                .is_ok()
        );
    }

    #[test]
    fn operations_lane_cannot_authorize_constitutional_action() {
        let authority = GovernanceAuthority {
            signer_class: SignerClass::Operations,
            lane: GovernanceLane::Operations,
        };

        assert!(
            authority
                .authorize(GovernanceAction::MutateRegistry)
                .is_err()
        );
    }

    #[test]
    fn emergency_lane_rejects_application_signer() {
        let authority = GovernanceAuthority {
            signer_class: SignerClass::Application,
            lane: GovernanceLane::Emergency,
        };

        assert!(
            authority
                .authorize(GovernanceAction::EmergencyHalt)
                .is_err()
        );
    }
}

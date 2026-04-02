//! Governance syscall admission helpers.

use crate::{
    errors::AoxcvmResult,
    policy::{
        execution::{CapabilityContext, RuntimeCapability, RuntimeCapabilityGate},
        governance::{GovernanceAction, GovernanceAuthority},
    },
};

/// Governance syscall payload boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GovernanceSyscall {
    pub action: GovernanceAction,
    pub authority: GovernanceAuthority,
}

impl GovernanceSyscall {
    /// Validates action authority and capability gate binding.
    pub fn validate(self) -> AoxcvmResult<()> {
        self.authority.authorize(self.action)?;

        RuntimeCapabilityGate.enforce(
            RuntimeCapability::SyscallGovernance,
            CapabilityContext {
                signer_class: self.authority.signer_class,
                lane: self.authority.lane,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::signer::SignerClass,
        policy::governance::{GovernanceAction, GovernanceAuthority, GovernanceLane},
        syscall::governance::GovernanceSyscall,
    };

    #[test]
    fn governance_syscall_requires_constitutional_lane() {
        let call = GovernanceSyscall {
            action: GovernanceAction::UpgradeProtocol,
            authority: GovernanceAuthority {
                signer_class: SignerClass::Governance,
                lane: GovernanceLane::Operations,
            },
        };

        assert!(call.validate().is_err());
    }

    #[test]
    fn governance_syscall_accepts_constitutional_authority() {
        let call = GovernanceSyscall {
            action: GovernanceAction::UpgradeProtocol,
            authority: GovernanceAuthority {
                signer_class: SignerClass::Governance,
                lane: GovernanceLane::Constitutional,
            },
        };

        assert!(call.validate().is_ok());
    }
}

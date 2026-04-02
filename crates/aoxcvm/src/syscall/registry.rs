//! Registry syscall admission helpers for constitutional mutations.

use crate::{
    errors::AoxcvmResult,
    policy::{
        execution::{CapabilityContext, RuntimeCapability, RuntimeCapabilityGate},
        governance::GovernanceAuthority,
    },
};

/// Registry mutation syscall envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryMutationSyscall {
    pub authority: GovernanceAuthority,
}

impl RegistryMutationSyscall {
    /// Enforces registry mutation capability at runtime.
    pub fn validate(self) -> AoxcvmResult<()> {
        RuntimeCapabilityGate.enforce(
            RuntimeCapability::RegistryMutation,
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
        policy::governance::{GovernanceAuthority, GovernanceLane},
        syscall::registry::RegistryMutationSyscall,
    };

    #[test]
    fn registry_mutation_denied_for_operations_lane() {
        let call = RegistryMutationSyscall {
            authority: GovernanceAuthority {
                signer_class: SignerClass::Governance,
                lane: GovernanceLane::Operations,
            },
        };

        assert!(call.validate().is_err());
    }

    #[test]
    fn registry_mutation_allowed_for_constitutional_lane() {
        let call = RegistryMutationSyscall {
            authority: GovernanceAuthority {
                signer_class: SignerClass::System,
                lane: GovernanceLane::Constitutional,
            },
        };

        assert!(call.validate().is_ok());
    }
}

//! Host-level capability checks delegated to runtime gate policy.

use crate::{
    errors::AoxcvmResult,
    policy::execution::{CapabilityContext, RuntimeCapability, RuntimeCapabilityGate},
};

/// Host operations that must pass constitutional capability checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostOperation {
    RestrictedFilesystemProbe,
    RegistryWrite,
    RuntimeMetadataWrite,
}

impl HostOperation {
    const fn capability(self) -> RuntimeCapability {
        match self {
            Self::RestrictedFilesystemProbe => RuntimeCapability::HostRestrictedOperation,
            Self::RegistryWrite => RuntimeCapability::RegistryMutation,
            Self::RuntimeMetadataWrite => RuntimeCapability::MetadataMutation,
        }
    }
}

/// Host capability checker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HostCapabilityChecker {
    gate: RuntimeCapabilityGate,
}

impl HostCapabilityChecker {
    pub fn check(self, operation: HostOperation, ctx: CapabilityContext) -> AoxcvmResult<()> {
        self.gate.enforce(operation.capability(), ctx)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::signer::SignerClass,
        host::capability_check::{HostCapabilityChecker, HostOperation},
        policy::execution::CapabilityContext,
        policy::governance::GovernanceLane,
    };

    #[test]
    fn registry_write_rejected_in_operations_lane() {
        let checker = HostCapabilityChecker::default();
        let result = checker.check(
            HostOperation::RegistryWrite,
            CapabilityContext {
                signer_class: SignerClass::Governance,
                lane: GovernanceLane::Operations,
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn runtime_metadata_write_allowed_for_ops_lane() {
        let checker = HostCapabilityChecker::default();
        let result = checker.check(
            HostOperation::RuntimeMetadataWrite,
            CapabilityContext {
                signer_class: SignerClass::Operations,
                lane: GovernanceLane::Operations,
            },
        );

        assert!(result.is_ok());
    }
}

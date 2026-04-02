//! Runtime capability-gate model for execution-time enforcement.

use crate::{
    auth::signer::SignerClass,
    errors::{AoxcvmError, AoxcvmResult},
    policy::governance::GovernanceLane,
};

/// Execution-time capability categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeCapability {
    SyscallGovernance,
    RegistryMutation,
    MetadataMutation,
    UpgradeAuthority,
    HostRestrictedOperation,
}

/// Bound context used when evaluating a capability gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityContext {
    pub signer_class: SignerClass,
    pub lane: GovernanceLane,
}

/// Canonical gate logic for runtime capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RuntimeCapabilityGate;

impl RuntimeCapabilityGate {
    /// Enforces capability law under signer class + lane context.
    pub fn enforce(
        self,
        capability: RuntimeCapability,
        ctx: CapabilityContext,
    ) -> AoxcvmResult<()> {
        let allowed = match capability {
            RuntimeCapability::SyscallGovernance => matches!(
                (ctx.signer_class, ctx.lane),
                (SignerClass::Governance, GovernanceLane::Constitutional)
                    | (SignerClass::System, GovernanceLane::Constitutional)
            ),
            RuntimeCapability::RegistryMutation => matches!(
                (ctx.signer_class, ctx.lane),
                (SignerClass::Governance, GovernanceLane::Constitutional)
                    | (SignerClass::System, GovernanceLane::Constitutional)
            ),
            RuntimeCapability::MetadataMutation => matches!(
                (ctx.signer_class, ctx.lane),
                (SignerClass::Governance, GovernanceLane::Operations)
                    | (SignerClass::Operations, GovernanceLane::Operations)
                    | (SignerClass::System, GovernanceLane::Constitutional)
            ),
            RuntimeCapability::UpgradeAuthority => matches!(
                (ctx.signer_class, ctx.lane),
                (SignerClass::Governance, GovernanceLane::Constitutional)
                    | (SignerClass::System, GovernanceLane::Constitutional)
            ),
            RuntimeCapability::HostRestrictedOperation => {
                !matches!(ctx.signer_class, SignerClass::Application)
            }
        };

        if allowed {
            Ok(())
        } else {
            Err(AoxcvmError::CapabilityDenied(
                "runtime capability gate denied by constitutional policy",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::signer::SignerClass,
        policy::{
            execution::{CapabilityContext, RuntimeCapability, RuntimeCapabilityGate},
            governance::GovernanceLane,
        },
    };

    #[test]
    fn registry_mutation_requires_constitutional_lane() {
        let gate = RuntimeCapabilityGate;
        let denied = gate.enforce(
            RuntimeCapability::RegistryMutation,
            CapabilityContext {
                signer_class: SignerClass::Governance,
                lane: GovernanceLane::Operations,
            },
        );
        assert!(denied.is_err());

        let allowed = gate.enforce(
            RuntimeCapability::RegistryMutation,
            CapabilityContext {
                signer_class: SignerClass::Governance,
                lane: GovernanceLane::Constitutional,
            },
        );
        assert!(allowed.is_ok());
    }

    #[test]
    fn host_restricted_ops_block_application_signer() {
        let gate = RuntimeCapabilityGate;
        let result = gate.enforce(
            RuntimeCapability::HostRestrictedOperation,
            CapabilityContext {
                signer_class: SignerClass::Application,
                lane: GovernanceLane::Operations,
            },
        );
        assert!(result.is_err());
    }
}

//! Phase-3 constitutional runtime wiring.
//!
//! This module binds governance lane law, runtime capability law, host
//! capability checks, and registry-backed auth identity into a single runtime
//! authorization surface.

use crate::{
    errors::AoxcvmResult,
    host::capability_check::{HostCapabilityChecker, HostOperation},
    policy::{
        execution::{CapabilityContext, RuntimeCapability, RuntimeCapabilityGate},
        governance::{GovernanceAction, GovernanceAuthority},
    },
    vm::admission::ActiveAuthProfile,
};

/// Runtime surfaces that are constitutionally governed at execution time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSurface {
    GovernanceAction(GovernanceAction),
    RegistryMutation,
    MetadataMutation,
    UpgradeTrigger,
    HostOperation(HostOperation),
}

/// Law-aware provenance emitted when an execution-time authorization passes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstitutionalProvenance {
    pub surface: RuntimeSurface,
    pub signer_class: crate::auth::signer::SignerClass,
    pub governance_lane: crate::policy::governance::GovernanceLane,
    pub auth_profile_id: Option<u32>,
    pub auth_profile_version: Option<u16>,
}

/// Execution-time constitutional authorizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ConstitutionalRuntime {
    gate: RuntimeCapabilityGate,
    host_checker: HostCapabilityChecker,
}

impl ConstitutionalRuntime {
    fn context(authority: GovernanceAuthority) -> CapabilityContext {
        CapabilityContext {
            signer_class: authority.signer_class,
            lane: authority.lane,
        }
    }

    fn provenance(
        surface: RuntimeSurface,
        authority: GovernanceAuthority,
        active_profile: Option<&ActiveAuthProfile>,
    ) -> ConstitutionalProvenance {
        ConstitutionalProvenance {
            surface,
            signer_class: authority.signer_class,
            governance_lane: authority.lane,
            auth_profile_id: active_profile.map(|p| p.profile_id.as_u32()),
            auth_profile_version: active_profile.map(|p| p.profile_version),
        }
    }

    /// Authorizes a typed governance action against lane + signer-class law and
    /// runtime capability gate.
    pub fn authorize_governance_action(
        self,
        authority: GovernanceAuthority,
        action: GovernanceAction,
        active_profile: Option<&ActiveAuthProfile>,
    ) -> AoxcvmResult<ConstitutionalProvenance> {
        authority.authorize(action)?;
        self.gate.enforce(
            RuntimeCapability::SyscallGovernance,
            Self::context(authority),
        )?;
        Ok(Self::provenance(
            RuntimeSurface::GovernanceAction(action),
            authority,
            active_profile,
        ))
    }

    /// Authorizes canonical registry mutation path.
    pub fn authorize_registry_mutation(
        self,
        authority: GovernanceAuthority,
        active_profile: Option<&ActiveAuthProfile>,
    ) -> AoxcvmResult<ConstitutionalProvenance> {
        authority.authorize(GovernanceAction::MutateRegistry)?;
        self.gate.enforce(
            RuntimeCapability::RegistryMutation,
            Self::context(authority),
        )?;
        Ok(Self::provenance(
            RuntimeSurface::RegistryMutation,
            authority,
            active_profile,
        ))
    }

    /// Authorizes runtime metadata mutation path.
    pub fn authorize_metadata_mutation(
        self,
        authority: GovernanceAuthority,
        active_profile: Option<&ActiveAuthProfile>,
    ) -> AoxcvmResult<ConstitutionalProvenance> {
        self.gate.enforce(
            RuntimeCapability::MetadataMutation,
            Self::context(authority),
        )?;
        Ok(Self::provenance(
            RuntimeSurface::MetadataMutation,
            authority,
            active_profile,
        ))
    }

    /// Authorizes upgrade trigger path.
    pub fn authorize_upgrade_trigger(
        self,
        authority: GovernanceAuthority,
        active_profile: Option<&ActiveAuthProfile>,
    ) -> AoxcvmResult<ConstitutionalProvenance> {
        authority.authorize(GovernanceAction::UpgradeProtocol)?;
        self.gate.enforce(
            RuntimeCapability::UpgradeAuthority,
            Self::context(authority),
        )?;
        Ok(Self::provenance(
            RuntimeSurface::UpgradeTrigger,
            authority,
            active_profile,
        ))
    }

    /// Authorizes host-bound restricted operations.
    pub fn authorize_host_operation(
        self,
        authority: GovernanceAuthority,
        op: HostOperation,
        active_profile: Option<&ActiveAuthProfile>,
    ) -> AoxcvmResult<ConstitutionalProvenance> {
        self.host_checker.check(op, Self::context(authority))?;
        Ok(Self::provenance(
            RuntimeSurface::HostOperation(op),
            authority,
            active_profile,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        auth::{registry::AuthProfileId, signer::SignerClass},
        host::capability_check::HostOperation,
        policy::governance::{GovernanceAction, GovernanceAuthority, GovernanceLane},
        vm::{
            admission::ActiveAuthProfile,
            constitutional_runtime::{ConstitutionalRuntime, RuntimeSurface},
        },
    };

    fn governance_auth() -> GovernanceAuthority {
        GovernanceAuthority {
            signer_class: SignerClass::Governance,
            lane: GovernanceLane::Constitutional,
        }
    }

    fn profile() -> ActiveAuthProfile {
        ActiveAuthProfile {
            profile_id: AuthProfileId::new(44),
            profile_version: 2,
            profile_name: "ops-v1".to_string(),
            signer_class: SignerClass::Governance,
        }
    }

    #[test]
    fn governance_action_returns_lane_and_profile_provenance() {
        let runtime = ConstitutionalRuntime::default();
        let provenance = runtime
            .authorize_governance_action(
                governance_auth(),
                GovernanceAction::UpgradeProtocol,
                Some(&profile()),
            )
            .expect("constitutional governance call should pass");

        assert_eq!(
            provenance.surface,
            RuntimeSurface::GovernanceAction(GovernanceAction::UpgradeProtocol)
        );
        assert_eq!(provenance.auth_profile_id, Some(44));
        assert_eq!(provenance.auth_profile_version, Some(2));
    }

    #[test]
    fn upgrade_is_denied_in_operations_lane() {
        let runtime = ConstitutionalRuntime::default();
        let authority = GovernanceAuthority {
            signer_class: SignerClass::Operations,
            lane: GovernanceLane::Operations,
        };

        assert!(
            runtime
                .authorize_upgrade_trigger(authority, Some(&profile()))
                .is_err()
        );
    }

    #[test]
    fn host_registry_write_denied_for_operations_lane() {
        let runtime = ConstitutionalRuntime::default();
        let authority = GovernanceAuthority {
            signer_class: SignerClass::Governance,
            lane: GovernanceLane::Operations,
        };

        assert!(
            runtime
                .authorize_host_operation(authority, HostOperation::RegistryWrite, None)
                .is_err()
        );
    }
}

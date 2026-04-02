//! Governance object class semantics.

use crate::policy::{
    execution::RuntimeCapability,
    governance::{GovernanceAction, GovernanceLane},
};

/// Governance object families introduced in constitutional runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GovernanceObjectClass {
    GovernanceContract,
    ProfileRegistryContract,
    UpgradeControllerContract,
    SettlementRouterContract,
}

impl GovernanceObjectClass {
    /// Governing lane required for mutable operations.
    pub const fn required_lane(self) -> GovernanceLane {
        match self {
            Self::GovernanceContract => GovernanceLane::Constitutional,
            Self::ProfileRegistryContract => GovernanceLane::Constitutional,
            Self::UpgradeControllerContract => GovernanceLane::Constitutional,
            Self::SettlementRouterContract => GovernanceLane::Operations,
        }
    }

    /// Canonical governance action routed by this object class.
    pub const fn default_action(self) -> GovernanceAction {
        match self {
            Self::GovernanceContract => GovernanceAction::ActivateFeature,
            Self::ProfileRegistryContract => GovernanceAction::MutateRegistry,
            Self::UpgradeControllerContract => GovernanceAction::UpgradeProtocol,
            Self::SettlementRouterContract => GovernanceAction::ActivateFeature,
        }
    }

    /// Runtime capability required to mutate this class.
    pub const fn required_capability(self) -> RuntimeCapability {
        match self {
            Self::GovernanceContract => RuntimeCapability::SyscallGovernance,
            Self::ProfileRegistryContract => RuntimeCapability::RegistryMutation,
            Self::UpgradeControllerContract => RuntimeCapability::UpgradeAuthority,
            Self::SettlementRouterContract => RuntimeCapability::MetadataMutation,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::policy::governance::GovernanceLane;

    use super::GovernanceObjectClass;

    #[test]
    fn upgrade_controller_is_constitutional() {
        assert_eq!(
            GovernanceObjectClass::UpgradeControllerContract.required_lane(),
            GovernanceLane::Constitutional
        );
    }

    #[test]
    fn settlement_router_is_ops_lane() {
        assert_eq!(
            GovernanceObjectClass::SettlementRouterContract.required_lane(),
            GovernanceLane::Operations
        );
    }
}

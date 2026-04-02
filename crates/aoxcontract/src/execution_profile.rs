// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use crate::VmTarget;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractClass {
    Application,
    System,
    Governed,
    Package,
    PolicyBound,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CapabilityProfile {
    pub storage_read: bool,
    pub storage_write: bool,
    pub package_dependency_access: bool,
    pub registry_access: bool,
    pub governance_hooks: bool,
    pub restricted_syscalls: bool,
    pub upgrade_authority: bool,
    pub metadata_mutation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PolicyProfile {
    pub review_required: bool,
    pub governance_activation_required: bool,
    pub restricted_to_auth_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionProfile {
    pub vm_target: VmTarget,
    pub contract_class: ContractClass,
    pub capability_profile: CapabilityProfile,
    pub policy_profile: PolicyProfile,
}

impl ExecutionProfile {
    pub fn phase2_default(vm_target: &VmTarget) -> Self {
        Self {
            vm_target: vm_target.clone(),
            contract_class: ContractClass::Application,
            capability_profile: CapabilityProfile {
                storage_read: true,
                ..CapabilityProfile::default()
            },
            policy_profile: PolicyProfile {
                review_required: true,
                ..PolicyProfile::default()
            },
        }
    }
}

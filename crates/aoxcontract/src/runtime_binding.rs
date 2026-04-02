// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use crate::{ContractDescriptor, ContractError, ContractId, ExecutionProfile, VmTarget};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaneBinding {
    Evm,
    Wasm,
    Sui,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ExecutionProfileRef(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeBindingDescriptor {
    pub contract_id: ContractId,
    pub vm_target: VmTarget,
    pub lane_binding: LaneBinding,
    pub execution_profile: ExecutionProfileRef,
    pub resolved_profile: ExecutionProfile,
}

impl RuntimeBindingDescriptor {
    pub fn from_descriptor(
        descriptor: &ContractDescriptor,
        lane_binding: LaneBinding,
        execution_profile: ExecutionProfileRef,
    ) -> Result<Self, ContractError> {
        Ok(Self {
            contract_id: descriptor.contract_id.clone(),
            vm_target: descriptor.manifest.vm_target.clone(),
            lane_binding,
            execution_profile,
            resolved_profile: descriptor.manifest.execution_profile.clone(),
        })
    }
}

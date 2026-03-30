// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};

use aoxcontract::{ContractDescriptor, ExecutionProfileRef, LaneBinding, RuntimeBindingDescriptor};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VmLaneBinding {
    pub binding: RuntimeBindingDescriptor,
}

impl VmLaneBinding {
    pub fn from_contract(
        descriptor: &ContractDescriptor,
        lane_binding: LaneBinding,
        execution_profile: ExecutionProfileRef,
    ) -> Self {
        Self {
            binding: RuntimeBindingDescriptor::from_descriptor(
                descriptor,
                lane_binding,
                execution_profile,
            )
            .expect("validated descriptor should produce runtime binding"),
        }
    }
}

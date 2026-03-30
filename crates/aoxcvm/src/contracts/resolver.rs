// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use aoxconfig::contracts::ContractsConfig;
use aoxcontract::{
    ContractDescriptor, ContractError, ExecutionProfileRef, LaneBinding, RuntimeBindingDescriptor,
    VmTarget,
};

pub fn resolve_lane_for_manifest(vm_target: &VmTarget) -> LaneBinding {
    match vm_target {
        VmTarget::Evm => LaneBinding::Evm,
        VmTarget::Wasm => LaneBinding::Wasm,
        VmTarget::SuiLike => LaneBinding::Sui,
        VmTarget::Custom(custom) => LaneBinding::Custom(custom.clone()),
    }
}

pub fn resolve_execution_profile(
    descriptor: &ContractDescriptor,
    config: &ContractsConfig,
) -> ExecutionProfileRef {
    let prefix = if config.artifact_policy.review_required {
        "reviewed"
    } else {
        "standard"
    };
    ExecutionProfileRef(format!("{prefix}:{}", descriptor.manifest.package))
}

pub fn resolve_runtime_binding(
    descriptor: &ContractDescriptor,
    config: &ContractsConfig,
) -> Result<RuntimeBindingDescriptor, ContractError> {
    RuntimeBindingDescriptor::from_descriptor(
        descriptor,
        resolve_lane_for_manifest(&descriptor.manifest.vm_target),
        resolve_execution_profile(descriptor, config),
    )
}

//! Contract runtime-binding compatibility surface for integration tests and VM adapters.

pub mod resolver {
    use aoxconfig::contracts::ContractsConfig;
    use aoxcontract::{
        ContractDescriptor, ContractError, ExecutionProfileRef, LaneBinding,
        RuntimeBindingDescriptor, VmTarget,
    };

    /// Resolves VM runtime binding from a canonical contract descriptor.
    pub fn resolve_runtime_binding(
        descriptor: &ContractDescriptor,
        _config: &ContractsConfig,
    ) -> Result<RuntimeBindingDescriptor, ContractError> {
        let lane = match descriptor.manifest.vm_target {
            VmTarget::Wasm => LaneBinding::Wasm,
            VmTarget::Evm => LaneBinding::Evm,
            VmTarget::SuiLike => LaneBinding::Sui,
            VmTarget::Custom(ref lane) => LaneBinding::Custom(lane.clone()),
        };

        RuntimeBindingDescriptor::from_descriptor(
            descriptor,
            lane,
            ExecutionProfileRef("phase1-default".to_string()),
        )
    }
}

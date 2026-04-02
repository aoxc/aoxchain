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
        config: &ContractsConfig,
    ) -> Result<RuntimeBindingDescriptor, ContractError> {
        if !config
            .artifact_policy
            .allowed_vm_targets
            .iter()
            .any(|target| {
                matches!(
                    (target, &descriptor.manifest.vm_target),
                    (VmTarget::Wasm, VmTarget::Wasm)
                        | (VmTarget::Evm, VmTarget::Evm)
                        | (VmTarget::SuiLike, VmTarget::SuiLike)
                        | (VmTarget::Custom(_), VmTarget::Custom(_))
                )
            })
        {
            return Err(ContractError::Policy(
                aoxcontract::PolicyValidationError::PolicyViolation(
                    "vm target disabled by contracts config".to_string(),
                ),
            ));
        }

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

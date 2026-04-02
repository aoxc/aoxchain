//! Contract runtime-binding compatibility surface for integration tests and VM adapters.

pub mod resolver {
    use aoxconfig::contracts::ContractsConfig;
    use aoxcontract::{
        ContractDescriptor, ContractError, ExecutionProfileRef, LaneBinding, PolicyValidationError,
        RuntimeBindingDescriptor, VmTarget,
    };

    /// Resolves VM runtime binding from descriptor + config in fail-closed mode.
    pub fn resolve_runtime_binding(
        descriptor: &ContractDescriptor,
        config: &ContractsConfig,
    ) -> Result<RuntimeBindingDescriptor, ContractError> {
        if !config
            .artifact_policy
            .allowed_vm_targets
            .contains(&descriptor.manifest.vm_target)
        {
            return Err(PolicyValidationError::PolicyViolation(
                "descriptor vm_target is disabled by contracts config".to_string(),
            )
            .into());
        }

        if descriptor.manifest.entrypoints.is_empty() {
            return Err(PolicyValidationError::PolicyViolation(
                "descriptor must expose at least one entrypoint".to_string(),
            )
            .into());
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

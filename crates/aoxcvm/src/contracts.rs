//! Contract runtime-binding compatibility surface for integration tests and VM adapters.

pub mod resolver {
    use aoxconfig::contracts::ContractsConfig;
    use aoxcontract::{
        ContractClass, ContractDescriptor, ContractError, ExecutionProfileRef, LaneBinding,
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

        let class_segment = match descriptor.manifest.execution_profile.contract_class {
            ContractClass::Application => "application",
            ContractClass::System => "system",
            ContractClass::Governed => "governed",
            ContractClass::Package => "package",
            ContractClass::PolicyBound => "policy-bound",
        };

        RuntimeBindingDescriptor::from_descriptor(
            descriptor,
            lane,
            ExecutionProfileRef(format!("phase2-{class_segment}")),
        )
    }
}

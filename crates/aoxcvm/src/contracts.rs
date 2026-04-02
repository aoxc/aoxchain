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
        enforce_phase2_runtime_law(descriptor, config)?;

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
        .unwrap();

        ContractDescriptor::new(manifest).unwrap()
    }

    #[test]
    fn resolver_maps_application_profile_ref() {
        let descriptor = sample_descriptor();
        let config = ContractsConfig::default();
        let binding = resolve_runtime_binding(&descriptor, &config).unwrap();
        assert_eq!(binding.execution_profile.0, "phase2-application");
    }

    #[test]
    fn resolver_maps_system_profile_ref() {
        let mut descriptor = sample_descriptor();
        descriptor.manifest.execution_profile.contract_class = ContractClass::System;
        descriptor.manifest.validate().unwrap();
        let config = ContractsConfig::default();
        let binding = resolve_runtime_binding(&descriptor, &config).unwrap();
        assert_eq!(binding.execution_profile.0, "phase2-system");
    }

    #[test]
    fn resolver_maps_policy_bound_profile_ref() {
        let mut descriptor = sample_descriptor();
        descriptor.manifest.execution_profile.contract_class = ContractClass::PolicyBound;
        descriptor
            .manifest
            .execution_profile
            .policy_profile
            .restricted_to_auth_profile = Some("ops-signer-v1".into());
        descriptor.manifest.validate().unwrap();
        let config = ContractsConfig::default();
        let binding = resolve_runtime_binding(&descriptor, &config).unwrap();
        assert_eq!(binding.execution_profile.0, "phase2-policy-bound");
    }

    #[test]
    fn resolver_rejects_profile_vm_mismatch_fail_closed() {
        let mut descriptor = sample_descriptor();
        descriptor.manifest.execution_profile.vm_target = VmTarget::Evm;
        let err = descriptor.manifest.validate().unwrap_err();
        assert!(matches!(
            err,
            aoxcontract::ContractError::Manifest(
                ManifestValidationError::ExecutionProfileVmTargetMismatch
            )
        ));
    }
}

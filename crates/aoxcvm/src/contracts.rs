//! Contract runtime-binding compatibility surface for integration tests and VM adapters.

pub mod resolver {
    use aoxconfig::contracts::ContractsConfig;
    use aoxcontract::{
        ContractClass, ContractDescriptor, ContractError, ExecutionProfileRef, LaneBinding,
        RuntimeBindingDescriptor, VmTarget,
    };

    /// Resolves VM runtime binding from a canonical contract descriptor.
    ///
    /// This resolver is intentionally deterministic and fail-closed:
    /// - the manifest VM target must be allowed by config,
    /// - the descriptor is assumed to carry a validated manifest,
    /// - the returned execution profile reference is class-aware.
    pub fn resolve_runtime_binding(
        descriptor: &ContractDescriptor,
        config: &ContractsConfig,
    ) -> Result<RuntimeBindingDescriptor, ContractError> {
        enforce_allowed_vm_target(descriptor, config)?;

        let lane = match &descriptor.manifest.vm_target {
            VmTarget::Wasm => LaneBinding::Wasm,
            VmTarget::Evm => LaneBinding::Evm,
            VmTarget::SuiLike => LaneBinding::Sui,
            VmTarget::Custom(lane) => LaneBinding::Custom(lane.clone()),
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

    fn enforce_allowed_vm_target(
        descriptor: &ContractDescriptor,
        config: &ContractsConfig,
    ) -> Result<(), ContractError> {
        let allowed = config
            .artifact_policy
            .allowed_vm_targets
            .iter()
            .any(|target| target == &descriptor.manifest.vm_target);

        if !allowed {
            return Err(ContractError::Policy(
                aoxcontract::PolicyValidationError::PolicyViolation(
                    "vm target disabled by contracts config".to_string(),
                ),
            ));
        }

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use aoxcontract::{
            ArtifactDigest, ArtifactDigestAlgorithm, ContractDescriptor, ContractMetadata,
            Entrypoint, ManifestValidationError, VmTarget,
        };
        use aoxcsdk::contracts::builder::ContractManifestBuilder;

        fn sample_descriptor() -> ContractDescriptor {
            let manifest = ContractManifestBuilder::new()
                .with_name("phase2_contract")
                .with_package("aox.phase2")
                .with_version("1.0.0")
                .with_contract_version("1.0.0")
                .with_vm_target(VmTarget::Wasm)
                .with_artifact_digest(ArtifactDigest {
                    algorithm: ArtifactDigestAlgorithm::Sha256,
                    value: "5f4dcc3b5aa765d61d8327deb882cf9922222222222222222222222222222222"
                        .into(),
                })
                .with_artifact_location("ipfs://phase2/contract.wasm")
                .with_metadata(ContractMetadata {
                    display_name: "Phase2 Contract".into(),
                    description: Some("phase2 runtime binding test".into()),
                    author: Some("AOX".into()),
                    organization: Some("AOX".into()),
                    source_reference: None,
                    tags: vec!["phase2".into()],
                    created_at: None,
                    updated_at: None,
                    audit_reference: Some("approved".into()),
                    notes: None,
                })
                .add_entrypoint(Entrypoint::new("execute", VmTarget::Wasm, None, vec![]).unwrap())
                .build()
                .unwrap();

            ContractDescriptor::new(manifest).unwrap()
        }

        #[test]
        fn resolver_maps_application_profile_ref() {
            let descriptor = sample_descriptor();
            let config = ContractsConfig::default();

            let binding = resolve_runtime_binding(&descriptor, &config).unwrap();
            assert_eq!(binding.execution_profile.0, "phase2-application");
            assert_eq!(binding.resolved_profile, descriptor.manifest.execution_profile);
        }

        #[test]
        fn resolver_maps_system_profile_ref() {
            let mut descriptor = sample_descriptor();
            descriptor.manifest.execution_profile.contract_class = ContractClass::System;
            descriptor.manifest.validate().unwrap();

            let config = ContractsConfig::default();
            let binding = resolve_runtime_binding(&descriptor, &config).unwrap();

            assert_eq!(binding.execution_profile.0, "phase2-system");
            assert_eq!(
                binding.resolved_profile.contract_class,
                ContractClass::System
            );
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
            assert_eq!(
                binding
                    .resolved_profile
                    .policy_profile
                    .restricted_to_auth_profile
                    .as_deref(),
                Some("ops-signer-v1")
            );
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

        #[test]
        fn resolver_rejects_disabled_vm_target_from_config() {
            let descriptor = sample_descriptor();
            let mut config = ContractsConfig::default();
            config.artifact_policy.allowed_vm_targets = vec![VmTarget::Evm];

            let err = resolve_runtime_binding(&descriptor, &config).unwrap_err();
            assert!(matches!(err, ContractError::Policy(_)));
        }
    }
}

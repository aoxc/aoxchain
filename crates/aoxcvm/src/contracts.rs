//! Contract runtime-binding compatibility surface for integration tests and VM adapters.

pub mod resolver {
    use aoxconfig::contracts::ContractsConfig;
    use aoxcontract::{
        CapabilityProfile, ContractClass, ContractDescriptor, ContractError, ExecutionProfileRef,
        LaneBinding, PolicyProfile, RuntimeBindingDescriptor, VmTarget,
    };

    fn is_canonical_auth_profile_id(value: &str) -> bool {
        !value.trim().is_empty()
            && value == value.trim()
            && value.chars().all(|c| {
                c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '-' | '.')
            })
    }

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
        enforce_policy_profile(descriptor, config)?;
        enforce_capability_profile(descriptor)?;

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

    fn enforce_policy_profile(
        descriptor: &ContractDescriptor,
        config: &ContractsConfig,
    ) -> Result<(), ContractError> {
        let class = &descriptor.manifest.execution_profile.contract_class;
        let policy = &descriptor.manifest.execution_profile.policy_profile;
        let capabilities = &descriptor.manifest.execution_profile.capability_profile;

        if config.artifact_policy.review_required && !policy.review_required {
            return Err(ContractError::Policy(
                aoxcontract::PolicyValidationError::PolicyViolation(
                    "execution profile cannot disable required review policy".to_string(),
                ),
            ));
        }

        if policy.governance_activation_required
            && !matches!(
                class,
                ContractClass::Governed | ContractClass::PolicyBound | ContractClass::System
            )
        {
            return Err(ContractError::Policy(
                aoxcontract::PolicyValidationError::PolicyViolation(
                    "governance activation is only valid for governed, policy-bound, or system classes"
                        .to_string(),
                ),
            ));
        }

        if capabilities.governance_hooks && !policy.governance_activation_required {
            return Err(ContractError::Policy(
                aoxcontract::PolicyValidationError::PolicyViolation(
                    "governance hooks require governance activation policy".to_string(),
                ),
            ));
        }

        match class {
            ContractClass::PolicyBound => {
                enforce_policy_bound_profile(policy, capabilities)?;
            }
            _ => {
                if policy.restricted_to_auth_profile.is_some() {
                    return Err(ContractError::Policy(
                        aoxcontract::PolicyValidationError::PolicyViolation(
                            "restricted auth profile is only allowed for policy-bound class"
                                .to_string(),
                        ),
                    ));
                }
            }
        }

        Ok(())
    }

    fn enforce_policy_bound_profile(
        policy: &PolicyProfile,
        capabilities: &CapabilityProfile,
    ) -> Result<(), ContractError> {
        let missing_or_empty_profile = match policy.restricted_to_auth_profile.as_deref() {
            Some(id) => !is_canonical_auth_profile_id(id),
            None => true,
        };
        if missing_or_empty_profile {
            return Err(ContractError::Policy(
                aoxcontract::PolicyValidationError::PolicyViolation(
                    "policy-bound contracts require a non-empty restricted auth profile"
                        .to_string(),
                ),
            ));
        }

        if !capabilities.restricted_syscalls {
            return Err(ContractError::Policy(
                aoxcontract::PolicyValidationError::PolicyViolation(
                    "policy-bound contracts must enable restricted syscalls".to_string(),
                ),
            ));
        }

        Ok(())
    }

    fn enforce_capability_profile(descriptor: &ContractDescriptor) -> Result<(), ContractError> {
        let class = &descriptor.manifest.execution_profile.contract_class;
        let capability = &descriptor.manifest.execution_profile.capability_profile;

        let mut forbidden = Vec::new();
        let mut push_if = |enabled: bool, feature: &'static str| {
            if enabled {
                forbidden.push(feature);
            }
        };

        match class {
            ContractClass::Application => {
                push_if(capability.registry_access, "registry_access");
                push_if(capability.governance_hooks, "governance_hooks");
                push_if(capability.metadata_mutation, "metadata_mutation");
                push_if(capability.upgrade_authority, "upgrade_authority");
            }
            ContractClass::System => {}
            ContractClass::Governed => {
                push_if(capability.upgrade_authority, "upgrade_authority");
            }
            ContractClass::Package => {
                push_if(capability.storage_write, "storage_write");
                push_if(capability.registry_access, "registry_access");
                push_if(capability.governance_hooks, "governance_hooks");
                push_if(capability.metadata_mutation, "metadata_mutation");
                push_if(capability.upgrade_authority, "upgrade_authority");
            }
            ContractClass::PolicyBound => {
                push_if(capability.governance_hooks, "governance_hooks");
                push_if(capability.metadata_mutation, "metadata_mutation");
                push_if(capability.upgrade_authority, "upgrade_authority");
            }
        }

        if forbidden.is_empty() {
            return Ok(());
        }

        Err(ContractError::Policy(
            aoxcontract::PolicyValidationError::PolicyViolation(format!(
                "{class:?} class forbids capabilities: {}",
                forbidden.join(", ")
            )),
        ))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use aoxcontract::{
            ArtifactDigest, ArtifactDigestAlgorithm, ContractDescriptor, ContractMetadata,
            Entrypoint, ManifestValidationError, Validate, VmTarget,
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
            assert_eq!(
                binding.resolved_profile,
                descriptor.manifest.execution_profile
            );
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

        #[test]
        fn resolver_rejects_policybound_without_auth_profile() {
            let mut descriptor = sample_descriptor();
            descriptor.manifest.execution_profile.contract_class = ContractClass::PolicyBound;
            descriptor
                .manifest
                .execution_profile
                .capability_profile
                .restricted_syscalls = true;
            descriptor
                .manifest
                .execution_profile
                .policy_profile
                .restricted_to_auth_profile = None;

            let err =
                resolve_runtime_binding(&descriptor, &ContractsConfig::default()).unwrap_err();
            assert!(err.to_string().contains("restricted auth profile"));
        }

        #[test]
        fn resolver_rejects_application_registry_access() {
            let mut descriptor = sample_descriptor();
            descriptor.manifest.execution_profile.contract_class = ContractClass::Application;
            descriptor
                .manifest
                .execution_profile
                .capability_profile
                .registry_access = true;

            let err =
                resolve_runtime_binding(&descriptor, &ContractsConfig::default()).unwrap_err();
            assert!(err.to_string().contains("forbids capabilities"));
            assert!(err.to_string().contains("registry_access"));
        }

        #[test]
        fn resolver_rejects_review_downgrade_when_config_requires_review() {
            let mut descriptor = sample_descriptor();
            descriptor
                .manifest
                .execution_profile
                .policy_profile
                .review_required = false;

            let err =
                resolve_runtime_binding(&descriptor, &ContractsConfig::default()).unwrap_err();
            assert!(err.to_string().contains("cannot disable required review"));
        }

        #[test]
        fn resolver_accepts_governed_contract_with_governance_policy() {
            let mut descriptor = sample_descriptor();
            descriptor.manifest.execution_profile.contract_class = ContractClass::Governed;
            descriptor
                .manifest
                .execution_profile
                .capability_profile
                .governance_hooks = true;
            descriptor
                .manifest
                .execution_profile
                .policy_profile
                .governance_activation_required = true;

            let binding =
                resolve_runtime_binding(&descriptor, &ContractsConfig::default()).unwrap();
            assert_eq!(binding.execution_profile.0, "phase2-governed");
        }
    }
}

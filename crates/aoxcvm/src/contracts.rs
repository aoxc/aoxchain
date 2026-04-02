//! Contract runtime-binding compatibility surface for integration tests and VM adapters.

pub mod resolver {
    use aoxconfig::contracts::ContractsConfig;
    use aoxcontract::{
        ContractCapability, ContractClass, ContractDescriptor, ContractError, ExecutionProfileRef,
        LaneBinding, PolicyValidationError, RuntimeBindingDescriptor, VmTarget,
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
    }

    fn enforce_phase2_runtime_law(
        descriptor: &ContractDescriptor,
        config: &ContractsConfig,
    ) -> Result<(), ContractError> {
        enforce_policy_profile(descriptor, config)?;
        enforce_capability_profile(descriptor)?;
        enforce_contract_class_matrix(descriptor)?;
        Ok(())
    }

    fn enforce_policy_profile(
        descriptor: &ContractDescriptor,
        config: &ContractsConfig,
    ) -> Result<(), ContractError> {
        let policy_profile = &descriptor.manifest.execution_profile.policy_profile;
        let manifest_policy = &descriptor.manifest.policy;

        if policy_profile.review_required && !manifest_policy.review_required {
            return Err(policy_violation(
                "execution profile requires review but manifest policy does not",
            ));
        }
        if config.artifact_policy.review_required && !policy_profile.review_required {
            return Err(policy_violation(
                "contracts config requires review but execution profile disables it",
            ));
        }
        if policy_profile.governance_activation_required
            && config.registry.activation_policy.as_str() != "governance"
        {
            return Err(policy_violation(
                "governance activation is required but contracts config is not governance mode",
            ));
        }
        if let Some(auth_profile) = &policy_profile.restricted_to_auth_profile
            && auth_profile.trim().is_empty()
        {
            return Err(policy_violation(
                "restricted_to_auth_profile must not be blank when declared",
            ));
        }
        Ok(())
    }

    fn enforce_capability_profile(descriptor: &ContractDescriptor) -> Result<(), ContractError> {
        let caps = &descriptor.manifest.execution_profile.capability_profile;
        let allowed = &descriptor.manifest.policy.allowed_capabilities;

        if caps.storage_read && !allowed.contains(&ContractCapability::StorageRead) {
            return Err(policy_violation(
                "execution profile requests storage_read but policy does not allow it",
            ));
        }
        if caps.storage_write && !allowed.contains(&ContractCapability::StorageWrite) {
            return Err(policy_violation(
                "execution profile requests storage_write but policy does not allow it",
            ));
        }
        if caps.storage_write && !caps.storage_read {
            return Err(policy_violation(
                "execution profile cannot enable storage_write without storage_read",
            ));
        }
        if caps.governance_hooks && !allowed.contains(&ContractCapability::GovernanceBound) {
            return Err(policy_violation(
                "execution profile requests governance_hooks without governance capability",
            ));
        }
        if caps.restricted_syscalls && !allowed.contains(&ContractCapability::PrivilegedHook) {
            return Err(policy_violation(
                "execution profile requests restricted_syscalls without privileged capability",
            ));
        }
        Ok(())
    }

    fn enforce_contract_class_matrix(descriptor: &ContractDescriptor) -> Result<(), ContractError> {
        let class = &descriptor.manifest.execution_profile.contract_class;
        let caps = &descriptor.manifest.execution_profile.capability_profile;
        let policy_profile = &descriptor.manifest.execution_profile.policy_profile;

        match class {
            ContractClass::Application => {
                if caps.governance_hooks
                    || caps.restricted_syscalls
                    || caps.upgrade_authority
                    || caps.metadata_mutation
                {
                    return Err(policy_violation(
                        "application contracts cannot request governed or restricted capabilities",
                    ));
                }
            }
            ContractClass::System => {
                if !policy_profile.review_required {
                    return Err(policy_violation(
                        "system contracts must remain review_required",
                    ));
                }
            }
            ContractClass::Governed => {
                if !policy_profile.governance_activation_required {
                    return Err(policy_violation(
                        "governed contracts must require governance activation",
                    ));
                }
            }
            ContractClass::Package => {
                if descriptor.manifest.entrypoints.len() > 1 {
                    return Err(policy_violation(
                        "package class contracts may expose at most one entrypoint",
                    ));
                }
            }
            ContractClass::PolicyBound => {
                if policy_profile.restricted_to_auth_profile.is_none() {
                    return Err(policy_violation(
                        "policy_bound contracts must declare restricted_to_auth_profile",
                    ));
                }
            }
        }
        Ok(())
    }

    fn policy_violation(message: &'static str) -> ContractError {
        ContractError::Policy(PolicyValidationError::PolicyViolation(message.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::resolver::resolve_runtime_binding;
    use aoxconfig::contracts::ContractsConfig;
    use aoxcontract::{
        ArtifactDigest, ArtifactDigestAlgorithm, ArtifactFormat, ArtifactLocationKind,
        Compatibility, ContractArtifactRef, ContractCapability, ContractClass, ContractDescriptor,
        ContractManifest, ContractMetadata, ContractPolicy, ContractVersion, Entrypoint, Integrity,
        ManifestValidationError, NetworkClass, RuntimeFamily, SourceTrustLevel, Validate, VmTarget,
    };

    fn sample_descriptor() -> ContractDescriptor {
        let digest = ArtifactDigest {
            algorithm: ArtifactDigestAlgorithm::Sha256,
            value: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        };
        let artifact = ContractArtifactRef::new(
            digest.clone(),
            64,
            ArtifactFormat::WasmModule,
            ArtifactLocationKind::Uri,
            "ipfs://resolver-test",
            None,
            Some("application/wasm".into()),
            VmTarget::Wasm,
        )
        .unwrap();
        let manifest = ContractManifest::new(
            "resolver_test",
            "aox.test",
            "1.0.0",
            ContractVersion("1.0.0".into()),
            VmTarget::Wasm,
            artifact,
            vec![Entrypoint::new("execute", VmTarget::Wasm, None, vec![]).unwrap()],
            digest.clone(),
            ContractMetadata {
                display_name: "Resolver Test".into(),
                description: None,
                author: None,
                organization: None,
                source_reference: None,
                tags: vec![],
                created_at: None,
                updated_at: None,
                audit_reference: None,
                notes: None,
            },
            ContractPolicy::new(
                vec![VmTarget::Wasm],
                vec![ArtifactFormat::WasmModule],
                1024,
                vec![ContractCapability::StorageRead],
                vec![],
                true,
                true,
                SourceTrustLevel::ReviewRequired,
            )
            .unwrap(),
            Compatibility::new(
                1,
                vec![1],
                vec![RuntimeFamily::Wasm],
                vec![NetworkClass::Testnet],
                vec![],
                false,
            )
            .unwrap(),
            Integrity {
                digest,
                artifact_size: 64,
                artifact_format: ArtifactFormat::WasmModule,
                media_type: Some("application/wasm".into()),
                signature_required: true,
                source_trust_level: SourceTrustLevel::ReviewRequired,
            },
            1,
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

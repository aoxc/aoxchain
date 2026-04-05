#[cfg(test)]
mod tests {
    use super::{BuilderError, ContractManifestBuilder};
    use aoxcontract::{
        ArtifactDigest, ArtifactDigestAlgorithm, CapabilityProfile, ContractCapability,
        ContractClass, Entrypoint, ExecutionProfile, NetworkClass, PolicyProfile, RuntimeFamily,
        Validate, VmTarget,
    };

    fn digest(seed: &str) -> ArtifactDigest {
        ArtifactDigest {
            algorithm: ArtifactDigestAlgorithm::Sha256,
            value: seed.to_string(),
        }
    }

    #[test]
    fn build_wasm_manifest_without_explicit_metadata() {
        let manifest = ContractManifestBuilder::wasm()
            .with_name("hello")
            .with_package("aox.hello")
            .with_artifact_digest(digest(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ))
            .with_artifact_location("ipfs://hello/contract.wasm")
            .add_entrypoint(
                Entrypoint::new("execute", VmTarget::Wasm, None, vec![])
                    .expect("entrypoint should build"),
            )
            .allow_capability(ContractCapability::StorageRead)
            .build()
            .expect("manifest should be built");

        assert_eq!(manifest.name, "hello");
        assert_eq!(manifest.metadata.display_name, "hello");
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(
            manifest.compatibility.supported_runtime_families,
            vec![RuntimeFamily::Wasm]
        );
        assert_eq!(
            manifest.compatibility.supported_network_classes,
            vec![
                NetworkClass::Mainnet,
                NetworkClass::Testnet,
                NetworkClass::Devnet
            ]
        );
    }

    #[test]
    fn build_descriptor_works() {
        let descriptor = ContractManifestBuilder::evm()
            .with_name("evm_contract")
            .with_package("aox.evm")
            .with_artifact_digest(digest(
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ))
            .with_artifact_location("ipfs://evm/contract.bin")
            .add_entrypoint(
                Entrypoint::new("execute", VmTarget::Evm, None, vec![])
                    .expect("entrypoint should build"),
            )
            .build_descriptor()
            .expect("descriptor should be built");

        assert_eq!(descriptor.manifest.vm_target, VmTarget::Evm);
    }

    #[test]
    fn missing_required_field_returns_package_error_when_package_is_absent() {
        let err = ContractManifestBuilder::new()
            .with_name("incomplete")
            .build()
            .expect_err("builder should fail");

        assert!(matches!(err, BuilderError::MissingField("package")));
    }

    #[test]
    fn missing_required_field_returns_vm_target_error_when_prior_fields_exist() {
        let err = ContractManifestBuilder::new()
            .with_name("incomplete")
            .with_package("aox.incomplete")
            .with_version("1.0.0")
            .with_contract_version("1.0.0")
            .build()
            .expect_err("builder should fail");

        assert!(matches!(err, BuilderError::MissingField("vm_target")));
    }

    #[test]
    fn missing_required_field_returns_artifact_location_error_after_vm_and_digest_are_set() {
        let err = ContractManifestBuilder::new()
            .with_name("incomplete")
            .with_package("aox.incomplete")
            .with_version("1.0.0")
            .with_contract_version("1.0.0")
            .with_vm_target(VmTarget::Wasm)
            .with_artifact_digest(digest(
                "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
            ))
            .build()
            .expect_err("builder should fail");

        assert!(matches!(
            err,
            BuilderError::MissingField("artifact_location")
        ));
    }

    #[test]
    fn explicit_compatibility_overrides_are_preserved() {
        let manifest = ContractManifestBuilder::new()
            .with_name("custom-compat")
            .with_package("aox.compat")
            .with_version("1.0.0")
            .with_contract_version("1.0.0")
            .with_vm_target(VmTarget::Custom("kernel-x".to_string()))
            .with_artifact_digest(digest(
                "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            ))
            .with_artifact_location("ipfs://custom/contract.bin")
            .with_supported_runtime_families(vec![RuntimeFamily::AoxVm, RuntimeFamily::Wasm])
            .with_supported_network_classes(vec![NetworkClass::Airgapped])
            .add_entrypoint(
                Entrypoint::new(
                    "execute",
                    VmTarget::Custom("kernel-x".to_string()),
                    None,
                    vec![],
                )
                .expect("entrypoint should build"),
            )
            .build()
            .expect("manifest should build");

        assert_eq!(
            manifest.compatibility.supported_runtime_families,
            vec![RuntimeFamily::AoxVm, RuntimeFamily::Wasm]
        );
        assert_eq!(
            manifest.compatibility.supported_network_classes,
            vec![NetworkClass::Airgapped]
        );
    }

    #[test]
    fn empty_compatibility_overrides_fall_back_to_defaults() {
        let manifest = ContractManifestBuilder::new()
            .with_name("default-compat")
            .with_package("aox.default")
            .with_version("1.0.0")
            .with_contract_version("1.0.0")
            .with_vm_target(VmTarget::Wasm)
            .with_artifact_digest(digest(
                "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            ))
            .with_artifact_location("ipfs://default/contract.wasm")
            .with_supported_runtime_families(vec![])
            .with_supported_network_classes(vec![])
            .add_entrypoint(
                Entrypoint::new("execute", VmTarget::Wasm, None, vec![])
                    .expect("entrypoint should build"),
            )
            .build()
            .expect("manifest should build");

        assert_eq!(
            manifest.compatibility.supported_runtime_families,
            vec![RuntimeFamily::Wasm]
        );
        assert_eq!(
            manifest.compatibility.supported_network_classes,
            vec![
                NetworkClass::Mainnet,
                NetworkClass::Testnet,
                NetworkClass::Devnet
            ]
        );
    }

    #[test]
    fn builder_supports_contract_class_and_profile_overrides() {
        let manifest = ContractManifestBuilder::wasm()
            .with_name("policy-contract")
            .with_package("aox.policy")
            .with_artifact_digest(digest(
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            ))
            .with_artifact_location("ipfs://policy/contract.wasm")
            .add_entrypoint(
                Entrypoint::new("execute", VmTarget::Wasm, None, vec![])
                    .expect("entrypoint should build"),
            )
            .allow_capability(ContractCapability::StorageRead)
            .with_contract_class(ContractClass::PolicyBound)
            .with_capability_profile(CapabilityProfile {
                storage_read: true,
                ..CapabilityProfile::default()
            })
            .with_policy_profile(PolicyProfile {
                review_required: true,
                governance_activation_required: false,
                restricted_to_auth_profile: Some("ops-v1".into()),
            })
            .build()
            .expect("manifest should build");

        assert_eq!(
            manifest.execution_profile.contract_class,
            ContractClass::PolicyBound
        );
        assert_eq!(
            manifest
                .execution_profile
                .policy_profile
                .restricted_to_auth_profile
                .as_deref(),
            Some("ops-v1")
        );
    }

    #[test]
    fn builder_supports_execution_profile_override() {
        let manifest = ContractManifestBuilder::wasm()
            .with_name("profile-override")
            .with_package("aox.policy.override")
            .with_artifact_digest(digest(
                "1212121212121212121212121212121212121212121212121212121212121212",
            ))
            .with_artifact_location("ipfs://policy/override.wasm")
            .add_entrypoint(
                Entrypoint::new("execute", VmTarget::Wasm, None, vec![])
                    .expect("entrypoint should build"),
            )
            .with_execution_profile(ExecutionProfile {
                vm_target: VmTarget::Wasm,
                contract_class: ContractClass::Governed,
                capability_profile: CapabilityProfile {
                    storage_read: true,
                    governance_hooks: true,
                    ..CapabilityProfile::default()
                },
                policy_profile: PolicyProfile {
                    review_required: true,
                    governance_activation_required: true,
                    restricted_to_auth_profile: None,
                },
            })
            .build()
            .expect("manifest should build");

        manifest.validate().expect("manifest should validate");
        assert_eq!(
            manifest.execution_profile.contract_class,
            ContractClass::Governed
        );
        assert!(
            manifest
                .execution_profile
                .capability_profile
                .governance_hooks
        );
        assert!(
            manifest
                .execution_profile
                .policy_profile
                .governance_activation_required
        );
    }

    #[test]
    fn builder_override_precedence_profile_then_class_and_policy() {
        let manifest = ContractManifestBuilder::wasm()
            .with_name("precedence-1")
            .with_package("aox.precedence.one")
            .with_artifact_digest(digest(
                "2222222222222222222222222222222222222222222222222222222222222222",
            ))
            .with_artifact_location("ipfs://policy/precedence1.wasm")
            .add_entrypoint(
                Entrypoint::new("execute", VmTarget::Wasm, None, vec![])
                    .expect("entrypoint should build"),
            )
            .with_execution_profile(ExecutionProfile {
                vm_target: VmTarget::Wasm,
                contract_class: ContractClass::Application,
                capability_profile: CapabilityProfile {
                    storage_read: true,
                    ..CapabilityProfile::default()
                },
                policy_profile: PolicyProfile {
                    review_required: true,
                    governance_activation_required: false,
                    restricted_to_auth_profile: None,
                },
            })
            .with_contract_class(ContractClass::PolicyBound)
            .with_capability_profile(CapabilityProfile {
                storage_read: true,
                restricted_syscalls: true,
                ..CapabilityProfile::default()
            })
            .with_policy_profile(PolicyProfile {
                review_required: true,
                governance_activation_required: false,
                restricted_to_auth_profile: Some("ops-v1".into()),
            })
            .build()
            .expect("manifest should build");

        assert_eq!(
            manifest.execution_profile.contract_class,
            ContractClass::PolicyBound
        );
        assert!(
            manifest
                .execution_profile
                .capability_profile
                .restricted_syscalls
        );
        assert_eq!(
            manifest
                .execution_profile
                .policy_profile
                .restricted_to_auth_profile
                .as_deref(),
            Some("ops-v1")
        );
    }

    #[test]
    fn builder_override_precedence_class_then_execution_profile() {
        let manifest = ContractManifestBuilder::wasm()
            .with_name("precedence-2")
            .with_package("aox.precedence.two")
            .with_artifact_digest(digest(
                "3333333333333333333333333333333333333333333333333333333333333333",
            ))
            .with_artifact_location("ipfs://policy/precedence2.wasm")
            .add_entrypoint(
                Entrypoint::new("execute", VmTarget::Wasm, None, vec![])
                    .expect("entrypoint should build"),
            )
            .with_contract_class(ContractClass::PolicyBound)
            .with_execution_profile(ExecutionProfile {
                vm_target: VmTarget::Wasm,
                contract_class: ContractClass::Governed,
                capability_profile: CapabilityProfile {
                    storage_read: true,
                    governance_hooks: true,
                    ..CapabilityProfile::default()
                },
                policy_profile: PolicyProfile {
                    review_required: true,
                    governance_activation_required: true,
                    restricted_to_auth_profile: None,
                },
            })
            .build()
            .expect("manifest should build");

        assert_eq!(
            manifest.execution_profile.contract_class,
            ContractClass::Governed
        );
        assert!(
            manifest
                .execution_profile
                .capability_profile
                .governance_hooks
        );
    }
}

use aoxconfig::contracts::ContractsConfig;
use aoxcontract::{
    ArtifactDigest, ArtifactDigestAlgorithm, CapabilityProfile, ContractClass, ContractDescriptor,
    Entrypoint, PolicyProfile, Validate, VmTarget,
};
use aoxcsdk::contracts::builder::{BuilderError, ContractManifestBuilder};
use aoxcvm::tx::{envelope::TxEnvelope, fee::FeeBudget, payload::TxPayload};
use aoxcvm::{
    contracts::resolver::resolve_runtime_binding,
    tx::kind::TxKind,
    vm::admission::{AdmissionError, validate_phase2_admission},
};

fn digest() -> ArtifactDigest {
    ArtifactDigest {
        algorithm: ArtifactDigestAlgorithm::Sha256,
        value: "1111111111111111111111111111111111111111111111111111111111111111".into(),
    }
}

fn base_builder() -> ContractManifestBuilder {
    ContractManifestBuilder::wasm()
        .with_name("phase2-full")
        .with_package("aox.phase2.full")
        .with_version("1.0.0")
        .with_contract_version("1.0.0")
        .with_artifact_digest(digest())
        .with_artifact_location("ipfs://phase2/full.wasm")
        .add_entrypoint(
            Entrypoint::new("execute", VmTarget::Wasm, None, vec![])
                .expect("entrypoint should build"),
        )
}

#[test]
fn phase2_integration_builder_manifest_descriptor_resolver_ok_flow() {
    let manifest = base_builder()
        .with_contract_class(ContractClass::PolicyBound)
        .with_capability_profile(CapabilityProfile {
            storage_read: true,
            restricted_syscalls: true,
            ..CapabilityProfile::default()
        })
        .with_policy_profile(PolicyProfile {
            review_required: true,
            governance_activation_required: true,
            restricted_to_auth_profile: Some("ops-v1".into()),
        })
        .build()
        .expect("manifest should build");

    manifest.validate().expect("manifest should validate");
    let descriptor = ContractDescriptor::new(manifest).expect("descriptor should build");

    let binding = resolve_runtime_binding(&descriptor, &ContractsConfig::default())
        .expect("binding should resolve");
    assert_eq!(binding.execution_profile.0, "phase2-policy-bound");

    let tx = TxEnvelope::new(
        2626,
        1,
        TxKind::Governance,
        FeeBudget::new(300_000, 1),
        TxPayload::new(vec![1]),
    );

    assert_eq!(
        validate_phase2_admission(&binding, &tx, Some("ops-v1")),
        Ok(())
    );
}

#[test]
fn phase2_integration_manifest_fail_closed_for_application_capability_escalation() {
    let err = base_builder()
        .with_contract_class(ContractClass::Application)
        .with_capability_profile(CapabilityProfile {
            storage_read: true,
            registry_access: true,
            ..CapabilityProfile::default()
        })
        .build()
        .expect_err("manifest build must fail for forbidden application capability");

    match err {
        BuilderError::Contract(contract_err) => {
            assert!(
                contract_err.to_string().contains("registry_access"),
                "unexpected contract error: {contract_err}"
            );
        }
        other => panic!("unexpected builder error variant: {other}"),
    }
}

#[test]
fn phase2_integration_admission_rejects_tx_kind_mismatch_for_class() {
    let manifest = base_builder()
        .with_contract_class(ContractClass::System)
        .with_capability_profile(CapabilityProfile {
            storage_read: true,
            registry_access: true,
            ..CapabilityProfile::default()
        })
        .build()
        .expect("manifest should build");

    manifest.validate().expect("manifest should validate");
    let descriptor = ContractDescriptor::new(manifest).expect("descriptor should build");

    let binding = resolve_runtime_binding(&descriptor, &ContractsConfig::default())
        .expect("binding should resolve");

    let tx = TxEnvelope::new(
        2626,
        1,
        TxKind::UserCall,
        FeeBudget::new(300_000, 1),
        TxPayload::new(vec![1]),
    );

    let err = validate_phase2_admission(&binding, &tx, None)
        .expect_err("admission must reject tx kind forbidden for class");

    assert_eq!(err, AdmissionError::TxKindForbiddenForClass);
}

#[test]
fn phase2_integration_rejects_policy_bound_with_malformed_active_auth_profile() {
    let manifest = base_builder()
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

    manifest.validate().expect("manifest should validate");
    let descriptor = ContractDescriptor::new(manifest).expect("descriptor should build");

    let binding = resolve_runtime_binding(&descriptor, &ContractsConfig::default())
        .expect("binding should resolve");

    let tx = TxEnvelope::new(
        2626,
        1,
        TxKind::UserCall,
        FeeBudget::new(300_000, 1),
        TxPayload::new(vec![1]),
    );

    let err = validate_phase2_admission(&binding, &tx, Some(" OPS-V1 "))
        .expect_err("admission must reject malformed or non-canonical auth profile");

    assert_eq!(err, AdmissionError::RestrictedAuthProfileMismatch);
}

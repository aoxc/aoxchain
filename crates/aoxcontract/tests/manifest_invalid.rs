// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

mod common;

use aoxcontract::{
    ArtifactValidationError, CapabilityProfile, CompatibilityError, ContractClass, ContractError,
    ContractManifest, ContractVersion, Entrypoint, ManifestValidationError, PolicyProfile,
    Validate, VmTarget,
};

#[test]
fn empty_name_is_rejected() {
    let manifest = common::sample_manifest();
    let err = ContractManifest::new(
        "   ",
        manifest.package,
        manifest.version,
        manifest.contract_version,
        manifest.vm_target,
        manifest.artifact,
        manifest.entrypoints,
        manifest.digest,
        manifest.metadata,
        manifest.policy,
        manifest.compatibility,
        manifest.integrity,
        manifest.schema_version,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ContractError::Manifest(ManifestValidationError::EmptyContractName)
    ));
}

#[test]
fn duplicate_entrypoint_is_rejected() {
    let mut manifest = common::sample_manifest();
    manifest
        .entrypoints
        .push(Entrypoint::new("execute", VmTarget::Wasm, None, vec![]).unwrap());
    let err = manifest.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Manifest(ManifestValidationError::DuplicateEntrypoint(_))
    ));
}

#[test]
fn oversized_artifact_is_rejected() {
    let mut manifest = common::sample_manifest();
    manifest.artifact.artifact_size = manifest.policy.max_artifact_size + 1;
    let err = manifest.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Artifact(ArtifactValidationError::ArtifactTooLarge)
    ));
}

#[test]
fn invalid_version_is_rejected() {
    let manifest = common::sample_manifest();
    let err = ContractManifest::new(
        manifest.name,
        manifest.package,
        "not-semver",
        ContractVersion("1.0.0".into()),
        manifest.vm_target,
        manifest.artifact,
        manifest.entrypoints,
        manifest.digest,
        manifest.metadata,
        manifest.policy,
        manifest.compatibility,
        manifest.integrity,
        1,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        ContractError::Manifest(ManifestValidationError::InvalidVersionFormat)
    ));
}

#[test]
fn compatibility_mismatch_is_rejected() {
    let mut manifest = common::sample_manifest();
    manifest.schema_version = 2;
    let err = manifest.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Compatibility(CompatibilityError::CompatibilityMismatch)
    ));
}

#[test]
fn empty_entrypoints_are_rejected() {
    let mut manifest = common::sample_manifest();
    manifest.entrypoints.clear();
    let err = manifest.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Manifest(ManifestValidationError::EmptyEntrypoints)
    ));
}

#[test]
fn integrity_metadata_mismatch_is_rejected() {
    let mut manifest = common::sample_manifest();
    manifest.integrity.artifact_size = manifest.artifact.artifact_size + 1;
    let err = manifest.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Manifest(ManifestValidationError::IntegrityMetadataMismatch)
    ));
}

#[test]
fn execution_profile_vm_target_mismatch_is_rejected() {
    let mut manifest = common::sample_manifest();
    manifest.execution_profile.vm_target = VmTarget::Evm;
    let err = manifest.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Manifest(ManifestValidationError::ExecutionProfileVmTargetMismatch)
    ));
}

#[test]
fn restricted_profile_is_rejected_for_non_policy_bound_class() {
    let mut manifest = common::sample_manifest();
    manifest.execution_profile.contract_class = ContractClass::Application;
    manifest.execution_profile.policy_profile = PolicyProfile {
        review_required: true,
        governance_activation_required: false,
        restricted_to_auth_profile: Some("ops-v1".into()),
    };

    let err = manifest.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Manifest(ManifestValidationError::RestrictedAuthProfileOnlyForPolicyBound)
    ));
}

#[test]
fn policy_bound_requires_canonical_auth_profile() {
    let mut manifest = common::sample_manifest();
    manifest.execution_profile.contract_class = ContractClass::PolicyBound;
    manifest.execution_profile.capability_profile = CapabilityProfile {
        storage_read: true,
        restricted_syscalls: true,
        ..CapabilityProfile::default()
    };
    manifest.execution_profile.policy_profile = PolicyProfile {
        review_required: true,
        governance_activation_required: false,
        restricted_to_auth_profile: Some(" OPS-V1 ".into()),
    };

    let err = manifest.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Manifest(ManifestValidationError::InvalidPolicyBoundProfile)
    ));
}

#[test]
fn governance_activation_is_rejected_for_application_class() {
    let mut manifest = common::sample_manifest();
    manifest.execution_profile.contract_class = ContractClass::Application;
    manifest.execution_profile.policy_profile = PolicyProfile {
        review_required: true,
        governance_activation_required: true,
        restricted_to_auth_profile: None,
    };

    let err = manifest.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Manifest(ManifestValidationError::InvalidGovernanceActivationClass)
    ));
}

#[test]
fn governance_hooks_without_activation_is_rejected() {
    let mut manifest = common::sample_manifest();
    manifest.execution_profile.contract_class = ContractClass::Governed;
    manifest.execution_profile.capability_profile = CapabilityProfile {
        storage_read: true,
        governance_hooks: true,
        ..CapabilityProfile::default()
    };
    manifest.execution_profile.policy_profile = PolicyProfile {
        review_required: true,
        governance_activation_required: false,
        restricted_to_auth_profile: None,
    };

    let err = manifest.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Manifest(ManifestValidationError::GovernanceHooksRequireActivation)
    ));
}

#[test]
fn package_class_rejects_storage_write_capability() {
    let mut manifest = common::sample_manifest();
    manifest.execution_profile.contract_class = ContractClass::Package;
    manifest.execution_profile.capability_profile = CapabilityProfile {
        storage_read: true,
        storage_write: true,
        ..CapabilityProfile::default()
    };

    let err = manifest.validate().unwrap_err();
    assert!(matches!(
        err,
        ContractError::Manifest(ManifestValidationError::CapabilityForbiddenForClass(
            "storage_write"
        ))
    ));
}

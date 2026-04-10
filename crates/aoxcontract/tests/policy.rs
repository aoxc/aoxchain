// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

mod common;

use aoxcontract::{
    ArtifactFormat, ContractCapability, ContractError, ContractPolicy, PolicyValidationError,
    PqSignatureScheme, QuantumMigrationMode, QuantumSecurityProfile, SourceTrustLevel, Validate,
    VmTarget,
};

#[test]
fn duplicate_capabilities_are_rejected() {
    let err = ContractPolicy::new(
        vec![VmTarget::Wasm],
        vec![ArtifactFormat::WasmModule],
        1024,
        vec![ContractCapability::StorageRead],
        vec![ContractCapability::StorageRead],
        false,
        false,
        SourceTrustLevel::Trusted,
    )
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Policy(PolicyValidationError::DuplicateCapability(_))
    ));
}

#[test]
fn forbidden_overlap_is_rejected() {
    let err = ContractPolicy::new(
        vec![VmTarget::Wasm],
        vec![ArtifactFormat::WasmModule],
        1024,
        vec![ContractCapability::TreasurySensitive],
        vec![ContractCapability::TreasurySensitive],
        false,
        false,
        SourceTrustLevel::Trusted,
    )
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Policy(PolicyValidationError::DuplicateCapability(_))
    ));
}

#[test]
fn policy_allows_valid_artifact() {
    common::sample_manifest().validate().unwrap();
}

#[test]
fn hybrid_mode_requires_signature_enforcement() {
    let err = ContractPolicy::new(
        vec![VmTarget::Wasm],
        vec![ArtifactFormat::WasmModule],
        1024,
        vec![],
        vec![],
        false,
        false,
        SourceTrustLevel::Trusted,
    )
    .unwrap()
    .with_quantum_security(QuantumSecurityProfile {
        migration_mode: QuantumMigrationMode::HybridDualSign,
        pq_signature_schemes: vec![PqSignatureScheme::MlDsa65],
        transition_epoch_start: Some(42),
        classical_retirement_epoch: Some(84),
        min_signature_bundles: 0,
    })
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Policy(PolicyValidationError::PolicyViolation(_))
    ));
}

#[test]
fn post_quantum_mode_requires_scheme_catalog() {
    let err = ContractPolicy::new(
        vec![VmTarget::Wasm],
        vec![ArtifactFormat::WasmModule],
        1024,
        vec![],
        vec![],
        false,
        true,
        SourceTrustLevel::Trusted,
    )
    .unwrap()
    .with_quantum_security(QuantumSecurityProfile {
        migration_mode: QuantumMigrationMode::PostQuantumOnly,
        pq_signature_schemes: vec![],
        transition_epoch_start: Some(42),
        classical_retirement_epoch: Some(84),
        min_signature_bundles: 1,
    })
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Policy(PolicyValidationError::PolicyViolation(_))
    ));
}

#[test]
fn transition_schedule_must_be_ordered() {
    let err = ContractPolicy::new(
        vec![VmTarget::Wasm],
        vec![ArtifactFormat::WasmModule],
        1024,
        vec![],
        vec![],
        false,
        true,
        SourceTrustLevel::Trusted,
    )
    .unwrap()
    .with_quantum_security(QuantumSecurityProfile {
        migration_mode: QuantumMigrationMode::HybridDualSign,
        pq_signature_schemes: vec![PqSignatureScheme::MlDsa65],
        transition_epoch_start: Some(500),
        classical_retirement_epoch: Some(500),
        min_signature_bundles: 2,
    })
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Policy(PolicyValidationError::PolicyViolation(_))
    ));
}

#[test]
fn post_quantum_policy_with_schedule_is_accepted() {
    ContractPolicy::new(
        vec![VmTarget::Wasm],
        vec![ArtifactFormat::WasmModule],
        1024,
        vec![],
        vec![],
        false,
        true,
        SourceTrustLevel::Trusted,
    )
    .unwrap()
    .with_quantum_security(QuantumSecurityProfile {
        migration_mode: QuantumMigrationMode::PostQuantumOnly,
        pq_signature_schemes: vec![
            PqSignatureScheme::MlDsa87,
            PqSignatureScheme::SlhDsaShake128f,
        ],
        transition_epoch_start: Some(100),
        classical_retirement_epoch: Some(220),
        min_signature_bundles: 1,
    })
    .unwrap();
}

// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

mod common;

use aoxcontract::{
    ArtifactFormat, ContractCapability, ContractError, ContractPolicy, PolicyValidationError,
    QuantumMigrationMode, QuantumSecurityProfile, SourceTrustLevel, Validate, VmTarget,
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
        pq_signature_schemes: vec!["ml_dsa_65".into()],
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
    })
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Policy(PolicyValidationError::PolicyViolation(_))
    ));
}

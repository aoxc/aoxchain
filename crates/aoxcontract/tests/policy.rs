// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

mod common;

use aoxcontract::{
    ArtifactFormat, ContractCapability, ContractError, ContractPolicy, PolicyValidationError,
    QuantumMigrationMode, QuantumSecurityProfile, SourceTrustLevel, Validate, VmTarget,
};
use serde_json::json;

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
        ..QuantumSecurityProfile::default()
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
        ..QuantumSecurityProfile::default()
    })
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Policy(PolicyValidationError::PolicyViolation(_))
    ));
}

#[test]
fn hybrid_mode_requires_scheme_catalog() {
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
        pq_signature_schemes: vec![],
        ..QuantumSecurityProfile::default()
    })
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Policy(PolicyValidationError::PolicyViolation(_))
    ));
}

#[test]
fn classical_mode_rejects_migration_epochs() {
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
        migration_mode: QuantumMigrationMode::ClassicalOnly,
        transition_epoch_start: Some(1200),
        pq_signature_schemes: vec![],
        ..QuantumSecurityProfile::default()
    })
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Policy(PolicyValidationError::PolicyViolation(_))
    ));
}

#[test]
fn post_quantum_only_rejects_classical_retirement_epoch() {
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
        classical_retirement_epoch: Some(1500),
        pq_signature_schemes: vec!["ml_dsa_65".into()],
        ..QuantumSecurityProfile::default()
    })
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Policy(PolicyValidationError::PolicyViolation(_))
    ));
}

#[test]
fn retirement_epoch_cannot_be_before_transition_epoch() {
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
        transition_epoch_start: Some(200),
        classical_retirement_epoch: Some(199),
        pq_signature_schemes: vec!["ml_dsa_65".into()],
        ..QuantumSecurityProfile::default()
    })
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Policy(PolicyValidationError::PolicyViolation(_))
    ));
}

#[test]
fn scheme_id_with_trailing_space_is_rejected() {
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
        pq_signature_schemes: vec!["ml_dsa_65 ".into()],
        ..QuantumSecurityProfile::default()
    })
    .unwrap_err();

    assert!(matches!(
        err,
        ContractError::Policy(PolicyValidationError::PolicyViolation(_))
    ));
}

#[test]
fn hybrid_policy_epoch_gates_and_bundle_floor_work() {
    let policy = ContractPolicy::new(
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
        transition_epoch_start: Some(100),
        classical_retirement_epoch: Some(200),
        min_signature_bundles: 2,
        pq_signature_schemes: vec!["ml_dsa_65".into()],
    })
    .unwrap();

    assert!(!policy.requires_post_quantum_signatures_at(150));
    assert!(policy.requires_post_quantum_signatures_at(200));
    assert_eq!(policy.minimum_required_signature_bundles_at(50), 1);
    assert_eq!(policy.minimum_required_signature_bundles_at(150), 2);
    assert_eq!(policy.minimum_required_signature_bundles_at(250), 2);
}

#[test]
fn post_quantum_mode_transition_epoch_is_respected() {
    let policy = ContractPolicy::new(
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
        transition_epoch_start: Some(500),
        min_signature_bundles: 3,
        pq_signature_schemes: vec!["falcon_1024".into()],
        ..QuantumSecurityProfile::default()
    })
    .unwrap();

    assert!(!policy.requires_post_quantum_signatures_at(499));
    assert!(policy.requires_post_quantum_signatures_at(500));
    assert_eq!(policy.minimum_required_signature_bundles_at(499), 1);
    assert_eq!(policy.minimum_required_signature_bundles_at(500), 3);
}

#[test]
fn legacy_quantum_security_json_deserializes_with_safe_defaults() {
    let profile: QuantumSecurityProfile = serde_json::from_value(json!({
        "migration_mode": "classical_only",
        "pq_signature_schemes": []
    }))
    .unwrap();

    assert_eq!(profile.transition_epoch_start, None);
    assert_eq!(profile.classical_retirement_epoch, None);
    assert_eq!(profile.min_signature_bundles, 1);
    assert_eq!(profile.pq_signature_schemes, Vec::<String>::new());
}

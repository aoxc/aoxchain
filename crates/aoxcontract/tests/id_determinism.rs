mod common;

use aoxcontract::{ContractId, canonical};

#[test]
fn same_input_produces_same_id() {
    let manifest = common::sample_manifest();
    let id1 = ContractId::derive(&manifest).unwrap();
    let id2 = ContractId::derive(&manifest).unwrap();
    assert_eq!(id1, id2);
}

#[test]
fn changing_artifact_changes_id() {
    let manifest = common::sample_manifest();
    let id1 = ContractId::derive(&manifest).unwrap();

    let mut changed = common::sample_manifest();
    changed.artifact.artifact_digest.value =
        "aaaaaaaaaac4d82f8d3808d2a0b5d7a4a8c29c5a1f9f2cbf3a1f0b9b9b654321".into();
    changed.digest = changed.artifact.artifact_digest.clone();
    changed.integrity.digest = changed.artifact.artifact_digest.clone();
    let id2 = ContractId::derive(&changed).unwrap();

    assert_ne!(id1, id2);
}

#[test]
fn canonical_output_is_stable() {
    let manifest = common::sample_manifest();
    let v1 = canonical::canonical_manifest_value(&manifest).unwrap();
    let v2 = canonical::canonical_manifest_value(&manifest).unwrap();
    assert_eq!(v1, v2);
}

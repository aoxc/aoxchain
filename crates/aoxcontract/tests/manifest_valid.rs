mod common;

use aoxcontract::{ContractDescriptor, ContractId, Validate, canonical};

#[test]
fn valid_manifest_is_accepted_and_roundtrips() {
    let manifest = common::sample_manifest();
    manifest.validate().unwrap();

    let bytes = canonical::canonical_manifest_bytes(&manifest).unwrap();
    let decoded: aoxcontract::ContractManifest = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decoded, manifest);

    let descriptor = ContractDescriptor::new(manifest.clone()).unwrap();
    let derived = ContractId::derive(&manifest).unwrap();
    assert_eq!(descriptor.contract_id, derived);
}

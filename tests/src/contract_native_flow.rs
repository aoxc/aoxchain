use aoxcdata::contracts::store::ContractStore;
use aoxconfig::contracts::ContractsConfig;
use aoxcontract::{
    ArtifactDigest, ArtifactDigestAlgorithm, ContractCapability, ContractDescriptor,
    ContractMetadata, ContractReviewStatus, Entrypoint, VmTarget,
};
use aoxcore::contract::receipt::ContractReceipt;
use aoxcore::contract::registry::ContractRegistry;
use aoxcsdk::contracts::builder::ContractManifestBuilder;
use aoxcvm::contracts::resolver::resolve_runtime_binding;

#[test]
fn native_contract_flow_connects_sdk_contract_core_data_and_vm() {
    let manifest = ContractManifestBuilder::new()
        .with_name("native_contract")
        .with_package("aox.native")
        .with_version("1.0.0")
        .with_contract_version("1.0.0")
        .with_vm_target(VmTarget::Wasm)
        .with_artifact_digest(ArtifactDigest {
            algorithm: ArtifactDigestAlgorithm::Sha256,
            value: "5f4dcc3b5aa765d61d8327deb882cf9922222222222222222222222222222222".into(),
        })
        .with_artifact_location("ipfs://native/contract.wasm")
        .with_metadata(ContractMetadata {
            display_name: "Native Contract".into(),
            description: Some("integration test".into()),
            author: Some("AOX".into()),
            organization: Some("AOX".into()),
            source_reference: None,
            tags: vec!["native".into()],
            created_at: None,
            updated_at: None,
            audit_reference: None,
            notes: None,
        })
        .add_entrypoint(Entrypoint::new("execute", VmTarget::Wasm, None, vec![]).unwrap())
        .allow_capability(ContractCapability::StorageRead)
        .build()
        .unwrap();

    let descriptor = ContractDescriptor::new(manifest).unwrap();
    let binding = resolve_runtime_binding(&descriptor, &ContractsConfig::default()).unwrap();
    assert_eq!(binding.vm_target, VmTarget::Wasm);

    let mut registry = ContractRegistry::default();
    let receipt = registry.register_contract(descriptor.clone(), 7).unwrap();
    assert!(matches!(receipt, ContractReceipt::Registered(_)));

    let record = registry
        .get_contract(&descriptor.contract_id)
        .unwrap()
        .clone();
    let mut store = ContractStore::default();
    store.put(record.clone());
    assert_eq!(
        store.get(&descriptor.contract_id).unwrap().contract_id,
        descriptor.contract_id
    );

    let approval = aoxcontract::ApprovalMarker {
        reviewer: "security-team".into(),
        status: ContractReviewStatus::Approved,
        note: Some("ready".into()),
    };
    assert_eq!(approval.reviewer, "security-team");
}

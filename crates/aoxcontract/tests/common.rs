use aoxcontract::{
    ArtifactDigest, ArtifactDigestAlgorithm, ArtifactFormat, ArtifactLocationKind, Compatibility,
    ContractArtifactRef, ContractCapability, ContractManifest, ContractMetadata, ContractPolicy,
    ContractVersion, Entrypoint, Integrity, NetworkClass, RuntimeFamily, SourceTrustLevel,
    VmTarget,
};

pub fn sample_manifest() -> ContractManifest {
    let digest = ArtifactDigest {
        algorithm: ArtifactDigestAlgorithm::Sha256,
        value: "f34d6e0eaac4d82f8d3808d2a0b5d7a4a8c29c5a1f9f2cbf3a1f0b9b9b123456".into(),
    };

    let artifact = ContractArtifactRef::new(
        digest.clone(),
        4096,
        ArtifactFormat::WasmModule,
        ArtifactLocationKind::Uri,
        "ipfs://aox/demo-artifact.wasm",
        None,
        Some("application/wasm".into()),
        VmTarget::Wasm,
    )
    .unwrap();

    let policy = ContractPolicy::new(
        vec![VmTarget::Wasm],
        vec![ArtifactFormat::WasmModule],
        10 * 1024 * 1024,
        vec![ContractCapability::StorageRead],
        vec![ContractCapability::TreasurySensitive],
        true,
        true,
        SourceTrustLevel::ReviewRequired,
    )
    .unwrap();

    let compatibility = Compatibility::new(
        1,
        vec![1],
        vec![RuntimeFamily::Wasm],
        vec![NetworkClass::Mainnet, NetworkClass::Testnet],
        vec![VmTarget::Evm],
        false,
    )
    .unwrap();

    let metadata = ContractMetadata {
        display_name: "Treasury Guard".into(),
        description: Some("Example canonical manifest fixture".into()),
        author: Some("AOXChain".into()),
        organization: Some("AOX".into()),
        source_reference: Some("https://example.invalid/src".into()),
        tags: vec!["finance".into(), "guard".into()],
        created_at: None,
        updated_at: None,
        audit_reference: Some("AUD-001".into()),
        notes: Some("fixture".into()),
    };

    let integrity = Integrity {
        digest: digest.clone(),
        artifact_size: 4096,
        artifact_format: ArtifactFormat::WasmModule,
        media_type: Some("application/wasm".into()),
        signature_required: true,
        source_trust_level: SourceTrustLevel::ReviewRequired,
    };

    ContractManifest::new(
        "treasury_guard",
        "aox.finance",
        "1.0.0",
        ContractVersion("1.0.0".into()),
        VmTarget::Wasm,
        artifact,
        vec![Entrypoint::new("execute", VmTarget::Wasm, None, vec![]).unwrap()],
        digest,
        metadata,
        policy,
        compatibility,
        integrity,
        1,
    )
    .unwrap()
}

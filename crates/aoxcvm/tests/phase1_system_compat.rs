use aoxconfig::contracts::ContractsConfig;
use aoxcontract::{
    ArtifactDigest, ArtifactDigestAlgorithm, ContractDescriptor, ContractMetadata, Entrypoint,
    VmTarget,
};
use aoxcvm::{
    contracts::resolver::resolve_runtime_binding,
    vm::{
        machine::{Instruction, Program},
        phase1::{
            BasicAuthVerifier, BasicObjectVerifier, ExecutionContract, InMemoryHost, SpecError,
            VmSpec, execute,
        },
    },
};

use aoxcvm::auth::{
    envelope::{AuthEnvelope, SignatureEntry},
    scheme::SignatureAlgorithm,
};
use aoxcvm::context::{
    block::BlockContext, call::CallContext, environment::EnvironmentContext,
    execution::ExecutionContext, origin::OriginContext, tx::TxContext,
};
use aoxcvm::tx::{envelope::TxEnvelope, fee::FeeBudget, kind::TxKind, payload::TxPayload};

fn descriptor(vm_target: VmTarget) -> ContractDescriptor {
    let manifest = aoxcsdk::contracts::builder::ContractManifestBuilder::new()
        .with_name("phase1_integration")
        .with_package("aox.phase1")
        .with_version("1.0.0")
        .with_contract_version("1.0.0")
        .with_vm_target(vm_target)
        .with_artifact_digest(ArtifactDigest {
            algorithm: ArtifactDigestAlgorithm::Sha256,
            value: "5f4dcc3b5aa765d61d8327deb882cf9922222222222222222222222222222222".into(),
        })
        .with_artifact_location("ipfs://phase1/module")
        .with_metadata(ContractMetadata {
            display_name: "Phase1 Integration".into(),
            description: Some("phase1 integration".into()),
            author: Some("AOX".into()),
            organization: Some("AOX".into()),
            source_reference: None,
            tags: vec!["phase1".into()],
            created_at: None,
            updated_at: None,
            audit_reference: Some("approved".into()),
            notes: None,
        })
        .add_entrypoint(Entrypoint::new("execute", VmTarget::Wasm, None, vec![]).unwrap())
        .build()
        .unwrap();

    ContractDescriptor::new(manifest).unwrap()
}

fn custom_descriptor(custom_id: &str) -> ContractDescriptor {
    let target = VmTarget::Custom(custom_id.to_string());
    let manifest = aoxcsdk::contracts::builder::ContractManifestBuilder::new()
        .with_name("phase1_custom")
        .with_package("aox.phase1.custom")
        .with_version("1.0.0")
        .with_contract_version("1.0.0")
        .with_vm_target(target.clone())
        .with_artifact_digest(ArtifactDigest {
            algorithm: ArtifactDigestAlgorithm::Sha256,
            value: "7f4dcc3b5aa765d61d8327deb882cf9922222222222222222222222222222222".into(),
        })
        .with_artifact_location("ipfs://phase1/custom")
        .with_metadata(ContractMetadata {
            display_name: "Phase1 Custom".into(),
            description: Some("phase1 custom".into()),
            author: Some("AOX".into()),
            organization: Some("AOX".into()),
            source_reference: None,
            tags: vec!["phase1".into(), "custom".into()],
            created_at: None,
            updated_at: None,
            audit_reference: Some("approved".into()),
            notes: None,
        })
        .add_entrypoint(Entrypoint::new("execute", target, None, vec![]).unwrap())
        .build()
        .unwrap();

    ContractDescriptor::new(manifest).unwrap()
}

fn tx() -> ExecutionContract {
    let tx = TxEnvelope::new(
        2626,
        1,
        TxKind::UserCall,
        FeeBudget::new(50, 1),
        TxPayload::new(vec![1]),
    );

    let auth = AuthEnvelope {
        domain: "tx".to_string(),
        nonce: 1,
        signers: vec![SignatureEntry {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: "k1".to_string(),
            signature: vec![7_u8; 64],
        }],
    };

    let context = ExecutionContext::new(
        EnvironmentContext::new(2626, 1),
        BlockContext::new(1, 0, 0, [0_u8; 32]),
        TxContext::new([0_u8; 32], 0, 50, false, 1, 0),
        CallContext::new(0),
        OriginContext::new([0_u8; 32], [0_u8; 32], [0_u8; 32], 0),
    );

    ExecutionContract {
        tx,
        auth,
        context,
        object: vec![1, 2, 3],
        program: Program {
            code: vec![Instruction::Push(9), Instruction::Halt],
        },
    }
}

#[test]
fn phase1_public_api_executes_through_single_entrypoint() {
    let desc = descriptor(VmTarget::Wasm);
    let spec = VmSpec::from_config(&ContractsConfig::default(), &desc).unwrap();

    let outcome = execute(
        &tx(),
        &mut InMemoryHost::default(),
        spec,
        &BasicAuthVerifier,
        &BasicObjectVerifier,
    )
    .unwrap();
    assert!(outcome.vm_error.is_none());
    assert_eq!(outcome.stack, vec![9]);
}

#[test]
fn phase1_vm_spec_is_fail_closed_when_config_disables_target() {
    let mut cfg = ContractsConfig::default();
    cfg.artifact_policy.allowed_vm_targets = vec![VmTarget::Evm];

    let err = VmSpec::from_config(&cfg, &descriptor(VmTarget::Wasm)).unwrap_err();
    assert!(matches!(err, SpecError::VmTargetDisabledByConfig));
}

#[test]
fn runtime_resolver_is_fail_closed_with_config() {
    let mut cfg = ContractsConfig::default();
    cfg.artifact_policy.allowed_vm_targets = vec![VmTarget::Evm];

    let err = resolve_runtime_binding(&descriptor(VmTarget::Wasm), &cfg).unwrap_err();
    assert!(err.to_string().contains("disabled"));
}

#[test]
fn phase1_vm_spec_custom_target_requires_exact_match() {
    let mut cfg = ContractsConfig::default();
    cfg.artifact_policy.allowed_vm_targets = vec![VmTarget::Custom("qml-v2".to_string())];

    let err = VmSpec::from_config(&cfg, &custom_descriptor("qml-v1")).unwrap_err();
    assert!(matches!(err, SpecError::VmTargetDisabledByConfig));
}

#[test]
fn phase1_vm_spec_custom_target_allows_exact_match() {
    let mut cfg = ContractsConfig::default();
    cfg.artifact_policy.allowed_vm_targets = vec![VmTarget::Custom("qml-v1".to_string())];

    let spec = VmSpec::from_config(&cfg, &custom_descriptor("qml-v1")).unwrap();
    assert_eq!(spec, VmSpec::default());
}

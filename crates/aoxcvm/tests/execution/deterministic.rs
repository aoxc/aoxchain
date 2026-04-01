use aoxcvm::auth::envelope::AuthEnvelope;
use aoxcvm::auth::scheme::AuthScheme;
use aoxcvm::auth::verifier::AuthVerifierPolicy;
use aoxcvm::constants::VM_CANONICAL_CHAIN_DOMAIN;
use aoxcvm::domains::Domain;
use aoxcvm::engine::lifecycle::run_transaction;
use aoxcvm::feature_flags::FeatureFlags;
use aoxcvm::limits::ExecutionLimits;
use aoxcvm::policy::vm_policy::VmPolicy;
use aoxcvm::tx::envelope::TransactionEnvelope;

#[test]
fn deterministic_flow_accepts_valid_transaction() {
    let tx = TransactionEnvelope {
        chain_domain: VM_CANONICAL_CHAIN_DOMAIN.to_string(),
        tx_hash: [7; 32],
        auth: AuthEnvelope {
            scheme: AuthScheme::HybridEd25519MlDsa,
            signer_key_id: [9; 32],
            domain: Domain::L1Transaction,
            nonce: 10,
            expiry_epoch: 100,
            payload_digest: [3; 32],
            capability_scope: vec!["asset.transfer".into()],
        },
        max_gas: 1_000,
        max_authority: 50,
        target_package: "core.asset".into(),
        target_entrypoint: "transfer".into(),
    };

    let policy = VmPolicy {
        protocol_version: 1,
        limits: ExecutionLimits::default(),
        features: FeatureFlags { pq_auth_primary: true, authority_metering: true, deterministic_host_v2: true },
    };

    let result = run_transaction(&tx, 10, 1, AuthVerifierPolicy { require_pq_scheme: true, max_nonce_gap: 100 }, &policy)
        .expect("valid deterministic execution should succeed");

    assert!(result.success);
    assert_eq!(result.diff.writes.len(), 1);
}

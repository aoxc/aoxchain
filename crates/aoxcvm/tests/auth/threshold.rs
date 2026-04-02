use aoxcvm::auth::{
    envelope::{AuthEnvelope, SignatureEntry},
    scheme::SignatureAlgorithm,
    threshold::ThresholdPolicy,
};

#[test]
fn threshold_policy_rejects_when_min_signers_not_met() {
    let envelope = AuthEnvelope {
        domain: "tx".to_owned(),
        nonce: 1,
        signers: vec![SignatureEntry {
            algorithm: SignatureAlgorithm::MlDsa65,
            key_id: "pq-a".to_owned(),
            signature: vec![5_u8; 256],
        }],
    };

    let policy = ThresholdPolicy {
        min_signers: 2,
        require_post_quantum: false,
    };

    assert!(!policy.is_satisfied_by(&envelope));
}

#[test]
fn threshold_policy_enforces_post_quantum_presence() {
    let envelope = AuthEnvelope {
        domain: "tx".to_owned(),
        nonce: 2,
        signers: vec![
            SignatureEntry {
                algorithm: SignatureAlgorithm::Ed25519,
                key_id: "classic-a".to_owned(),
                signature: vec![6_u8; 64],
            },
            SignatureEntry {
                algorithm: SignatureAlgorithm::EcdsaP256,
                key_id: "classic-b".to_owned(),
                signature: vec![7_u8; 64],
            },
        ],
    };

    let policy = ThresholdPolicy {
        min_signers: 2,
        require_post_quantum: true,
    };

    assert!(!policy.is_satisfied_by(&envelope));
}

use aoxcvm::auth::{
    envelope::{AuthEnvelope, SignatureEntry},
    hybrid::HybridPolicy,
    scheme::SignatureAlgorithm,
};

fn signer(algorithm: SignatureAlgorithm, key_id: &str, signature_len: usize) -> SignatureEntry {
    SignatureEntry {
        algorithm,
        key_id: key_id.to_owned(),
        signature: vec![11_u8; signature_len],
    }
}

#[test]
fn hybrid_policy_accepts_classic_plus_pq() {
    let envelope = AuthEnvelope {
        domain: "tx".to_owned(),
        nonce: 10,
        signers: vec![
            signer(SignatureAlgorithm::Ed25519, "classic-a", 64),
            signer(SignatureAlgorithm::MlDsa65, "pq-a", 2048),
        ],
    };

    assert!(HybridPolicy::default().validate(&envelope).is_ok());
}

#[test]
fn hybrid_policy_rejects_classic_only_set() {
    let envelope = AuthEnvelope {
        domain: "tx".to_owned(),
        nonce: 11,
        signers: vec![signer(SignatureAlgorithm::Ed25519, "classic-only", 64)],
    };

    assert!(HybridPolicy::default().validate(&envelope).is_err());
}

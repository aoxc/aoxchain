use aoxcvm::auth::{
    envelope::{AuthEnvelope, SignatureEntry},
    replay::ReplayGuard,
    scheme::SignatureAlgorithm,
};

fn envelope(nonce: u64) -> AuthEnvelope {
    AuthEnvelope {
        domain: "tx".to_owned(),
        nonce,
        signers: vec![SignatureEntry {
            algorithm: SignatureAlgorithm::MlDsa65,
            key_id: "pq".to_owned(),
            signature: vec![9_u8; 256],
        }],
    }
}

#[test]
fn replay_guard_rejects_nonce_replay() {
    let mut guard = ReplayGuard::new();

    assert!(guard.admit("alice", &envelope(7)));
    assert!(!guard.admit("alice", &envelope(7)));
    assert!(!guard.admit("alice", &envelope(6)));
    assert_eq!(guard.highest_nonce("alice"), Some(7));
}

#[test]
fn replay_guard_allows_strictly_newer_nonce() {
    let mut guard = ReplayGuard::new();

    assert!(guard.admit("alice", &envelope(9)));
    assert!(guard.admit("alice", &envelope(10)));
    assert_eq!(guard.highest_nonce("alice"), Some(10));
}

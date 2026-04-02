use aoxcvm::auth::{
    envelope::{AuthEnvelope, AuthEnvelopeLimits, SignatureEntry},
    scheme::{AuthProfile, SignatureAlgorithm},
};
use aoxcvm::errors::AoxcvmError;

#[test]
fn legacy_profile_accepts_ed25519_signers() {
    let envelope = AuthEnvelope {
        domain: "tx".to_owned(),
        nonce: 1,
        signers: vec![SignatureEntry {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: "legacy-ed25519".to_owned(),
            signature: vec![42_u8; 64],
        }],
    };

    assert!(
        envelope
            .validate(AuthProfile::Legacy, AuthEnvelopeLimits::default())
            .is_ok()
    );
}

#[test]
fn legacy_profile_rejects_oversized_signature() {
    let limits = AuthEnvelopeLimits {
        max_signatures: 2,
        max_signature_bytes: 32,
    };

    let envelope = AuthEnvelope {
        domain: "tx".to_owned(),
        nonce: 2,
        signers: vec![SignatureEntry {
            algorithm: SignatureAlgorithm::Ed25519,
            key_id: "legacy-ed25519".to_owned(),
            signature: vec![1_u8; 64],
        }],
    };

    let error = envelope.validate(AuthProfile::Legacy, limits).unwrap_err();
    assert_eq!(
        error,
        AoxcvmError::AuthLimitExceeded {
            limit: "max_signature_bytes",
            got: 64,
            max: 32,
        }
    );
}

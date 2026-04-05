use super::core::{MAX_NONCE_B64_LEN, MAX_SALT_B64_LEN};
use super::*;
use base64::{Engine, engine::general_purpose};

#[test]
fn encrypt_then_decrypt_roundtrip_succeeds() {
    let plaintext = b"super-secret-key-material";
    let password = "correct horse battery staple";

    let serialized = encrypt_key(plaintext, password).expect("encryption must succeed");
    let decrypted = decrypt_key(&serialized, password).expect("decryption must succeed");

    assert_eq!(decrypted, plaintext);
}

#[test]
fn decrypt_fails_with_wrong_password() {
    let plaintext = b"super-secret-key-material";
    let password = "correct horse battery staple";

    let serialized = encrypt_key(plaintext, password).expect("encryption must succeed");
    let result = decrypt_key(&serialized, "wrong password");

    assert_eq!(result, Err(KeyfileError::DecryptionFailed));
}

#[test]
fn empty_password_is_rejected() {
    let result = encrypt_key(b"abc", "");
    assert_eq!(result, Err(KeyfileError::EmptyPassword));
}

#[test]
fn password_with_surrounding_whitespace_is_rejected() {
    let result = encrypt_key(b"abc", " password ");
    assert_eq!(result, Err(KeyfileError::EmptyPassword));
}

#[test]
fn envelope_validation_accepts_valid_output() {
    let envelope = encrypt_key_to_envelope(b"abc", "password").expect("encryption must succeed");
    assert_eq!(validate_envelope(&envelope), Ok(()));
}

#[test]
fn serialized_keyfile_reports_validity() {
    let serialized = encrypt_key(b"abc", "password").expect("encryption must succeed");
    assert!(is_valid_keyfile(&serialized));
}

#[test]
fn validate_envelope_rejects_invalid_version() {
    let mut envelope =
        encrypt_key_to_envelope(b"abc", "password").expect("encryption must succeed");
    envelope.version = 99;

    assert_eq!(
        validate_envelope(&envelope),
        Err(KeyfileError::InvalidVersion)
    );
}

#[test]
fn validate_envelope_rejects_invalid_kdf_algorithm() {
    let mut envelope =
        encrypt_key_to_envelope(b"abc", "password").expect("encryption must succeed");
    envelope.kdf.algorithm = "pbkdf2".to_string();

    assert_eq!(
        validate_envelope(&envelope),
        Err(KeyfileError::InvalidKdfAlgorithm)
    );
}

#[test]
fn validate_envelope_rejects_invalid_salt_length() {
    let mut envelope =
        encrypt_key_to_envelope(b"abc", "password").expect("encryption must succeed");
    envelope.salt_b64 = general_purpose::STANDARD.encode([0u8; 8]);

    assert_eq!(
        validate_envelope(&envelope),
        Err(KeyfileError::InvalidSaltLength)
    );
}

#[test]
fn validate_envelope_rejects_oversized_salt_b64_before_decode() {
    let mut envelope =
        encrypt_key_to_envelope(b"abc", "password").expect("encryption must succeed");
    envelope.salt_b64 = "A".repeat(MAX_SALT_B64_LEN + 1);

    assert_eq!(
        validate_envelope(&envelope),
        Err(KeyfileError::InvalidSaltLength)
    );
}

#[test]
fn validate_envelope_rejects_invalid_nonce_length() {
    let mut envelope =
        encrypt_key_to_envelope(b"abc", "password").expect("encryption must succeed");
    envelope.nonce_b64 = general_purpose::STANDARD.encode([0u8; 8]);

    assert_eq!(
        validate_envelope(&envelope),
        Err(KeyfileError::InvalidNonceLength)
    );
}

#[test]
fn validate_envelope_rejects_oversized_nonce_b64_before_decode() {
    let mut envelope =
        encrypt_key_to_envelope(b"abc", "password").expect("encryption must succeed");
    envelope.nonce_b64 = "A".repeat(MAX_NONCE_B64_LEN + 1);

    assert_eq!(
        validate_envelope(&envelope),
        Err(KeyfileError::InvalidNonceLength)
    );
}

#[test]
fn validate_envelope_rejects_empty_ciphertext() {
    let mut envelope =
        encrypt_key_to_envelope(b"abc", "password").expect("encryption must succeed");
    envelope.ciphertext_b64 = general_purpose::STANDARD.encode(Vec::<u8>::new());

    assert!(matches!(
        validate_envelope(&envelope),
        Err(KeyfileError::InvalidCiphertext | KeyfileError::InvalidFormat)
    ));
}

#[test]
fn envelope_fingerprint_is_stable() {
    let envelope = encrypt_key_to_envelope(b"abc", "password").expect("encryption must succeed");

    let a = envelope.fingerprint().expect("fingerprint must succeed");
    let b = envelope.fingerprint().expect("fingerprint must succeed");

    assert_eq!(a, b);
    assert_eq!(a.len(), 16);
}

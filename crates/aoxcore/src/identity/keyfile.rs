use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};

use argon2::{Algorithm, Argon2, Params, Version};

use base64::{Engine, engine::general_purpose};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Canonical domain prefix embedded into serialized AOXC keyfiles.
///
/// This value is authenticated as associated data during encryption and
/// decryption in order to prevent cross-protocol blob confusion.
const AOXC_KEYFILE_DOMAIN: &[u8] = b"AOXC-KEYFILE-V1";

/// Canonical AOXC keyfile format version.
const KEYFILE_VERSION: u8 = 1;

/// Salt length in bytes for Argon2id derivation.
const SALT_LEN: usize = 16;

/// AES-GCM nonce length in bytes.
const NONCE_LEN: usize = 12;

/// AES-256 key length in bytes.
const KEY_LEN: usize = 32;

/// Default Argon2 memory cost in KiB.
const DEFAULT_ARGON2_M_COST: u32 = 65_536;

/// Default Argon2 time cost.
const DEFAULT_ARGON2_T_COST: u32 = 3;

/// Default Argon2 parallelism cost.
const DEFAULT_ARGON2_P_COST: u32 = 1;

/// Canonical serialized keyfile envelope.
///
/// This structure is suitable for persistence, transport, and deterministic
/// format validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyfileEnvelope {
    pub version: u8,
    pub kdf: KeyfileKdf,
    pub salt_b64: String,
    pub nonce_b64: String,
    pub ciphertext_b64: String,
}

/// KDF metadata embedded into the keyfile envelope.
///
/// Storing KDF parameters makes future migrations and deterministic
/// re-derivation possible.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyfileKdf {
    pub algorithm: String,
    pub memory_cost_kib: u32,
    pub time_cost: u32,
    pub parallelism: u32,
    pub output_len: usize,
}

/// Canonical keyfile error surface.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyfileError {
    EmptyPassword,
    InvalidArgon2Params,
    KeyDerivationFailed,
    EncryptionFailed,
    DecryptionFailed,
    InvalidFormat,
    InvalidVersion,
    InvalidBase64,
    InvalidSaltLength,
    InvalidNonceLength,
    InvalidCiphertext,
    SerializationFailed(String),
    DeserializationFailed(String),
}

impl KeyfileError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyPassword => "KEYFILE_EMPTY_PASSWORD",
            Self::InvalidArgon2Params => "KEYFILE_INVALID_ARGON2_PARAMS",
            Self::KeyDerivationFailed => "KEYFILE_KEY_DERIVATION_FAILED",
            Self::EncryptionFailed => "KEYFILE_ENCRYPTION_FAILED",
            Self::DecryptionFailed => "KEYFILE_DECRYPTION_FAILED",
            Self::InvalidFormat => "KEYFILE_INVALID_FORMAT",
            Self::InvalidVersion => "KEYFILE_INVALID_VERSION",
            Self::InvalidBase64 => "KEYFILE_INVALID_BASE64",
            Self::InvalidSaltLength => "KEYFILE_INVALID_SALT_LENGTH",
            Self::InvalidNonceLength => "KEYFILE_INVALID_NONCE_LENGTH",
            Self::InvalidCiphertext => "KEYFILE_INVALID_CIPHERTEXT",
            Self::SerializationFailed(_) => "KEYFILE_SERIALIZATION_FAILED",
            Self::DeserializationFailed(_) => "KEYFILE_DESERIALIZATION_FAILED",
        }
    }
}

impl fmt::Display for KeyfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPassword => {
                write!(f, "keyfile operation failed: password must not be empty")
            }
            Self::InvalidArgon2Params => {
                write!(f, "keyfile operation failed: invalid Argon2 parameters")
            }
            Self::KeyDerivationFailed => {
                write!(f, "keyfile operation failed: key derivation failed")
            }
            Self::EncryptionFailed => {
                write!(f, "keyfile operation failed: AES-GCM encryption failed")
            }
            Self::DecryptionFailed => {
                write!(f, "keyfile operation failed: AES-GCM decryption failed")
            }
            Self::InvalidFormat => {
                write!(f, "keyfile operation failed: serialized format is invalid")
            }
            Self::InvalidVersion => {
                write!(f, "keyfile operation failed: unsupported keyfile version")
            }
            Self::InvalidBase64 => {
                write!(f, "keyfile operation failed: base64 decoding failed")
            }
            Self::InvalidSaltLength => {
                write!(f, "keyfile operation failed: salt length is invalid")
            }
            Self::InvalidNonceLength => {
                write!(f, "keyfile operation failed: nonce length is invalid")
            }
            Self::InvalidCiphertext => {
                write!(f, "keyfile operation failed: ciphertext is invalid")
            }
            Self::SerializationFailed(error) => {
                write!(
                    f,
                    "keyfile operation failed: serialization failed: {}",
                    error
                )
            }
            Self::DeserializationFailed(error) => {
                write!(
                    f,
                    "keyfile operation failed: deserialization failed: {}",
                    error
                )
            }
        }
    }
}

impl std::error::Error for KeyfileError {}

/// Encrypts raw key material using:
/// - Argon2id for password-based key derivation
/// - AES-256-GCM for authenticated encryption
///
/// Returns a serialized JSON envelope.
pub fn encrypt_key(data: &[u8], password: &str) -> Result<String, KeyfileError> {
    let envelope = encrypt_key_to_envelope(data, password)?;

    serde_json::to_string_pretty(&envelope)
        .map_err(|error| KeyfileError::SerializationFailed(error.to_string()))
}

/// Encrypts raw key material and returns the structured envelope directly.
pub fn encrypt_key_to_envelope(
    data: &[u8],
    password: &str,
) -> Result<KeyfileEnvelope, KeyfileError> {
    if password.trim().is_empty() {
        return Err(KeyfileError::EmptyPassword);
    }

    let mut rng = rand::rng();

    let mut salt = [0u8; SALT_LEN];
    rng.fill(&mut salt);

    let params = default_argon2_params()?;

    let mut key_bytes = [0u8; KEY_LEN];
    derive_key(password, &salt, &params, &mut key_bytes)?;

    let cipher =
        Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| KeyfileError::EncryptionFailed)?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes);

    let nonce = Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: data,
                aad: AOXC_KEYFILE_DOMAIN,
            },
        )
        .map_err(|_| KeyfileError::EncryptionFailed)?;

    Ok(KeyfileEnvelope {
        version: KEYFILE_VERSION,
        kdf: KeyfileKdf {
            algorithm: "argon2id".to_string(),
            memory_cost_kib: DEFAULT_ARGON2_M_COST,
            time_cost: DEFAULT_ARGON2_T_COST,
            parallelism: DEFAULT_ARGON2_P_COST,
            output_len: KEY_LEN,
        },
        salt_b64: general_purpose::STANDARD.encode(salt),
        nonce_b64: general_purpose::STANDARD.encode(nonce_bytes),
        ciphertext_b64: general_purpose::STANDARD.encode(ciphertext),
    })
}

/// Decrypts a serialized JSON keyfile produced by [`encrypt_key`].
pub fn decrypt_key(serialized: &str, password: &str) -> Result<Vec<u8>, KeyfileError> {
    let envelope: KeyfileEnvelope = serde_json::from_str(serialized)
        .map_err(|error| KeyfileError::DeserializationFailed(error.to_string()))?;

    decrypt_key_from_envelope(&envelope, password)
}

/// Decrypts a structured keyfile envelope.
pub fn decrypt_key_from_envelope(
    envelope: &KeyfileEnvelope,
    password: &str,
) -> Result<Vec<u8>, KeyfileError> {
    if password.trim().is_empty() {
        return Err(KeyfileError::EmptyPassword);
    }

    validate_envelope(envelope)?;

    let salt = general_purpose::STANDARD
        .decode(&envelope.salt_b64)
        .map_err(|_| KeyfileError::InvalidBase64)?;

    if salt.len() != SALT_LEN {
        return Err(KeyfileError::InvalidSaltLength);
    }

    let nonce_vec = general_purpose::STANDARD
        .decode(&envelope.nonce_b64)
        .map_err(|_| KeyfileError::InvalidBase64)?;

    let nonce_bytes: [u8; NONCE_LEN] = nonce_vec
        .as_slice()
        .try_into()
        .map_err(|_| KeyfileError::InvalidNonceLength)?;

    let ciphertext = general_purpose::STANDARD
        .decode(&envelope.ciphertext_b64)
        .map_err(|_| KeyfileError::InvalidBase64)?;

    if ciphertext.is_empty() {
        return Err(KeyfileError::InvalidCiphertext);
    }

    let params = params_from_envelope(&envelope.kdf)?;

    let mut key_bytes = [0u8; KEY_LEN];
    derive_key(password, &salt, &params, &mut key_bytes)?;

    let cipher =
        Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| KeyfileError::DecryptionFailed)?;

    let nonce = Nonce::from(nonce_bytes);

    cipher
        .decrypt(
            &nonce,
            Payload {
                msg: ciphertext.as_ref(),
                aad: AOXC_KEYFILE_DOMAIN,
            },
        )
        .map_err(|_| KeyfileError::DecryptionFailed)
}

/// Returns true if a serialized keyfile is structurally valid JSON and envelope-valid.
#[must_use]
pub fn is_valid_keyfile(serialized: &str) -> bool {
    match serde_json::from_str::<KeyfileEnvelope>(serialized) {
        Ok(envelope) => validate_envelope(&envelope).is_ok(),
        Err(_) => false,
    }
}

/// Validates a structured envelope without attempting decryption.
pub fn validate_envelope(envelope: &KeyfileEnvelope) -> Result<(), KeyfileError> {
    if envelope.version != KEYFILE_VERSION {
        return Err(KeyfileError::InvalidVersion);
    }

    if !envelope
        .kdf
        .algorithm
        .trim()
        .eq_ignore_ascii_case("argon2id")
    {
        return Err(KeyfileError::InvalidFormat);
    }

    if envelope.kdf.output_len != KEY_LEN {
        return Err(KeyfileError::InvalidFormat);
    }

    if envelope.salt_b64.trim().is_empty()
        || envelope.nonce_b64.trim().is_empty()
        || envelope.ciphertext_b64.trim().is_empty()
    {
        return Err(KeyfileError::InvalidFormat);
    }

    Ok(())
}

/// Builds the default Argon2 parameter set.
fn default_argon2_params() -> Result<Params, KeyfileError> {
    Params::new(
        DEFAULT_ARGON2_M_COST,
        DEFAULT_ARGON2_T_COST,
        DEFAULT_ARGON2_P_COST,
        Some(KEY_LEN),
    )
    .map_err(|_| KeyfileError::InvalidArgon2Params)
}

/// Rebuilds Argon2 parameters from persisted KDF metadata.
fn params_from_envelope(kdf: &KeyfileKdf) -> Result<Params, KeyfileError> {
    Params::new(
        kdf.memory_cost_kib,
        kdf.time_cost,
        kdf.parallelism,
        Some(kdf.output_len),
    )
    .map_err(|_| KeyfileError::InvalidArgon2Params)
}

/// Derives an AES key from password and salt using Argon2id.
fn derive_key(
    password: &str,
    salt: &[u8],
    params: &Params,
    out: &mut [u8; KEY_LEN],
) -> Result<(), KeyfileError> {
    if params.output_len().unwrap_or(KEY_LEN) != KEY_LEN {
        return Err(KeyfileError::InvalidArgon2Params);
    }

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params.clone());

    argon2
        .hash_password_into(password.as_bytes(), salt, out)
        .map_err(|_| KeyfileError::KeyDerivationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn envelope_validation_accepts_valid_output() {
        let envelope =
            encrypt_key_to_envelope(b"abc", "password").expect("encryption must succeed");
        assert_eq!(validate_envelope(&envelope), Ok(()));
    }

    #[test]
    fn serialized_keyfile_reports_validity() {
        let serialized = encrypt_key(b"abc", "password").expect("encryption must succeed");
        assert!(is_valid_keyfile(&serialized));
    }
}

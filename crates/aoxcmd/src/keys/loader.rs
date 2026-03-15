use aoxcore::identity::actor_id::generate_and_validate_actor_id;
use aoxcore::identity::ca::CertificateAuthority;
use aoxcore::identity::certificate::Certificate;
use aoxcore::identity::keyfile::{decrypt_key, encrypt_key, is_valid_keyfile};
use aoxcore::identity::passport::Passport;
use aoxcore::identity::pq_keys::{
    generate_keypair, public_key_from_bytes, secret_key_from_bytes, serialize_public_key,
    serialize_secret_key,
};

use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::material::KeyMaterial;
use super::paths::KeyPaths;

/// Bootstrap request describing how local key material should be created or loaded.
#[derive(Debug, Clone)]
pub struct KeyBootstrapRequest {
    pub chain: String,
    pub role: String,
    pub zone: String,
    pub password: String,
    pub certificate_validity_secs: u64,
}

impl KeyBootstrapRequest {
    /// Creates a new bootstrap request.
    #[must_use]
    pub fn new(
        chain: impl Into<String>,
        role: impl Into<String>,
        zone: impl Into<String>,
        password: impl Into<String>,
        certificate_validity_secs: u64,
    ) -> Self {
        Self {
            chain: chain.into(),
            role: role.into(),
            zone: zone.into(),
            password: password.into(),
            certificate_validity_secs,
        }
    }
}

/// Internal persisted bundle used by `aoxcmd` to reconstruct public identity
/// without decrypting auxiliary files separately.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedKeyBundle {
    actor_id: String,
    role: String,
    zone: String,
    public_key_hex: String,
    encrypted_secret_key: String,
}

/// Canonical loader error surface for key lifecycle operations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyLoaderError {
    EmptyPassword,
    InvalidValidityWindow,
    FilesystemError(String),
    InvalidKeyfile,
    SecretKeyDecryptFailed(String),
    SecretKeyRestoreFailed(String),
    PublicKeyRestoreFailed(String),
    ActorIdGenerationFailed(String),
    CertificateBuildFailed(String),
    CertificateSignFailed(String),
    CertificateSerializeFailed(String),
    CertificateParseFailed(String),
    PassportSerializeFailed(String),
    PassportParseFailed(String),
    BundleSerializeFailed(String),
    BundleParseFailed(String),
    TimeError,
}

impl KeyLoaderError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyPassword => "KEY_LOADER_EMPTY_PASSWORD",
            Self::InvalidValidityWindow => "KEY_LOADER_INVALID_VALIDITY_WINDOW",
            Self::FilesystemError(_) => "KEY_LOADER_FILESYSTEM_ERROR",
            Self::InvalidKeyfile => "KEY_LOADER_INVALID_KEYFILE",
            Self::SecretKeyDecryptFailed(_) => "KEY_LOADER_SECRET_KEY_DECRYPT_FAILED",
            Self::SecretKeyRestoreFailed(_) => "KEY_LOADER_SECRET_KEY_RESTORE_FAILED",
            Self::PublicKeyRestoreFailed(_) => "KEY_LOADER_PUBLIC_KEY_RESTORE_FAILED",
            Self::ActorIdGenerationFailed(_) => "KEY_LOADER_ACTOR_ID_GENERATION_FAILED",
            Self::CertificateBuildFailed(_) => "KEY_LOADER_CERTIFICATE_BUILD_FAILED",
            Self::CertificateSignFailed(_) => "KEY_LOADER_CERTIFICATE_SIGN_FAILED",
            Self::CertificateSerializeFailed(_) => "KEY_LOADER_CERTIFICATE_SERIALIZE_FAILED",
            Self::CertificateParseFailed(_) => "KEY_LOADER_CERTIFICATE_PARSE_FAILED",
            Self::PassportSerializeFailed(_) => "KEY_LOADER_PASSPORT_SERIALIZE_FAILED",
            Self::PassportParseFailed(_) => "KEY_LOADER_PASSPORT_PARSE_FAILED",
            Self::BundleSerializeFailed(_) => "KEY_LOADER_BUNDLE_SERIALIZE_FAILED",
            Self::BundleParseFailed(_) => "KEY_LOADER_BUNDLE_PARSE_FAILED",
            Self::TimeError => "KEY_LOADER_TIME_ERROR",
        }
    }
}

impl fmt::Display for KeyLoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPassword => write!(f, "key loader failed: password must not be empty"),
            Self::InvalidValidityWindow => {
                write!(
                    f,
                    "key loader failed: certificate validity window must be greater than zero"
                )
            }
            Self::FilesystemError(error) => write!(f, "key loader filesystem error: {error}"),
            Self::InvalidKeyfile => {
                write!(f, "key loader failed: stored keyfile is invalid")
            }
            Self::SecretKeyDecryptFailed(error) => {
                write!(f, "key loader failed: secret key decryption failed: {error}")
            }
            Self::SecretKeyRestoreFailed(error) => {
                write!(f, "key loader failed: secret key restoration failed: {error}")
            }
            Self::PublicKeyRestoreFailed(error) => {
                write!(f, "key loader failed: public key restoration failed: {error}")
            }
            Self::ActorIdGenerationFailed(error) => {
                write!(f, "key loader failed: actor id generation failed: {error}")
            }
            Self::CertificateBuildFailed(error) => {
                write!(f, "key loader failed: certificate build failed: {error}")
            }
            Self::CertificateSignFailed(error) => {
                write!(f, "key loader failed: certificate signing failed: {error}")
            }
            Self::CertificateSerializeFailed(error) => {
                write!(f, "key loader failed: certificate serialization failed: {error}")
            }
            Self::CertificateParseFailed(error) => {
                write!(f, "key loader failed: certificate parsing failed: {error}")
            }
            Self::PassportSerializeFailed(error) => {
                write!(f, "key loader failed: passport serialization failed: {error}")
            }
            Self::PassportParseFailed(error) => {
                write!(f, "key loader failed: passport parsing failed: {error}")
            }
            Self::BundleSerializeFailed(error) => {
                write!(f, "key loader failed: key bundle serialization failed: {error}")
            }
            Self::BundleParseFailed(error) => {
                write!(f, "key loader failed: key bundle parsing failed: {error}")
            }
            Self::TimeError => write!(f, "key loader failed: system time is invalid"),
        }
    }
}

impl std::error::Error for KeyLoaderError {}

/// Filesystem-backed key loader and bootstrap utility.
pub struct KeyLoader;

impl KeyLoader {
    /// Loads existing key material or creates it if it does not exist yet.
    pub fn load_or_create(
        paths: &KeyPaths,
        request: &KeyBootstrapRequest,
        ca: &CertificateAuthority,
    ) -> Result<KeyMaterial, KeyLoaderError> {
        if request.password.trim().is_empty() {
            return Err(KeyLoaderError::EmptyPassword);
        }

        if request.certificate_validity_secs == 0 {
            return Err(KeyLoaderError::InvalidValidityWindow);
        }

        fs::create_dir_all(paths.base_dir())
            .map_err(|error| KeyLoaderError::FilesystemError(error.to_string()))?;

        if paths.secret_keyfile_path.exists() {
            return Self::load_existing(paths, request, ca);
        }

        Self::create_new(paths, request, ca)
    }

    fn load_existing(
        paths: &KeyPaths,
        request: &KeyBootstrapRequest,
        ca: &CertificateAuthority,
    ) -> Result<KeyMaterial, KeyLoaderError> {
        let serialized_bundle = fs::read_to_string(&paths.secret_keyfile_path)
            .map_err(|error| KeyLoaderError::FilesystemError(error.to_string()))?;

        let bundle: PersistedKeyBundle = serde_json::from_str(&serialized_bundle)
            .map_err(|error| KeyLoaderError::BundleParseFailed(error.to_string()))?;

        if !is_valid_keyfile(&bundle.encrypted_secret_key) {
            return Err(KeyLoaderError::InvalidKeyfile);
        }

        let secret_key_bytes = decrypt_key(&bundle.encrypted_secret_key, &request.password)
            .map_err(|error| KeyLoaderError::SecretKeyDecryptFailed(error.to_string()))?;

        let secret_key = secret_key_from_bytes(&secret_key_bytes)
            .map_err(KeyLoaderError::SecretKeyRestoreFailed)?;

        let public_key_bytes = hex::decode(&bundle.public_key_hex)
            .map_err(|error| KeyLoaderError::PublicKeyRestoreFailed(error.to_string()))?;

        let public_key = public_key_from_bytes(&public_key_bytes)
            .map_err(KeyLoaderError::PublicKeyRestoreFailed)?;

        let expected_actor_id = generate_and_validate_actor_id(
            &serialize_public_key(&public_key),
            &request.role,
            &request.zone,
        )
        .map_err(|error| KeyLoaderError::ActorIdGenerationFailed(error.to_string()))?;

        if bundle.actor_id != expected_actor_id {
            return Err(KeyLoaderError::ActorIdGenerationFailed(
                "stored actor id does not match derived identity".to_string(),
            ));
        }

        let _ = secret_key;

        let certificate = if path_exists(&paths.certificate_path) {
            Some(Self::load_certificate(&paths.certificate_path)?)
        } else {
            let cert = Self::issue_certificate(
                &bundle.actor_id,
                &request.role,
                &request.zone,
                &bundle.public_key_hex,
                &request.chain,
                request.certificate_validity_secs,
                ca,
            )?;
            Self::save_certificate(&paths.certificate_path, &cert)?;
            Some(cert)
        };

        let passport = match &certificate {
            Some(cert) => {
                if path_exists(&paths.passport_path) {
                    Some(Self::load_passport(&paths.passport_path)?)
                } else {
                    let passport = Self::build_passport(cert)?;
                    Self::save_passport(&paths.passport_path, &passport)?;
                    Some(passport)
                }
            }
            None => None,
        };

        Ok(KeyMaterial {
            actor_id: bundle.actor_id,
            role: bundle.role,
            zone: bundle.zone,
            public_key_hex: bundle.public_key_hex,
            encrypted_secret_key: bundle.encrypted_secret_key,
            certificate,
            passport,
        })
    }

    fn create_new(
        paths: &KeyPaths,
        request: &KeyBootstrapRequest,
        ca: &CertificateAuthority,
    ) -> Result<KeyMaterial, KeyLoaderError> {
        let (public_key, secret_key) = generate_keypair();

        let public_key_bytes = serialize_public_key(&public_key);
        let secret_key_bytes = serialize_secret_key(&secret_key);

        let actor_id = generate_and_validate_actor_id(&public_key_bytes, &request.role, &request.zone)
            .map_err(|error| KeyLoaderError::ActorIdGenerationFailed(error.to_string()))?;

        let public_key_hex = hex::encode_upper(&public_key_bytes);

        let encrypted_secret_key = encrypt_key(&secret_key_bytes, &request.password)
            .map_err(|error| KeyLoaderError::SecretKeyDecryptFailed(error.to_string()))?;

        let certificate = Self::issue_certificate(
            &actor_id,
            &request.role,
            &request.zone,
            &public_key_hex,
            &request.chain,
            request.certificate_validity_secs,
            ca,
        )?;

        let passport = Self::build_passport(&certificate)?;

        let bundle = PersistedKeyBundle {
            actor_id: actor_id.clone(),
            role: request.role.clone(),
            zone: request.zone.clone(),
            public_key_hex: public_key_hex.clone(),
            encrypted_secret_key: encrypted_secret_key.clone(),
        };

        Self::save_bundle(&paths.secret_keyfile_path, &bundle)?;
        Self::save_certificate(&paths.certificate_path, &certificate)?;
        Self::save_passport(&paths.passport_path, &passport)?;

        Ok(KeyMaterial {
            actor_id,
            role: request.role.clone(),
            zone: request.zone.clone(),
            public_key_hex,
            encrypted_secret_key,
            certificate: Some(certificate),
            passport: Some(passport),
        })
    }

    fn issue_certificate(
        actor_id: &str,
        role: &str,
        zone: &str,
        public_key_hex: &str,
        chain: &str,
        validity_secs: u64,
        ca: &CertificateAuthority,
    ) -> Result<Certificate, KeyLoaderError> {
        let issued_at = current_unix_time()?;
        let expires_at = issued_at.saturating_add(validity_secs);

        let unsigned = Certificate::new_unsigned(
            chain.to_string(),
            actor_id.to_string(),
            role.to_string(),
            zone.to_string(),
            public_key_hex.to_string(),
            issued_at,
            expires_at,
        );

        unsigned
            .validate_unsigned()
            .map_err(|error| KeyLoaderError::CertificateBuildFailed(error.to_string()))?;

        ca.sign_certificate(unsigned)
            .map_err(KeyLoaderError::CertificateSignFailed)
    }

    fn build_passport(cert: &Certificate) -> Result<Passport, KeyLoaderError> {
        cert.validate_signed()
            .map_err(|error| KeyLoaderError::CertificateBuildFailed(error.to_string()))?;

        let certificate_json = serde_json::to_string(cert)
            .map_err(|error| KeyLoaderError::CertificateSerializeFailed(error.to_string()))?;

        Ok(Passport::new(
            cert.actor_id.clone(),
            cert.role.clone(),
            cert.zone.clone(),
            certificate_json,
            cert.issued_at,
            cert.expires_at,
        ))
    }

    fn save_bundle(path: &Path, bundle: &PersistedKeyBundle) -> Result<(), KeyLoaderError> {
        let serialized = serde_json::to_string_pretty(bundle)
            .map_err(|error| KeyLoaderError::BundleSerializeFailed(error.to_string()))?;

        fs::write(path, serialized).map_err(|error| KeyLoaderError::FilesystemError(error.to_string()))
    }

    fn save_certificate(path: &Path, certificate: &Certificate) -> Result<(), KeyLoaderError> {
        let serialized = serde_json::to_string_pretty(certificate)
            .map_err(|error| KeyLoaderError::CertificateSerializeFailed(error.to_string()))?;

        fs::write(path, serialized).map_err(|error| KeyLoaderError::FilesystemError(error.to_string()))
    }

    fn load_certificate(path: &Path) -> Result<Certificate, KeyLoaderError> {
        let serialized = fs::read_to_string(path)
            .map_err(|error| KeyLoaderError::FilesystemError(error.to_string()))?;

        let certificate: Certificate = serde_json::from_str(&serialized)
            .map_err(|error| KeyLoaderError::CertificateParseFailed(error.to_string()))?;

        certificate
            .validate_signed()
            .map_err(|error| KeyLoaderError::CertificateBuildFailed(error.to_string()))?;

        Ok(certificate)
    }

    fn save_passport(path: &Path, passport: &Passport) -> Result<(), KeyLoaderError> {
        let serialized = passport
            .to_json()
            .map_err(KeyLoaderError::PassportSerializeFailed)?;

        fs::write(path, serialized).map_err(|error| KeyLoaderError::FilesystemError(error.to_string()))
    }

    fn load_passport(path: &Path) -> Result<Passport, KeyLoaderError> {
        let serialized = fs::read_to_string(path)
            .map_err(|error| KeyLoaderError::FilesystemError(error.to_string()))?;

        Passport::from_json(&serialized).map_err(KeyLoaderError::PassportParseFailed)
    }
}

fn path_exists(path: &Path) -> bool {
    path.exists() && path.is_file()
}

fn current_unix_time() -> Result<u64, KeyLoaderError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| KeyLoaderError::TimeError)
}

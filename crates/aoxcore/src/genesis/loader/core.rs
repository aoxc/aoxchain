// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! AOXC genesis loader.
//!
//! This module is responsible for loading, validating, persisting, and
//! constructing AOXC-native genesis artifacts.
//!
//! Design objectives:
//! - network-agnostic loader behavior,
//! - strict file-boundary validation,
//! - atomic persistence,
//! - reproducible artifact writing,
//! - future compatibility with signed genesis envelopes,
//! - explicit separation between parse, validation, and persistence failures.
//!
//! Security rationale:
//! The loader is the file-system boundary for genesis artifacts. It must reject
//! malformed, oversized, empty, or structurally invalid inputs before they
//! become trusted configuration objects.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::genesis::config::{
    AOXC_FAMILY_ID, AOXCANDSeal, ChainIdentity, GenesisAccount, GenesisConfig, GenesisConfigError,
    NetworkClass, SettlementLink, Validator,
};

/// Canonical treasury account identifier used by default genesis builders.
///
/// Governance note:
/// This identifier is AOXC-native and should remain stable unless the treasury
/// identity policy is explicitly migrated.
pub const TREASURY_ACCOUNT: &str = "AOXC_TREASURY_GENESIS";

/// Default treasury allocation used by default genesis builders.
pub(crate) const DEFAULT_TREASURY: u128 = 1_000_000_000;

/// Default target block time in milliseconds.
const DEFAULT_BLOCK_TIME_MS: u64 = 3_000;

/// Maximum accepted genesis file size in bytes.
///
/// Security rationale:
/// This is a defensive upper bound intended to reject obviously malformed or
/// hostile input sizes while remaining operationally sufficient for normal
/// genesis artifacts.
const MAX_GENESIS_FILE_SIZE_BYTES: u64 = 4 * 1024 * 1024;

/// Canonical extension for fingerprint sidecar files.
const FINGERPRINT_EXTENSION: &str = "fingerprint";

/// Canonical extension for future detached signature sidecar files.
const SIGNATURE_EXTENSION: &str = "sig";

/// Errors produced during genesis loading and persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenesisError {
    ReadError(String),
    ParseError(String),
    ValidationError(String),
    WriteError(String),
    IntegrityError(String),
}

impl std::fmt::Display for GenesisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadError(error) => write!(f, "GENESIS_READ_ERROR: {error}"),
            Self::ParseError(error) => write!(f, "GENESIS_PARSE_ERROR: {error}"),
            Self::ValidationError(error) => write!(f, "GENESIS_VALIDATION_ERROR: {error}"),
            Self::WriteError(error) => write!(f, "GENESIS_WRITE_ERROR: {error}"),
            Self::IntegrityError(error) => write!(f, "GENESIS_INTEGRITY_ERROR: {error}"),
        }
    }
}

impl std::error::Error for GenesisError {}

impl From<GenesisConfigError> for GenesisError {
    fn from(error: GenesisConfigError) -> Self {
        Self::ValidationError(error.to_string())
    }
}

/// Result of loading a genesis artifact together with its derived fingerprint
/// and optional sidecar metadata.
///
/// Operational rationale:
/// This structure allows callers to observe both the validated config and the
/// loader-side artifact integrity context without re-reading files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedGenesisArtifact {
    pub config: GenesisConfig,
    pub fingerprint_hex: String,
    pub fingerprint_sidecar_present: bool,
    pub detached_signature_sidecar_present: bool,
}

/// Responsible for loading, validating, persisting, and constructing
/// AOXC-native genesis configuration artifacts.
pub struct GenesisLoader;

impl GenesisLoader {
    /// Loads genesis configuration from disk.
    ///
    /// Security properties:
    /// - rejects non-file paths,
    /// - rejects empty or oversized input,
    /// - parses JSON into a typed model,
    /// - validates the parsed configuration before returning it.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<GenesisConfig, GenesisError> {
        Ok(Self::load_artifact(path)?.config)
    }

    /// Loads genesis configuration together with loader-side artifact metadata.
    ///
    /// Integrity properties:
    /// - derives the canonical fingerprint after validation,
    /// - optionally verifies a fingerprint sidecar if present,
    /// - reports detached signature sidecar presence for future verification flows.
    pub fn load_artifact<P: AsRef<Path>>(path: P) -> Result<LoadedGenesisArtifact, GenesisError> {
        let path = path.as_ref();
        validate_input_path(path)?;

        let data =
            fs::read_to_string(path).map_err(|error| GenesisError::ReadError(error.to_string()))?;

        if data.trim().is_empty() {
            return Err(GenesisError::ParseError(format!(
                "genesis file contains no usable JSON content: {}",
                path.display()
            )));
        }

        let config: GenesisConfig = serde_json::from_str(&data)
            .map_err(|error| GenesisError::ParseError(error.to_string()))?;

        config.validate()?;
        let fingerprint_hex = config.fingerprint()?;

        let fingerprint_path = fingerprint_sidecar_path(path);
        let signature_path = signature_sidecar_path(path);

        let fingerprint_sidecar_present = fingerprint_path.is_file();
        let detached_signature_sidecar_present = signature_path.is_file();

        if fingerprint_sidecar_present {
            verify_fingerprint_sidecar(&fingerprint_path, &fingerprint_hex)?;
        }

        Ok(LoadedGenesisArtifact {
            config,
            fingerprint_hex,
            fingerprint_sidecar_present,
            detached_signature_sidecar_present,
        })
    }

    /// Persists genesis configuration to disk.
    ///
    /// Persistence properties:
    /// - validates the config before writing,
    /// - writes canonical pretty JSON,
    /// - uses a temporary file + atomic rename pattern,
    /// - synchronizes file contents before rename,
    /// - synchronizes the parent directory afterward,
    /// - emits a fingerprint sidecar for operator and audit workflows.
    pub fn save<P: AsRef<Path>>(genesis: &GenesisConfig, path: P) -> Result<(), GenesisError> {
        genesis.validate()?;

        let path = path.as_ref();
        ensure_parent_exists(path)?;

        let json = serde_json::to_vec_pretty(genesis)
            .map_err(|error| GenesisError::WriteError(error.to_string()))?;

        let fingerprint = genesis.fingerprint()?;
        let temp_path = temporary_path(path);

        write_atomic(&temp_path, path, &json)
            .map_err(|error| GenesisError::WriteError(error.to_string()))?;

        let fingerprint_path = fingerprint_sidecar_path(path);
        let fingerprint_temp = temporary_path(&fingerprint_path);

        write_atomic(&fingerprint_temp, &fingerprint_path, fingerprint.as_bytes())
            .map_err(|error| GenesisError::WriteError(error.to_string()))?;

        Ok(())
    }

    /// Constructs the canonical AOXC public mainnet genesis configuration.
    pub fn load_default() -> Result<GenesisConfig, GenesisError> {
        Self::build_named_default(NetworkClass::PublicMainnet)
    }

    /// Constructs the canonical AOXC public testnet genesis configuration.
    pub fn load_default_testnet() -> Result<GenesisConfig, GenesisError> {
        Self::build_named_default(NetworkClass::PublicTestnet)
    }

    /// Constructs the canonical AOXC validation genesis configuration.
    pub fn load_default_validation() -> Result<GenesisConfig, GenesisError> {
        Self::build_named_default(NetworkClass::Validation)
    }

    /// Constructs the canonical AOXC devnet genesis configuration.
    pub fn load_default_devnet() -> Result<GenesisConfig, GenesisError> {
        Self::build_named_default(NetworkClass::Devnet)
    }

    /// Loads genesis from disk, or creates and persists the canonical AOXC
    /// default mainnet genesis when the target path does not yet exist.
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<GenesisConfig, GenesisError> {
        let path_ref = path.as_ref();

        match fs::metadata(path_ref) {
            Ok(metadata) => {
                if !metadata.is_file() {
                    return Err(GenesisError::ReadError(format!(
                        "path exists but is not a regular file: {}",
                        path_ref.display()
                    )));
                }

                Self::load(path_ref)
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let genesis = Self::load_default()?;
                Self::save(&genesis, path_ref)?;
                Ok(genesis)
            }
            Err(error) => Err(GenesisError::ReadError(error.to_string())),
        }
    }

    /// Resolves a path into an owned `PathBuf`.
    #[must_use]
    pub fn resolve_path<P: AsRef<Path>>(path: P) -> PathBuf {
        path.as_ref().to_path_buf()
    }

    /// Returns the expected fingerprint sidecar path for the given genesis path.
    #[must_use]
    pub fn resolve_fingerprint_sidecar_path<P: AsRef<Path>>(path: P) -> PathBuf {
        fingerprint_sidecar_path(path.as_ref())
    }

    /// Returns the expected detached signature sidecar path for the given genesis path.
    #[must_use]
    pub fn resolve_signature_sidecar_path<P: AsRef<Path>>(path: P) -> PathBuf {
        signature_sidecar_path(path.as_ref())
    }

    /// Constructs a policy-compliant AOXC-native default genesis configuration
    /// for the requested network class.
    fn build_named_default(network_class: NetworkClass) -> Result<GenesisConfig, GenesisError> {
        let (governance_serial_ordinal, class_instance_ordinal, chain_name, validator_id, seal_id) =
            match network_class {
                NetworkClass::PublicMainnet => (
                    1,
                    1,
                    "AOXC AKDENIZ",
                    "aoxc-validator-mainnet-001",
                    "aoxc-seal-mainnet-001",
                ),
                NetworkClass::PublicTestnet => (
                    2,
                    1,
                    "AOXC Pusula",
                    "aoxc-validator-testnet-001",
                    "aoxc-seal-testnet-001",
                ),
                NetworkClass::Validation => (
                    4,
                    1,
                    "AOXC Mizan",
                    "aoxc-validator-validation-001",
                    "aoxc-seal-validation-001",
                ),
                NetworkClass::Devnet => (
                    3,
                    1,
                    "AOXC Kivilcim",
                    "aoxc-validator-devnet-001",
                    "aoxc-seal-devnet-001",
                ),
                NetworkClass::SovereignPrivate => (
                    101,
                    1,
                    "AOXC Sovereign Private 001",
                    "aoxc-validator-sovereign-001",
                    "aoxc-seal-sovereign-001",
                ),
                NetworkClass::Consortium => (
                    201,
                    1,
                    "AOXC Consortium 001",
                    "aoxc-validator-consortium-001",
                    "aoxc-seal-consortium-001",
                ),
                NetworkClass::RegulatedPrivate => (
                    301,
                    1,
                    "AOXC Regulated Private 001",
                    "aoxc-validator-regulated-001",
                    "aoxc-seal-regulated-001",
                ),
            };

        let identity = ChainIdentity::new(
            AOXC_FAMILY_ID,
            network_class,
            governance_serial_ordinal,
            class_instance_ordinal,
            chain_name,
        )?;

        let treasury_account = GenesisAccount {
            address: TREASURY_ACCOUNT.to_string(),
            balance: DEFAULT_TREASURY,
        };

        let config = GenesisConfig::new(
            identity,
            DEFAULT_BLOCK_TIME_MS,
            vec![Validator {
                id: validator_id.to_string(),
            }],
            vec![treasury_account],
            DEFAULT_TREASURY,
            SettlementLink {
                endpoint: "aoxc://settlement/root".to_string(),
            },
            AOXCANDSeal {
                seal_id: seal_id.to_string(),
            },
        )?;

        Ok(config)
    }

    /// Placeholder hook for future detached-signature verification.
    ///
    /// Future rationale:
    /// Genesis artifacts are expected to evolve toward signed distribution
    /// workflows. This function intentionally exists as a stable upgrade point.
    pub fn verify_detached_signature_sidecar<P: AsRef<Path>>(path: P) -> Result<(), GenesisError> {
        let signature_path = signature_sidecar_path(path.as_ref());

        if !signature_path.exists() {
            return Ok(());
        }

        Err(GenesisError::IntegrityError(format!(
            "detached signature sidecar is present but signature verification is not yet implemented: {}",
            signature_path.display()
        )))
    }
}

/// Validates the input genesis path before attempting a read.
fn validate_input_path(path: &Path) -> Result<(), GenesisError> {
    let metadata =
        fs::metadata(path).map_err(|error| GenesisError::ReadError(error.to_string()))?;

    if !metadata.is_file() {
        return Err(GenesisError::ReadError(format!(
            "path is not a regular file: {}",
            path.display()
        )));
    }

    if metadata.len() == 0 {
        return Err(GenesisError::ParseError(format!(
            "genesis file is empty: {}",
            path.display()
        )));
    }

    if metadata.len() > MAX_GENESIS_FILE_SIZE_BYTES {
        return Err(GenesisError::ReadError(format!(
            "genesis file exceeds size limit ({} bytes): {}",
            MAX_GENESIS_FILE_SIZE_BYTES,
            path.display()
        )));
    }

    Ok(())
}

/// Ensures the parent directory of the target path exists.
fn ensure_parent_exists(path: &Path) -> Result<(), GenesisError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| GenesisError::WriteError(error.to_string()))?;
    }

    Ok(())
}

/// Verifies the content of a fingerprint sidecar against the derived fingerprint.
fn verify_fingerprint_sidecar(
    fingerprint_path: &Path,
    expected_fingerprint_hex: &str,
) -> Result<(), GenesisError> {
    let stored = fs::read_to_string(fingerprint_path)
        .map_err(|error| GenesisError::ReadError(error.to_string()))?;

    let normalized = stored.trim();

    if normalized.is_empty() {
        return Err(GenesisError::IntegrityError(format!(
            "fingerprint sidecar is empty: {}",
            fingerprint_path.display()
        )));
    }

    if normalized != expected_fingerprint_hex {
        return Err(GenesisError::IntegrityError(format!(
            "fingerprint sidecar mismatch: expected {}, found {}",
            expected_fingerprint_hex, normalized
        )));
    }

    Ok(())
}

/// Builds a deterministic temporary path adjacent to the target file.
fn temporary_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("artifact");

    let temp_name = format!(".{}.tmp", file_name);

    match path.parent() {
        Some(parent) => parent.join(temp_name),
        None => PathBuf::from(temp_name),
    }
}

/// Returns the fingerprint sidecar path for a genesis file.
fn fingerprint_sidecar_path(path: &Path) -> PathBuf {
    with_appended_extension(path, FINGERPRINT_EXTENSION)
}

/// Returns the detached signature sidecar path for a genesis file.
fn signature_sidecar_path(path: &Path) -> PathBuf {
    with_appended_extension(path, SIGNATURE_EXTENSION)
}

/// Appends a secondary extension to a path.
///
/// Example:
/// `genesis.json` + `fingerprint` => `genesis.json.fingerprint`
fn with_appended_extension(path: &Path, extension_suffix: &str) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("artifact");

    let appended = format!("{file_name}.{extension_suffix}");

    match path.parent() {
        Some(parent) => parent.join(appended),
        None => PathBuf::from(appended),
    }
}

/// Writes bytes to a temporary file, flushes and syncs them, then renames
/// the file into the final destination.
///
/// This reduces the risk of partially written artifacts on interruption.
fn write_atomic(temp_path: &Path, final_path: &Path, content: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(temp_path)?;

    file.write_all(content)?;
    file.flush()?;
    file.sync_all()?;
    drop(file);

    fs::rename(temp_path, final_path)?;
    sync_parent_directory(final_path)?;

    Ok(())
}

/// Synchronizes the parent directory after an atomic rename when available.
fn sync_parent_directory(path: &Path) -> io::Result<()> {
    let parent = match path.parent() {
        Some(parent) => parent,
        None => return Ok(()),
    };

    let dir = OpenOptions::new().read(true).open(parent)?;
    dir.sync_all()
}

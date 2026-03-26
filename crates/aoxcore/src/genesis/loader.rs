//! AOXC genesis loader.
//!
//! This module is responsible for loading, validating, persisting, and
//! constructing genesis artifacts for AOXC-native networks.
//!
//! The implementation is intentionally aligned with the AOXC forward-compatible
//! identity model where:
//! - the binary remains network-agnostic,
//! - network identity is derived from genesis configuration,
//! - public, test, validation, and private deployments share the same loader,
//! - hard-coded third-party settlement network references are prohibited.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::config::{
    AOXC_FAMILY_ID, AOXCANDSeal, ChainIdentity, GenesisAccount, GenesisConfig, GenesisConfigError,
    NetworkClass, SettlementLink, Validator,
};

/// Canonical treasury account identifier used by the default genesis builder.
///
/// This identifier is AOXC-native and must remain stable unless the treasury
/// naming policy is formally migrated.
pub const TREASURY_ACCOUNT: &str = "AOXC_TREASURY_GENESIS";

/// Default treasury allocation used by the default genesis builders.
const DEFAULT_TREASURY: u128 = 1_000_000_000;

/// Default target block time in milliseconds.
const DEFAULT_BLOCK_TIME_MS: u64 = 3_000;

/// Maximum accepted genesis file size in bytes.
///
/// This is a defensive bound intended to reject obviously malformed or hostile
/// input sizes while remaining operationally generous for normal genesis files.
const MAX_GENESIS_FILE_SIZE_BYTES: u64 = 4 * 1024 * 1024;

/// Errors produced during genesis loading and persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenesisError {
    ReadError(String),
    ParseError(String),
    ValidationError(String),
    WriteError(String),
}

impl std::fmt::Display for GenesisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadError(error) => write!(f, "GENESIS_READ_ERROR: {error}"),
            Self::ParseError(error) => write!(f, "GENESIS_PARSE_ERROR: {error}"),
            Self::ValidationError(error) => write!(f, "GENESIS_VALIDATION_ERROR: {error}"),
            Self::WriteError(error) => write!(f, "GENESIS_WRITE_ERROR: {error}"),
        }
    }
}

impl std::error::Error for GenesisError {}

impl From<GenesisConfigError> for GenesisError {
    fn from(error: GenesisConfigError) -> Self {
        Self::ValidationError(error.to_string())
    }
}

/// Responsible for loading, validating, persisting, and constructing
/// AOXC-native genesis configuration artifacts.
pub struct GenesisLoader;

impl GenesisLoader {
    /// Loads genesis configuration from disk.
    ///
    /// Security properties:
    /// - rejects non-file paths;
    /// - rejects empty or oversized genesis input;
    /// - validates the parsed configuration before returning it.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<GenesisConfig, GenesisError> {
        let path = path.as_ref();

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
        Ok(config)
    }

    /// Persists genesis configuration to disk.
    ///
    /// Security properties:
    /// - ensures the parent directory exists;
    /// - writes to a temporary file first;
    /// - flushes and synchronizes file contents before rename;
    /// - renames atomically into the destination path on supported filesystems.
    pub fn save<P: AsRef<Path>>(genesis: &GenesisConfig, path: P) -> Result<(), GenesisError> {
        genesis.validate()?;

        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| GenesisError::WriteError(error.to_string()))?;
        }

        let json = serde_json::to_vec_pretty(genesis)
            .map_err(|error| GenesisError::WriteError(error.to_string()))?;

        let temp_path = temporary_path(path);

        write_atomic(&temp_path, path, &json)
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
}

/// Builds a deterministic temporary path adjacent to the target genesis file.
fn temporary_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("genesis.json");

    let temp_name = format!(".{}.tmp", file_name);

    match path.parent() {
        Some(parent) => parent.join(temp_name),
        None => PathBuf::from(temp_name),
    }
}

/// Writes bytes to a temporary file, flushes and syncs them, then renames
/// the file into the final destination.
///
/// This reduces the risk of partially written genesis files on interruption.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_default_builds_aoxc_mainnet_identity() {
        let config = GenesisLoader::load_default().expect("default mainnet genesis must build");

        assert_eq!(config.identity.family_id, AOXC_FAMILY_ID);
        assert_eq!(config.identity.chain_id, 2626000001);
        assert_eq!(config.identity.network_serial, "2626-001");
        assert_eq!(config.identity.network_id, "aoxc-mainnet-2626-001");
        assert_eq!(config.identity.network_class, NetworkClass::PublicMainnet);
        assert_eq!(config.treasury, DEFAULT_TREASURY);
        assert_eq!(config.accounts.len(), 1);
        assert_eq!(config.accounts[0].address, TREASURY_ACCOUNT);
        assert_eq!(config.accounts[0].balance, DEFAULT_TREASURY);
    }

    #[test]
    fn load_default_testnet_builds_aoxc_testnet_identity() {
        let config =
            GenesisLoader::load_default_testnet().expect("default testnet genesis must build");

        assert_eq!(config.identity.family_id, AOXC_FAMILY_ID);
        assert_eq!(config.identity.chain_id, 2626010001);
        assert_eq!(config.identity.network_serial, "2626-002");
        assert_eq!(config.identity.network_id, "aoxc-testnet-2626-002");
        assert_eq!(config.identity.network_class, NetworkClass::PublicTestnet);
    }

    #[test]
    fn load_returns_validation_error_for_invalid_genesis_file() {
        let temp_dir =
            std::env::temp_dir().join(format!("aoxc-genesis-loader-test-{}", std::process::id()));
        fs::create_dir_all(&temp_dir).expect("temp dir must be created");

        let path = temp_dir.join("genesis.json");

        fs::write(
            &path,
            r#"{
                "identity": {
                    "family_id": 2626,
                    "chain_id": 1,
                    "network_serial": "2626-001",
                    "network_id": "aoxc-mainnet-2626-001",
                    "chain_name": "AOXC AKDENIZ",
                    "network_class": "public-mainnet"
                },
                "block_time": 0,
                "validators": [],
                "accounts": [],
                "treasury": 0,
                "settlement_link": { "endpoint": "aoxc://settlement/root" },
                "genesis_seal": { "seal_id": "aoxc-seal-mainnet-001" }
            }"#,
        )
        .expect("invalid genesis fixture must write");

        let err = GenesisLoader::load(&path).expect_err("invalid genesis must be rejected");
        assert!(matches!(err, GenesisError::ValidationError(_)));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&temp_dir);
    }
}

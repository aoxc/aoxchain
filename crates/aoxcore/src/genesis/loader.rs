use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::config::{GenesisBlock, GenesisConfig, TREASURY_ACCOUNT};

/// Default treasury allocation.
const DEFAULT_TREASURY: u128 = 1_000_000_000;

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
    ConstructionError(String),
    WriteError(String),
}

impl std::fmt::Display for GenesisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadError(error) => write!(f, "GENESIS_READ_ERROR: {error}"),
            Self::ParseError(error) => write!(f, "GENESIS_PARSE_ERROR: {error}"),
            Self::ValidationError(error) => write!(f, "GENESIS_VALIDATION_ERROR: {error}"),
            Self::ConstructionError(error) => write!(f, "GENESIS_CONSTRUCTION_ERROR: {error}"),
            Self::WriteError(error) => write!(f, "GENESIS_WRITE_ERROR: {error}"),
        }
    }
}

impl std::error::Error for GenesisError {}

/// Responsible for loading, validating, persisting, and constructing
/// genesis artifacts from configuration sources.
pub struct GenesisLoader;

impl GenesisLoader {
    /// Loads genesis configuration from disk and constructs a `GenesisBlock`.
    ///
    /// Security properties:
    /// - rejects non-file paths;
    /// - rejects empty or oversized genesis input;
    /// - validates the parsed configuration before constructing the block.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<GenesisBlock, GenesisError> {
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

        config.validate().map_err(GenesisError::ValidationError)?;

        GenesisBlock::try_new(config).map_err(GenesisError::ConstructionError)
    }

    /// Persists genesis configuration to disk.
    ///
    /// Security properties:
    /// - ensures parent directory exists;
    /// - writes to a temporary file first;
    /// - flushes and synchronizes file contents before rename;
    /// - renames atomically into the destination path on supported filesystems.
    pub fn save<P: AsRef<Path>>(genesis: &GenesisConfig, path: P) -> Result<(), GenesisError> {
        genesis.validate().map_err(GenesisError::ValidationError)?;

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

    /// Constructs a default genesis block.
    ///
    /// Compatibility behavior is preserved:
    /// - treasury is funded;
    /// - the canonical treasury account is inserted into the genesis accounts list.
    #[must_use]
    pub fn load_default() -> GenesisBlock {
        let mut config = GenesisConfig::new();

        config.treasury = DEFAULT_TREASURY;
        config.add_account(TREASURY_ACCOUNT.to_string(), DEFAULT_TREASURY);

        GenesisBlock::try_new(config)
            .expect("GENESIS_DEFAULT: default genesis configuration must remain valid")
    }

    /// Loads genesis from disk, or creates and persists a default genesis
    /// when the target path does not yet exist.
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Result<GenesisBlock, GenesisError> {
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
                let genesis = Self::load_default();
                Self::save(&genesis.config, path_ref)?;
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
    fn load_returns_validation_error_for_invalid_genesis_file() {
        let temp_dir =
            std::env::temp_dir().join(format!("aoxc-genesis-loader-test-{}", std::process::id()));
        fs::create_dir_all(&temp_dir).expect("temp dir must be created");
        let path = temp_dir.join("genesis.json");
        fs::write(&path, r#"{"chain_id":"","chain_num":1001,"accounts":[],"validators":[],"treasury":0,"block_time":0,"genesis_seal":{"node_sig":null,"ai_sig":null,"dao_sig":null},"settlement_link":{"native_symbol":"AOXC","native_decimals":18,"settlement_network":"xlayer-mainnet","settlement_token_address":"0x1111111111111111111111111111111111111111","settlement_main_contract":"0x2222222222222222222222222222222222222222","settlement_multisig_contract":"0x3333333333333333333333333333333333333333","equivalence_mode":"1:1"}}"#).expect("invalid genesis fixture must write");

        let err = GenesisLoader::load(&path).expect_err("invalid genesis must be rejected");
        assert!(matches!(err, GenesisError::ValidationError(_)));

        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(&temp_dir);
    }
}

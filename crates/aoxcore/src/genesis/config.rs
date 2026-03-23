use aoxcunity::Validator;

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::convert::TryFrom;

use std::collections::HashSet;

/// Default main chain numeric identifier.
const DEFAULT_CHAIN_NUM: u32 = 1;

/// Default block time in seconds.
const DEFAULT_BLOCK_TIME: u64 = 6;

/// Minimum accepted block time in seconds.
const MIN_BLOCK_TIME: u64 = 1;

/// Maximum accepted block time in seconds.
///
/// This upper bound prevents obviously unsafe or nonsensical bootstrap values
/// while remaining operationally flexible for development and test environments.
const MAX_BLOCK_TIME: u64 = 600;

/// AOXC namespace prefix used for deterministic chain identifier generation.
const AOXC_PREFIX: &str = "AOXC";

/// Canonical treasury account identifier used by default bootstrap helpers.
pub const TREASURY_ACCOUNT: &str = "AOXC_TREASURY";

/// Maximum accepted account address length for genesis records.
const MAX_ADDRESS_LEN: usize = 128;

/// Maximum accepted validator identifier length in bytes.
const MAX_VALIDATOR_ID_LEN: usize = 32;

/// Maximum accepted metadata string length.
const MAX_METADATA_LEN: usize = 256;

/// Runtime-only genesis seal retained for compatibility with the current
/// bootstrap flow.
///
/// This structure is intentionally local to `aoxcore` because the new
/// `aoxcunity` surface no longer exposes the prior AOXCAND seal type.
#[derive(Clone, Default, Debug)]
pub struct AOXCANDSeal {
    pub node_sig: Option<Vec<u8>>,
    pub ai_sig: Option<Vec<u8>>,
    pub dao_sig: Option<Vec<u8>>,
}

impl AOXCANDSeal {
    /// Creates an empty runtime seal container.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true when all seal segments are present.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.node_sig.is_some() && self.ai_sig.is_some() && self.dao_sig.is_some()
    }
}

/// Represents an account initialized at genesis.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GenesisAccount {
    pub address: String,
    pub balance: u128,
}

/// Cross-layer metadata used to bind AOXC native economics to X Layer assets.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct SettlementLink {
    /// Native chain currency symbol.
    pub native_symbol: String,
    /// Native currency decimals.
    pub native_decimals: u8,
    /// Settlement network identifier.
    pub settlement_network: String,
    /// Bridged token address on settlement network (EVM hex string).
    pub settlement_token_address: String,
    /// Main coordination contract address on settlement network.
    pub settlement_main_contract: String,
    /// Multisig governance contract address on settlement network.
    pub settlement_multisig_contract: String,
    /// Canonical relationship mode (for now `1:1`).
    pub equivalence_mode: String,
}

impl Default for SettlementLink {
    fn default() -> Self {
        Self {
            native_symbol: "AOXC".to_string(),
            native_decimals: 18,
            settlement_network: "xlayer".to_string(),
            settlement_token_address: "0xeb9580c3946bb47d73aae1d4f7a94148b554b2f4".to_string(),
            settlement_main_contract: "0x97bdd1fd1caf756e00efd42eba9406821465b365".to_string(),
            settlement_multisig_contract: "0x20c0dd8b6559912acfac2ce061b8d5b19db8ca84".to_string(),
            equivalence_mode: "1:1".to_string(),
        }
    }
}

/// Genesis configuration used to bootstrap the chain.
///
/// Compatibility note:
/// - `validators` and `genesis_seal` remain runtime-only and are intentionally
///   excluded from serialized genesis artifacts to preserve the current system
///   contract and loader behavior.
/// - `#[serde(skip, default)]` ensures safe deserialization even when these
///   fields are not present in `genesis.json`.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GenesisConfig {
    /// Numeric chain identifier.
    pub chain_num: u32,

    /// Human-readable chain identifier.
    pub chain_id: String,

    /// Block time in seconds.
    pub block_time: u64,

    /// Validator set injected at runtime.
    #[serde(skip, default)]
    pub validators: Vec<Validator>,

    /// Genesis accounts.
    pub accounts: Vec<GenesisAccount>,

    /// Treasury allocation.
    ///
    /// Compatibility note:
    /// The current model preserves this standalone treasury field because the
    /// rest of the system already depends on it.
    pub treasury: u128,

    /// Cross-layer settlement binding metadata.
    #[serde(default)]
    pub settlement_link: SettlementLink,

    /// Runtime seal, excluded from persistent serialization.
    #[serde(skip, default = "default_genesis_seal")]
    pub genesis_seal: AOXCANDSeal,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl GenesisConfig {
    /// Creates the default AOXC genesis configuration.
    #[must_use]
    pub fn new() -> Self {
        let chain_id = Self::generate_chain_id(DEFAULT_CHAIN_NUM);

        Self {
            chain_num: DEFAULT_CHAIN_NUM,
            chain_id,
            block_time: DEFAULT_BLOCK_TIME,
            validators: Vec::with_capacity(16),
            accounts: Vec::with_capacity(64),
            treasury: 0,
            settlement_link: SettlementLink::default(),
            genesis_seal: AOXCANDSeal::new(),
        }
    }

    /// Generates a deterministic chain identifier from the numeric chain number.
    #[must_use]
    pub fn generate_chain_id(num: u32) -> String {
        format!("{}-{:04}-MAIN", AOXC_PREFIX, num)
    }

    /// Adds a validator if the validator identifier is unique.
    ///
    /// Runtime-only validators remain outside the serialized genesis payload,
    /// but duplicate validator identifiers are still rejected in-memory.
    pub fn add_validator(&mut self, validator: Validator) {
        if self
            .validators
            .iter()
            .any(|existing| existing.id == validator.id)
        {
            return;
        }

        self.validators.push(validator);
    }

    /// Adds a genesis account if the address is unique.
    ///
    /// This helper preserves the current void-return contract and performs
    /// duplicate suppression rather than failing loudly.
    pub fn add_account(&mut self, address: String, balance: u128) {
        if self
            .accounts
            .iter()
            .any(|account| account.address == address)
        {
            return;
        }

        self.accounts.push(GenesisAccount { address, balance });
    }

    /// Calculates the total supply defined in genesis.
    ///
    /// Compatibility behavior:
    /// - The method includes the standalone treasury allocation.
    /// - If the canonical treasury account is also present in `accounts`,
    ///   its balance is excluded from the account loop to avoid double-counting.
    #[must_use]
    pub fn total_supply(&self) -> u128 {
        let mut total = self.treasury;

        for account in &self.accounts {
            if account.address == TREASURY_ACCOUNT {
                continue;
            }

            total = total.saturating_add(account.balance);
        }

        total
    }

    /// Validates structural integrity of the genesis configuration.
    ///
    /// Validation scope intentionally remains compatible with the current model:
    /// - runtime-only validator and seal fields are allowed but not required
    ///   for serialized genesis integrity;
    /// - treasury remains a standalone field;
    /// - chain id must match the deterministic chain number derivation.
    pub fn validate(&self) -> Result<(), String> {
        if self.chain_num == 0 {
            return Err("GENESIS: invalid chain number".into());
        }

        let expected_chain_id = Self::generate_chain_id(self.chain_num);
        if self.chain_id != expected_chain_id {
            return Err(format!(
                "GENESIS: chain id mismatch, expected {} got {}",
                expected_chain_id, self.chain_id
            ));
        }

        if !(MIN_BLOCK_TIME..=MAX_BLOCK_TIME).contains(&self.block_time) {
            return Err(format!(
                "GENESIS: block time must be within {}..={} seconds",
                MIN_BLOCK_TIME, MAX_BLOCK_TIME
            ));
        }

        let mut seen_accounts: HashSet<&str> = HashSet::with_capacity(self.accounts.len());

        for account in &self.accounts {
            if !is_valid_address(&account.address) {
                return Err(format!(
                    "GENESIS: invalid account address {}",
                    account.address
                ));
            }

            if !seen_accounts.insert(account.address.as_str()) {
                return Err(format!("GENESIS: duplicate account {}", account.address));
            }

            if account.balance == 0 {
                return Err(format!("GENESIS: zero balance account {}", account.address));
            }
        }

        let mut seen_validators: HashSet<[u8; 32]> = HashSet::with_capacity(self.validators.len());

        for validator in &self.validators {
            if validator.id == [0u8; 32] {
                return Err("GENESIS: validator id missing".into());
            }

            if validator.id.len() > MAX_VALIDATOR_ID_LEN {
                return Err(format!(
                    "GENESIS: validator id too long {:02x?}",
                    validator.id
                ));
            }

            if !seen_validators.insert(validator.id) {
                return Err(format!(
                    "GENESIS: duplicate validator {:02x?}",
                    validator.id
                ));
            }
        }

        validate_settlement_link(&self.settlement_link)?;

        Ok(())
    }

    /// Computes the deterministic genesis state hash.
    ///
    /// Compatibility design:
    /// - runtime-only fields (`validators`, `genesis_seal`) remain excluded;
    /// - accounts are canonically ordered by address before hashing;
    /// - the serialized hash input is stable across insertion-order differences.
    pub fn try_state_hash(&self) -> Result<String, String> {
        #[derive(Serialize)]
        struct CanonicalGenesisConfig<'a> {
            chain_num: u32,
            chain_id: &'a str,
            block_time: u64,
            accounts: Vec<&'a GenesisAccount>,
            treasury: u128,
            settlement_link: &'a SettlementLink,
        }

        let mut accounts: Vec<&GenesisAccount> = self.accounts.iter().collect();
        accounts.sort_by(|left, right| left.address.cmp(&right.address));

        let canonical = CanonicalGenesisConfig {
            chain_num: self.chain_num,
            chain_id: &self.chain_id,
            block_time: self.block_time,
            accounts,
            treasury: self.treasury,
            settlement_link: &self.settlement_link,
        };

        let encoded = serde_json::to_vec(&canonical)
            .map_err(|error| format!("GENESIS_HASH: canonical serialization failure: {error}"))?;

        let mut hasher = Sha3_256::new();
        hasher.update(encoded);

        let digest = hasher.finalize();
        Ok(hex::encode(digest))
    }

    /// Computes the deterministic genesis state hash.
    ///
    /// This compatibility wrapper preserves the historic infallible API for
    /// callers that have already validated the configuration.
    #[must_use]
    pub fn state_hash(&self) -> String {
        self.try_state_hash()
            .expect("GENESIS_HASH: validated genesis config must serialize canonically")
    }

    /// Computes a lightweight deterministic fingerprint for quick identity checks.
    ///
    /// The fingerprint follows the same canonical account ordering strategy used
    /// by `state_hash()` to avoid insertion-order drift.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha3_256::new();

        hasher.update(self.chain_id.as_bytes());
        hasher.update(self.chain_num.to_be_bytes());
        hasher.update(self.block_time.to_be_bytes());
        hasher.update(self.treasury.to_be_bytes());
        hasher.update(self.settlement_link.native_symbol.as_bytes());
        hasher.update([self.settlement_link.native_decimals]);
        hasher.update(self.settlement_link.settlement_network.as_bytes());
        hasher.update(self.settlement_link.settlement_token_address.as_bytes());
        hasher.update(self.settlement_link.settlement_main_contract.as_bytes());
        hasher.update(self.settlement_link.settlement_multisig_contract.as_bytes());
        hasher.update(self.settlement_link.equivalence_mode.as_bytes());

        let mut accounts: Vec<&GenesisAccount> = self.accounts.iter().collect();
        accounts.sort_by(|left, right| left.address.cmp(&right.address));

        for account in accounts {
            hasher.update(account.address.as_bytes());
            hasher.update(account.balance.to_be_bytes());
        }

        let digest = hasher.finalize();
        hex::encode(digest)
    }

    /// Attaches the node seal payload.
    pub fn seal_with_node(&mut self, signature: Vec<u8>) {
        self.genesis_seal.node_sig = Some(signature);
    }

    /// Attaches the AI verification seal payload.
    pub fn seal_with_ai(&mut self, signature: Vec<u8>) {
        self.genesis_seal.ai_sig = Some(signature);
    }

    /// Attaches the DAO approval seal payload.
    pub fn seal_with_dao(&mut self, signature: Vec<u8>) {
        self.genesis_seal.dao_sig = Some(signature);
    }

    /// Returns true when the runtime genesis seal is complete.
    #[must_use]
    pub fn is_sealed(&self) -> bool {
        self.genesis_seal.is_complete()
    }
}

/// Immutable block used to bootstrap the chain.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GenesisBlock {
    /// Chain identifier.
    pub chain_id: String,

    /// Genesis timestamp in UNIX seconds.
    pub timestamp: u64,

    /// Genesis configuration.
    pub config: GenesisConfig,

    /// Deterministic state hash.
    pub state_hash: String,
}

impl GenesisBlock {
    /// Creates a genesis block from configuration with explicit error handling.
    pub fn try_new(config: GenesisConfig) -> Result<Self, String> {
        config.validate()?;

        let timestamp = u64::try_from(chrono::Utc::now().timestamp())
            .map_err(|_| "GENESIS_BLOCK: timestamp must be non-negative".to_string())?;
        let state_hash = config.try_state_hash()?;

        Ok(Self {
            chain_id: config.chain_id.clone(),
            timestamp,
            config,
            state_hash,
        })
    }

    /// Creates a genesis block from configuration.
    ///
    /// This constructor preserves the current infallible API contract.
    /// Invalid configuration is treated as a bootstrap invariant violation.
    #[must_use]
    pub fn new(config: GenesisConfig) -> Self {
        Self::try_new(config).expect("GENESIS_BLOCK: invalid genesis configuration")
    }

    /// Creates a default genesis block.
    ///
    /// Compatibility behavior is preserved:
    /// - treasury is funded;
    /// - the canonical treasury account is inserted into `accounts`.
    ///
    /// `total_supply()` is hardened to avoid double-counting this treasury entry.
    #[must_use]
    pub fn new_default() -> Self {
        let mut config = GenesisConfig::new();

        config.treasury = 1_000_000_000;
        config.add_account(TREASURY_ACCOUNT.to_string(), 1_000_000_000);

        Self::new(config)
    }
}

/// Provides the default runtime genesis seal for deserialization paths.
fn default_genesis_seal() -> AOXCANDSeal {
    AOXCANDSeal::new()
}

/// Returns true when an account address conforms to the current genesis policy.
///
/// Current policy:
/// - non-empty after trimming;
/// - bounded maximum length;
/// - ASCII alphanumeric, underscore, hyphen, and dot are allowed.
fn is_valid_address(value: &str) -> bool {
    let trimmed = value.trim();

    if trimmed.is_empty() || trimmed.len() > MAX_ADDRESS_LEN {
        return false;
    }

    trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
}

fn validate_settlement_link(link: &SettlementLink) -> Result<(), String> {
    let symbol = link.native_symbol.trim();
    if symbol.is_empty() || symbol.len() > 16 || !symbol.chars().all(|ch| ch.is_ascii_uppercase()) {
        return Err(
            "GENESIS: settlement native_symbol must be uppercase ASCII and <=16 chars".to_string(),
        );
    }

    if link.native_decimals > 30 {
        return Err("GENESIS: settlement native_decimals must be <= 30".to_string());
    }

    if link.settlement_network.trim().is_empty() || link.settlement_network.len() > 32 {
        return Err("GENESIS: settlement network id is invalid".to_string());
    }

    if link.equivalence_mode.trim().is_empty() || link.equivalence_mode.len() > 16 {
        return Err("GENESIS: settlement equivalence_mode is invalid".to_string());
    }

    for (label, value) in [
        ("settlement_token_address", &link.settlement_token_address),
        ("settlement_main_contract", &link.settlement_main_contract),
        (
            "settlement_multisig_contract",
            &link.settlement_multisig_contract,
        ),
    ] {
        if value.len() > MAX_METADATA_LEN || !is_valid_evm_address(value) {
            return Err(format!("GENESIS: invalid {}", label));
        }
    }

    Ok(())
}

fn is_valid_evm_address(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.len() != 42 || !trimmed.starts_with("0x") {
        return false;
    }

    trimmed[2..].chars().all(|ch| ch.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::{GenesisConfig, SettlementLink};

    #[test]
    fn settlement_defaults_are_valid() {
        let cfg = GenesisConfig::new();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn settlement_invalid_token_address_is_rejected() {
        let mut cfg = GenesisConfig::new();
        cfg.settlement_link.settlement_token_address = "bad-address".to_string();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn settlement_link_changes_hash() {
        let mut first = GenesisConfig::new();
        let second = GenesisConfig::new();

        first.settlement_link = SettlementLink {
            settlement_network: "xlayer-alt".to_string(),
            ..first.settlement_link.clone()
        };

        assert_ne!(first.state_hash(), second.state_hash());
    }

    #[test]
    fn try_new_rejects_invalid_genesis_config_without_panicking() {
        let mut cfg = GenesisConfig::new();
        cfg.chain_id.clear();

        let err = super::GenesisBlock::try_new(cfg).expect_err("invalid config must be rejected");
        assert!(err.starts_with("GENESIS:"));
    }
}

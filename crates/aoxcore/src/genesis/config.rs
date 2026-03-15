use aoxcunity::Validator;

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

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

        Ok(())
    }

    /// Computes the deterministic genesis state hash.
    ///
    /// Compatibility design:
    /// - runtime-only fields (`validators`, `genesis_seal`) remain excluded;
    /// - accounts are canonically ordered by address before hashing;
    /// - the serialized hash input is stable across insertion-order differences.
    #[must_use]
    pub fn state_hash(&self) -> String {
        #[derive(Serialize)]
        struct CanonicalGenesisConfig<'a> {
            chain_num: u32,
            chain_id: &'a str,
            block_time: u64,
            accounts: Vec<&'a GenesisAccount>,
            treasury: u128,
        }

        let mut accounts: Vec<&GenesisAccount> = self.accounts.iter().collect();
        accounts.sort_by(|left, right| left.address.cmp(&right.address));

        let canonical = CanonicalGenesisConfig {
            chain_num: self.chain_num,
            chain_id: &self.chain_id,
            block_time: self.block_time,
            accounts,
            treasury: self.treasury,
        };

        let encoded =
            serde_json::to_vec(&canonical).expect("GENESIS_HASH: canonical serialization failure");

        let mut hasher = Sha3_256::new();
        hasher.update(encoded);

        let digest = hasher.finalize();
        hex::encode(digest)
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
    /// Creates a genesis block from configuration.
    ///
    /// This constructor preserves the current infallible API contract.
    /// Invalid configuration is treated as a bootstrap invariant violation.
    #[must_use]
    pub fn new(config: GenesisConfig) -> Self {
        config
            .validate()
            .expect("GENESIS_BLOCK: invalid genesis configuration");

        let timestamp = u64::try_from(chrono::Utc::now().timestamp())
            .expect("GENESIS_BLOCK: timestamp must be non-negative");
        let state_hash = config.state_hash();

        Self {
            chain_id: config.chain_id.clone(),
            timestamp,
            config,
            state_hash,
        }
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

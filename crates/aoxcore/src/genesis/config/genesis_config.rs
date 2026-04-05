/// Genesis configuration root.
///
/// Security rationale:
/// This structure is intended to remain fingerprintable under a canonical
/// encoding. No consensus-sensitive fingerprint may depend on debug formatting
/// or unstable serialization behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Canonical chain identity.
    pub identity: ChainIdentity,

    /// Block target time in milliseconds.
    pub block_time: u64,

    /// Genesis validators.
    pub validators: Vec<Validator>,

    /// Genesis accounts and their initial balances / permissions.
    pub accounts: Vec<GenesisAccount>,

    /// Initial treasury allocation.
    pub treasury: u128,

    /// Settlement anchoring configuration.
    pub settlement_link: SettlementLink,

    /// AOXC genesis seal material.
    pub genesis_seal: AOXCANDSeal,

    /// Genesis creation timestamp (unix seconds, UTC).
    #[serde(default = "default_genesis_timestamp")]
    pub genesis_time_unix: u64,

    /// Protocol version for compatibility pinning.
    #[serde(default = "default_protocol_version")]
    pub protocol_version: String,

    /// Quantum-resilience and cryptographic policy profile.
    #[serde(default)]
    pub quantum_policy: QuantumPolicy,

    /// Canonical bootstrap peers for the selected network.
    #[serde(default)]
    pub boot_nodes: Vec<BootNode>,

    /// Kernel-level staking, reward, slash, and multisig policy for seven node roles.
    #[serde(default)]
    pub node_policy: NodePolicy,

    /// Human-readable genesis release notes.
    #[serde(default)]
    pub genesis_notes: String,
}

impl GenesisConfig {
    /// Constructs a validated genesis configuration.
    pub fn new(
        identity: ChainIdentity,
        block_time: u64,
        validators: Vec<Validator>,
        accounts: Vec<GenesisAccount>,
        treasury: u128,
        settlement_link: SettlementLink,
        genesis_seal: AOXCANDSeal,
    ) -> Result<Self, GenesisConfigError> {
        let network_class = identity.network_class;

        let cfg = Self {
            identity,
            block_time,
            validators,
            accounts,
            treasury,
            settlement_link,
            genesis_seal,
            genesis_time_unix: default_genesis_timestamp(),
            protocol_version: default_protocol_version(),
            quantum_policy: QuantumPolicy::for_network_class(network_class),
            boot_nodes: default_boot_nodes(network_class),
            node_policy: NodePolicy::for_network_class(network_class),
            genesis_notes: default_genesis_notes(network_class).to_string(),
        };

        cfg.validate()?;
        Ok(cfg)
    }

    /// Returns a canonical public mainnet identity helper.
    pub fn mainnet_identity(
        chain_name: impl Into<String>,
    ) -> Result<ChainIdentity, GenesisConfigError> {
        ChainIdentity::new(
            AOXC_FAMILY_ID,
            NetworkClass::PublicMainnet,
            1,
            1,
            chain_name,
        )
    }

    /// Returns a canonical public testnet identity helper.
    pub fn testnet_identity(
        chain_name: impl Into<String>,
    ) -> Result<ChainIdentity, GenesisConfigError> {
        ChainIdentity::new(
            AOXC_FAMILY_ID,
            NetworkClass::PublicTestnet,
            2,
            1,
            chain_name,
        )
    }

    /// Validates genesis invariants.
    pub fn validate(&self) -> Result<(), GenesisConfigError> {
        self.identity.validate()?;

        if self.block_time == 0 {
            return Err(GenesisConfigError::InvalidBlockTime);
        }

        if self.validators.is_empty() {
            return Err(GenesisConfigError::EmptyValidators);
        }

        if self.accounts.is_empty() {
            return Err(GenesisConfigError::EmptyAccounts);
        }

        if self.protocol_version.trim().is_empty() {
            return Err(GenesisConfigError::InvalidProtocolVersion);
        }

        validate_protocol_version(&self.protocol_version)?;
        validate_genesis_notes(&self.genesis_notes)?;

        self.quantum_policy
            .validate_for_network_class(self.identity.network_class)?;
        self.node_policy
            .validate_for_network_class(self.identity.network_class)?;

        self.settlement_link.validate()?;
        self.genesis_seal.validate()?;

        if self.boot_nodes.is_empty() {
            return Err(GenesisConfigError::EmptyBootNodes);
        }

        validate_unique_validator_ids(&self.validators)?;
        validate_unique_account_addresses(&self.accounts)?;
        validate_unique_boot_node_ids(&self.boot_nodes)?;

        for validator in &self.validators {
            validator.validate()?;
        }

        for account in &self.accounts {
            account.validate()?;
        }

        for node in &self.boot_nodes {
            node.validate()?;
        }

        Ok(())
    }

    /// Computes a deterministic state fingerprint for the genesis configuration.
    ///
    /// Security properties:
    /// - identity material is included,
    /// - dynamic collections are canonically encoded in explicit order,
    /// - debug formatting is never used,
    /// - field framing is explicit and versioned.
    pub fn try_state_hash(&self) -> Result<[u8; 32], GenesisConfigError> {
        self.validate()?;

        let mut enc = CanonicalEncoder::new(DOMAIN_GENESIS_STATE_FINGERPRINT);
        enc.u8(GENESIS_ENCODING_VERSION);

        self.identity.encode_canonical(&mut enc);

        enc.u64(self.block_time);
        enc.u128(self.treasury);
        enc.u64(self.genesis_time_unix);
        enc.str(&self.protocol_version);
        enc.str(&self.genesis_notes);

        enc.usize(self.validators.len())?;
        for validator in &self.validators {
            validator.encode_canonical(&mut enc);
        }

        enc.usize(self.accounts.len())?;
        for account in &self.accounts {
            account.encode_canonical(&mut enc);
        }

        self.settlement_link.encode_canonical(&mut enc);
        self.genesis_seal.encode_canonical(&mut enc);
        self.quantum_policy.encode_canonical(&mut enc);

        enc.usize(self.boot_nodes.len())?;
        for node in &self.boot_nodes {
            node.encode_canonical(&mut enc);
        }
        self.node_policy.encode_canonical(&mut enc)?;

        Ok(enc.finish())
    }

    /// Returns the hex-encoded deterministic genesis fingerprint.
    pub fn fingerprint(&self) -> Result<String, GenesisConfigError> {
        Ok(hex::encode(self.try_state_hash()?))
    }
}

/// Canonical numeric chain identifier builder.
///
/// Format:
/// `FFFFCCNNNN`
pub fn build_chain_id(
    family_id: u32,
    network_class: NetworkClass,
    class_instance_ordinal: u32,
) -> Result<u64, GenesisConfigError> {
    if family_id == 0 {
        return Err(GenesisConfigError::InvalidFamilyId);
    }

    if class_instance_ordinal == 0 || class_instance_ordinal > MAX_NETWORK_INSTANCE_ORDINAL {
        return Err(GenesisConfigError::InvalidClassInstanceOrdinal {
            value: class_instance_ordinal,
        });
    }

    let class_code = u64::from(network_class.class_code());
    let family = u64::from(family_id);
    let ordinal = u64::from(class_instance_ordinal);

    Ok((family * 1_000_000) + (class_code * 10_000) + ordinal)
}

/// Canonical governance serial builder.
///
/// Format:
/// `FFFF-NNN`
#[must_use]
pub fn build_network_serial(family_id: u32, governance_serial_ordinal: u16) -> String {
    format!("{family_id}-{governance_serial_ordinal:03}")
}

/// Canonical machine-readable network identifier builder.
///
/// Format:
/// `aoxc-<class>-<family>-<ordinal>`
#[must_use]
pub fn build_network_id(network_class: NetworkClass, network_serial: &str) -> String {
    format!(
        "{}-{}-{}",
        AOXC_NETWORK_ID_PREFIX,
        network_class.slug(),
        network_serial.to_ascii_lowercase()
    )
}

/// Validates governance serial shape.
///
/// Expected format:
/// - `<family_id>-<3 digit ordinal>`
pub fn validate_network_serial(
    expected_family_id: u32,
    network_serial: &str,
) -> Result<(), GenesisConfigError> {
    let Some((family, ordinal)) = network_serial.split_once('-') else {
        return Err(GenesisConfigError::MalformedNetworkSerial);
    };

    let parsed_family = family
        .parse::<u32>()
        .map_err(|_| GenesisConfigError::MalformedNetworkSerial)?;

    if parsed_family != expected_family_id {
        return Err(GenesisConfigError::NetworkSerialFamilyMismatch {
            expected: expected_family_id,
            actual: parsed_family,
        });
    }

    if ordinal.len() != 3 || !ordinal.chars().all(|c| c.is_ascii_digit()) {
        return Err(GenesisConfigError::MalformedNetworkSerial);
    }

    let parsed_ordinal = ordinal
        .parse::<u16>()
        .map_err(|_| GenesisConfigError::MalformedNetworkSerial)?;

    if parsed_ordinal == 0 || parsed_ordinal > MAX_NETWORK_SERIAL_ORDINAL {
        return Err(GenesisConfigError::InvalidNetworkSerialOrdinal {
            value: parsed_ordinal,
        });
    }

    Ok(())
}

/// Validates canonical machine-readable network identifier shape.
pub fn validate_network_id(
    network_class: NetworkClass,
    network_serial: &str,
    network_id: &str,
) -> Result<(), GenesisConfigError> {
    let expected = build_network_id(network_class, network_serial);
    if network_id != expected {
        return Err(GenesisConfigError::NetworkIdMismatch {
            expected,
            actual: network_id.to_owned(),
        });
    }

    Ok(())
}

fn validate_chain_name(chain_name: &str) -> Result<(), GenesisConfigError> {
    let trimmed = chain_name.trim();

    if trimmed.is_empty() {
        return Err(GenesisConfigError::EmptyChainName);
    }

    if trimmed.len() > MAX_CHAIN_NAME_LEN {
        return Err(GenesisConfigError::InvalidChainNameLength {
            length: trimmed.len(),
        });
    }

    Ok(())
}

fn validate_protocol_version(version: &str) -> Result<(), GenesisConfigError> {
    let trimmed = version.trim();

    if trimmed.is_empty() {
        return Err(GenesisConfigError::InvalidProtocolVersion);
    }

    if trimmed.len() > MAX_PROTOCOL_VERSION_LEN {
        return Err(GenesisConfigError::ProtocolVersionTooLong {
            length: trimmed.len(),
        });
    }

    Ok(())
}

fn validate_genesis_notes(notes: &str) -> Result<(), GenesisConfigError> {
    if notes.len() > MAX_GENESIS_NOTES_LEN {
        return Err(GenesisConfigError::GenesisNotesTooLong {
            length: notes.len(),
        });
    }

    Ok(())
}

/// Returns the expected `FFFFCC` prefix segment of the numeric `chain_id`.
fn expected_chain_id_prefix(family_id: u32, network_class: NetworkClass) -> u64 {
    (u64::from(family_id) * 100) + u64::from(network_class.class_code())
}

fn validate_unique_validator_ids(validators: &[Validator]) -> Result<(), GenesisConfigError> {
    let mut seen = HashSet::with_capacity(validators.len());

    for validator in validators {
        if !seen.insert(validator.id.as_str()) {
            return Err(GenesisConfigError::DuplicateValidatorId {
                id: validator.id.clone(),
            });
        }
    }

    Ok(())
}

fn validate_unique_account_addresses(
    accounts: &[GenesisAccount],
) -> Result<(), GenesisConfigError> {
    let mut seen = HashSet::with_capacity(accounts.len());

    for account in accounts {
        if !seen.insert(account.address.as_str()) {
            return Err(GenesisConfigError::DuplicateAccountAddress {
                address: account.address.clone(),
            });
        }
    }

    Ok(())
}

fn validate_unique_boot_node_ids(nodes: &[BootNode]) -> Result<(), GenesisConfigError> {
    let mut seen = HashSet::with_capacity(nodes.len());

    for node in nodes {
        if !seen.insert(node.node_id.as_str()) {
            return Err(GenesisConfigError::DuplicateBootNodeId {
                node_id: node.node_id.clone(),
            });
        }
    }

    Ok(())
}

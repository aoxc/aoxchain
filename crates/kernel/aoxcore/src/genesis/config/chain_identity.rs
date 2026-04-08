/// Governance-facing chain identity model.
///
/// Security rationale:
/// This structure intentionally carries both human-readable and
/// machine-readable identifiers. These fields should become immutable once
/// genesis is finalized for a given network instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainIdentity {
    /// AOXC family namespace root.
    pub family_id: u32,

    /// Canonical protocol-facing numeric chain identifier.
    ///
    /// Example:
    /// - `2626000001` for public mainnet instance 1
    /// - `2626010001` for public testnet instance 1
    pub chain_id: u64,

    /// Human-readable governance serial.
    ///
    /// Example:
    /// - `2626-001`
    /// - `2626-101`
    pub network_serial: String,

    /// Canonical machine-readable network identifier.
    ///
    /// Example:
    /// - `aoxc-mainnet-2626-001`
    /// - `aoxc-testnet-2626-002`
    pub network_id: String,

    /// Human-readable chain display name.
    pub chain_name: String,

    /// Canonical network class.
    pub network_class: NetworkClass,
}

impl ChainIdentity {
    /// Constructs a canonical chain identity from explicit policy inputs.
    ///
    /// This constructor deterministically derives:
    /// - `chain_id`
    /// - `network_serial`
    /// - `network_id`
    pub fn new(
        family_id: u32,
        network_class: NetworkClass,
        governance_serial_ordinal: u16,
        class_instance_ordinal: u32,
        chain_name: impl Into<String>,
    ) -> Result<Self, GenesisConfigError> {
        if governance_serial_ordinal == 0 || governance_serial_ordinal > MAX_NETWORK_SERIAL_ORDINAL
        {
            return Err(GenesisConfigError::InvalidNetworkSerialOrdinal {
                value: governance_serial_ordinal,
            });
        }

        if class_instance_ordinal == 0 || class_instance_ordinal > MAX_NETWORK_INSTANCE_ORDINAL {
            return Err(GenesisConfigError::InvalidClassInstanceOrdinal {
                value: class_instance_ordinal,
            });
        }

        let chain_name = chain_name.into();
        validate_chain_name(&chain_name)?;

        let network_serial = build_network_serial(family_id, governance_serial_ordinal);
        let chain_id = build_chain_id(family_id, network_class, class_instance_ordinal)?;
        let network_id = build_network_id(network_class, &network_serial);

        let identity = Self {
            family_id,
            chain_id,
            network_serial,
            network_id,
            chain_name,
            network_class,
        };

        identity.validate()?;
        Ok(identity)
    }

    /// Validates canonical identity invariants.
    pub fn validate(&self) -> Result<(), GenesisConfigError> {
        if self.family_id == 0 {
            return Err(GenesisConfigError::InvalidFamilyId);
        }

        validate_chain_name(&self.chain_name)?;
        validate_network_serial(self.family_id, &self.network_serial)?;
        validate_network_id(self.network_class, &self.network_serial, &self.network_id)?;

        let expected_prefix = expected_chain_id_prefix(self.family_id, self.network_class);
        let actual_prefix = self.chain_id / 10_000;

        if actual_prefix != expected_prefix {
            return Err(GenesisConfigError::ChainIdPrefixMismatch {
                expected_prefix,
                actual_prefix,
            });
        }

        let instance_ordinal = self.chain_id % 10_000;
        if instance_ordinal == 0 || instance_ordinal > u64::from(MAX_NETWORK_INSTANCE_ORDINAL) {
            return Err(GenesisConfigError::InvalidDerivedChainIdOrdinal {
                value: instance_ordinal,
            });
        }

        Ok(())
    }

    /// Returns a deterministic fingerprint for the identity surface.
    ///
    /// This is not a signature and must not be treated as such.
    #[must_use]
    pub fn fingerprint_hex(&self) -> String {
        let mut enc = CanonicalEncoder::new(DOMAIN_CHAIN_IDENTITY_FINGERPRINT);
        self.encode_canonical(&mut enc);
        hex::encode(enc.finish())
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.u8(GENESIS_ENCODING_VERSION);
        enc.u32(self.family_id);
        enc.u64(self.chain_id);
        enc.str(&self.network_serial);
        enc.str(&self.network_id);
        enc.str(&self.chain_name);
        enc.str(self.network_class.slug());
    }
}

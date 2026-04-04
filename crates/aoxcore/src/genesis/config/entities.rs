/// Validator descriptor.
///
/// Integration note:
/// This placeholder-compatible type is intentionally strict enough for
/// production-oriented genesis hardening while remaining lightweight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Validator {
    pub id: String,
}

impl Validator {
    fn validate(&self) -> Result<(), GenesisConfigError> {
        validate_identifier(&self.id).map_err(|_| GenesisConfigError::DuplicateValidatorId {
            id: self.id.clone(),
        })
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.str(&self.id);
    }
}

/// Genesis account descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisAccount {
    pub address: String,
    pub balance: u128,
}

impl GenesisAccount {
    fn validate(&self) -> Result<(), GenesisConfigError> {
        validate_identifier(&self.address).map_err(|_| {
            GenesisConfigError::DuplicateAccountAddress {
                address: self.address.clone(),
            }
        })
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.str(&self.address);
        enc.u128(self.balance);
    }
}

/// Settlement anchoring descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementLink {
    pub endpoint: String,
}

impl SettlementLink {
    fn validate(&self) -> Result<(), GenesisConfigError> {
        validate_endpoint(&self.endpoint).map_err(|_| GenesisConfigError::InvalidSettlementLink)
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.str(&self.endpoint);
    }
}

/// Genesis seal descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AOXCANDSeal {
    pub seal_id: String,
}

impl AOXCANDSeal {
    fn validate(&self) -> Result<(), GenesisConfigError> {
        validate_identifier(&self.seal_id).map_err(|_| GenesisConfigError::InvalidGenesisSeal)
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.str(&self.seal_id);
    }
}

/// Canonical bootstrap node descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BootNode {
    pub node_id: String,
    pub endpoint: String,
    pub role: String,
}

impl BootNode {
    fn validate(&self) -> Result<(), GenesisConfigError> {
        validate_identifier(&self.node_id).map_err(|_| GenesisConfigError::InvalidBootNode {
            node_id: self.node_id.clone(),
        })?;

        validate_identifier(&self.role).map_err(|_| GenesisConfigError::InvalidBootNode {
            node_id: self.node_id.clone(),
        })?;

        validate_endpoint(&self.endpoint).map_err(|_| GenesisConfigError::InvalidBootNode {
            node_id: self.node_id.clone(),
        })?;

        Ok(())
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.str(&self.node_id);
        enc.str(&self.endpoint);
        enc.str(&self.role);
    }
}

/// Quantum security policy profile for genesis.
///
/// Security rationale:
/// This is a policy-bearing object, not a descriptive note. Validation must
/// reject structurally invalid or cryptographically weak profiles for the
/// target network class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuantumPolicy {
    pub pq_signature_schemes: Vec<String>,
    pub classical_signature_schemes: Vec<String>,
    pub handshake_kem: String,
    pub state_hash: String,
    pub commitment_hash: String,
    pub min_signature_threshold: u16,
    pub rotation_epoch_blocks: u64,
}

impl Default for QuantumPolicy {
    fn default() -> Self {
        Self::for_network_class(NetworkClass::Devnet)
    }
}

impl QuantumPolicy {
    #[must_use]
    pub fn for_network_class(network_class: NetworkClass) -> Self {
        match network_class {
            NetworkClass::PublicMainnet => Self {
                pq_signature_schemes: vec!["ML-DSA-87".into(), "SLH-DSA-SHA2-192f".into()],
                classical_signature_schemes: vec!["Ed25519".into()],
                handshake_kem: "ML-KEM-1024".into(),
                state_hash: "SHA3-256".into(),
                commitment_hash: "BLAKE3".into(),
                min_signature_threshold: 3,
                rotation_epoch_blocks: 43_200,
            },
            NetworkClass::PublicTestnet | NetworkClass::Validation => Self {
                pq_signature_schemes: vec!["ML-DSA-65".into()],
                classical_signature_schemes: vec!["Ed25519".into()],
                handshake_kem: "ML-KEM-768".into(),
                state_hash: "SHA3-256".into(),
                commitment_hash: "BLAKE3".into(),
                min_signature_threshold: 2,
                rotation_epoch_blocks: 14_400,
            },
            NetworkClass::Devnet
            | NetworkClass::SovereignPrivate
            | NetworkClass::Consortium
            | NetworkClass::RegulatedPrivate => Self {
                pq_signature_schemes: vec!["ML-DSA-44".into()],
                classical_signature_schemes: vec!["Ed25519".into()],
                handshake_kem: "ML-KEM-512".into(),
                state_hash: "SHA3-256".into(),
                commitment_hash: "BLAKE3".into(),
                min_signature_threshold: 1,
                rotation_epoch_blocks: 7_200,
            },
        }
    }

    fn validate_for_network_class(
        &self,
        network_class: NetworkClass,
    ) -> Result<(), GenesisConfigError> {
        if self.pq_signature_schemes.is_empty()
            || self.classical_signature_schemes.is_empty()
            || self.handshake_kem.trim().is_empty()
            || self.state_hash.trim().is_empty()
            || self.commitment_hash.trim().is_empty()
            || self.min_signature_threshold == 0
            || self.rotation_epoch_blocks == 0
        {
            return Err(GenesisConfigError::InvalidQuantumPolicy);
        }

        for alg in &self.pq_signature_schemes {
            validate_algorithm_name(alg)?;
        }

        for alg in &self.classical_signature_schemes {
            validate_algorithm_name(alg)?;
        }

        validate_algorithm_name(&self.handshake_kem)?;
        validate_algorithm_name(&self.state_hash)?;
        validate_algorithm_name(&self.commitment_hash)?;

        if network_class == NetworkClass::PublicMainnet {
            if self.handshake_kem != "ML-KEM-1024" {
                return Err(GenesisConfigError::WeakQuantumPolicy {
                    reason: "public mainnet requires ML-KEM-1024 for handshake policy",
                });
            }

            if self.min_signature_threshold < 3 {
                return Err(GenesisConfigError::WeakQuantumPolicy {
                    reason: "public mainnet requires a minimum signature threshold of 3",
                });
            }

            if !self
                .pq_signature_schemes
                .iter()
                .any(|alg| alg == "ML-DSA-87")
            {
                return Err(GenesisConfigError::WeakQuantumPolicy {
                    reason: "public mainnet requires ML-DSA-87 in the PQ signature policy",
                });
            }
        }

        if self.state_hash != "SHA3-256" {
            return Err(GenesisConfigError::WeakQuantumPolicy {
                reason: "state hash policy must use SHA3-256",
            });
        }

        if self.commitment_hash != "BLAKE3" {
            return Err(GenesisConfigError::WeakQuantumPolicy {
                reason: "commitment hash policy must use BLAKE3",
            });
        }

        Ok(())
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.strs(&self.pq_signature_schemes).ok();
        enc.strs(&self.classical_signature_schemes).ok();
        enc.str(&self.handshake_kem);
        enc.str(&self.state_hash);
        enc.str(&self.commitment_hash);
        enc.u16(self.min_signature_threshold);
        enc.u64(self.rotation_epoch_blocks);
    }
}

fn validate_identifier(value: &str) -> Result<(), ()> {
    let trimmed = value.trim();

    if trimmed.is_empty() || trimmed.len() > MAX_IDENTIFIER_LEN {
        return Err(());
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' || c == '.')
    {
        return Err(());
    }

    Ok(())
}

fn validate_endpoint(value: &str) -> Result<(), ()> {
    let trimmed = value.trim();

    if trimmed.is_empty() || trimmed.len() > MAX_ENDPOINT_LEN {
        return Err(());
    }

    if !trimmed.contains("://") {
        return Err(());
    }

    Ok(())
}

fn validate_algorithm_name(value: &str) -> Result<(), GenesisConfigError> {
    let trimmed = value.trim();

    if trimmed.is_empty()
        || trimmed.len() > MAX_ALGORITHM_NAME_LEN
        || !trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(GenesisConfigError::InvalidQuantumAlgorithmName {
            value: value.to_string(),
        });
    }

    Ok(())
}

fn default_genesis_timestamp() -> u64 {
    DEFAULT_GENESIS_TIMESTAMP_UNIX
}

fn default_protocol_version() -> String {
    "aoxc-genesis/v3".to_string()
}

fn default_genesis_notes(network_class: NetworkClass) -> &'static str {
    match network_class {
        NetworkClass::PublicMainnet => {
            "AOXC mainnet genesis: production consensus, strict post-quantum threshold policy, audited settlement profile."
        }
        NetworkClass::PublicTestnet => {
            "AOXC testnet genesis: interoperability rehearsal, PQ migration validation, accelerated validator rotation."
        }
        NetworkClass::Devnet => {
            "AOXC devnet genesis: rapid experimentation profile with bounded cryptographic downgrade allowances."
        }
        NetworkClass::Validation => {
            "AOXC validation genesis: pre-production hardening and release-candidate verification."
        }
        NetworkClass::SovereignPrivate => "AOXC sovereign-private genesis profile.",
        NetworkClass::Consortium => "AOXC consortium genesis profile.",
        NetworkClass::RegulatedPrivate => "AOXC regulated-private genesis profile.",
    }
}

fn default_boot_nodes(network_class: NetworkClass) -> Vec<BootNode> {
    let suffix = network_class.slug();

    vec![
        BootNode {
            node_id: format!("aoxc-{suffix}-boot-001"),
            endpoint: format!("aoxc://{suffix}.seed-001.aoxc.net:443"),
            role: "seed".to_string(),
        },
        BootNode {
            node_id: format!("aoxc-{suffix}-boot-002"),
            endpoint: format!("aoxc://{suffix}.seed-002.aoxc.net:443"),
            role: "relay".to_string(),
        },
    ]
}

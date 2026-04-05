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

/// Canonical node roles for the AOXC seven-role topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum NodeRole {
    Forge,
    Quorum,
    Seal,
    Archive,
    Sentinel,
    Relay,
    Pocket,
}

impl NodeRole {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Forge => "forge",
            Self::Quorum => "quorum",
            Self::Seal => "seal",
            Self::Archive => "archive",
            Self::Sentinel => "sentinel",
            Self::Relay => "relay",
            Self::Pocket => "pocket",
        }
    }

    #[must_use]
    const fn all() -> [Self; 7] {
        [
            Self::Forge,
            Self::Quorum,
            Self::Seal,
            Self::Archive,
            Self::Sentinel,
            Self::Relay,
            Self::Pocket,
        ]
    }
}

/// Role-specific staking, reward, and slashing policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeRolePolicy {
    pub role: NodeRole,
    pub min_stake: u128,
    pub reward_bps: u16,
    pub slash_bps: u16,
    pub multisig_threshold: u8,
    pub multisig_participants: u8,
    pub quantum_seal_required: bool,
}

impl NodeRolePolicy {
    fn validate(&self) -> Result<(), GenesisConfigError> {
        if self.reward_bps > 10_000
            || self.slash_bps > 10_000
            || self.multisig_threshold == 0
            || self.multisig_participants == 0
            || self.multisig_threshold > self.multisig_participants
        {
            return Err(GenesisConfigError::InvalidNodePolicy);
        }

        if self.multisig_threshold < 2 {
            return Err(GenesisConfigError::WeakNodeRolePolicy {
                role: self.role.as_str().to_string(),
                reason: "all node roles must enforce multisig threshold >= 2",
            });
        }

        if self.role != NodeRole::Pocket && self.min_stake == 0 {
            return Err(GenesisConfigError::WeakNodeRolePolicy {
                role: self.role.as_str().to_string(),
                reason: "non-pocket roles must require a positive minimum stake",
            });
        }

        if self.role == NodeRole::Seal && !self.quantum_seal_required {
            return Err(GenesisConfigError::WeakNodeRolePolicy {
                role: self.role.as_str().to_string(),
                reason: "seal role must require quantum sealing",
            });
        }

        Ok(())
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.str(self.role.as_str());
        enc.u128(self.min_stake);
        enc.u16(self.reward_bps);
        enc.u16(self.slash_bps);
        enc.u8(self.multisig_threshold);
        enc.u8(self.multisig_participants);
        enc.u8(u8::from(self.quantum_seal_required));
    }
}

/// Node policy profile that governs the seven-role AOXC node topology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodePolicy {
    pub role_policies: Vec<NodeRolePolicy>,
    pub seal_layers: Vec<SealLayerPolicy>,
    pub treasury_reward_bps: u16,
    pub governance_epoch_blocks: u64,
}

impl Default for NodePolicy {
    fn default() -> Self {
        Self::for_network_class(NetworkClass::Devnet)
    }
}

impl NodePolicy {
    #[must_use]
    pub fn for_network_class(network_class: NetworkClass) -> Self {
        let (stake_multiplier, epoch_blocks) = match network_class {
            NetworkClass::PublicMainnet => (10_u128, 43_200),
            NetworkClass::PublicTestnet | NetworkClass::Validation => (3_u128, 14_400),
            NetworkClass::Devnet
            | NetworkClass::SovereignPrivate
            | NetworkClass::Consortium
            | NetworkClass::RegulatedPrivate => (1_u128, 7_200),
        };

        Self {
            role_policies: vec![
                NodeRolePolicy {
                    role: NodeRole::Forge,
                    min_stake: 50_000 * stake_multiplier,
                    reward_bps: 2_800,
                    slash_bps: 2_000,
                    multisig_threshold: 3,
                    multisig_participants: 5,
                    quantum_seal_required: false,
                },
                NodeRolePolicy {
                    role: NodeRole::Quorum,
                    min_stake: 75_000 * stake_multiplier,
                    reward_bps: 2_400,
                    slash_bps: 3_500,
                    multisig_threshold: 4,
                    multisig_participants: 7,
                    quantum_seal_required: false,
                },
                NodeRolePolicy {
                    role: NodeRole::Seal,
                    min_stake: 40_000 * stake_multiplier,
                    reward_bps: 1_400,
                    slash_bps: 4_000,
                    multisig_threshold: 3,
                    multisig_participants: 5,
                    quantum_seal_required: true,
                },
                NodeRolePolicy {
                    role: NodeRole::Archive,
                    min_stake: 10_000 * stake_multiplier,
                    reward_bps: 900,
                    slash_bps: 800,
                    multisig_threshold: 2,
                    multisig_participants: 3,
                    quantum_seal_required: false,
                },
                NodeRolePolicy {
                    role: NodeRole::Sentinel,
                    min_stake: 20_000 * stake_multiplier,
                    reward_bps: 1_100,
                    slash_bps: 1_500,
                    multisig_threshold: 3,
                    multisig_participants: 5,
                    quantum_seal_required: false,
                },
                NodeRolePolicy {
                    role: NodeRole::Relay,
                    min_stake: 5_000 * stake_multiplier,
                    reward_bps: 800,
                    slash_bps: 600,
                    multisig_threshold: 2,
                    multisig_participants: 3,
                    quantum_seal_required: false,
                },
                NodeRolePolicy {
                    role: NodeRole::Pocket,
                    min_stake: 0,
                    reward_bps: 300,
                    slash_bps: 200,
                    multisig_threshold: 2,
                    multisig_participants: 2,
                    quantum_seal_required: false,
                },
            ],
            seal_layers: vec![SealLayerPolicy {
                layer_id: "seal-v1".to_string(),
                commitment_hash: "BLAKE3".to_string(),
                activation_epoch: 0,
                quantum_hardened: true,
            }],
            treasury_reward_bps: 300,
            governance_epoch_blocks: epoch_blocks,
        }
    }

    /// Kernel-level multisig threshold checker for role-scoped authorization paths.
    pub fn is_multisig_quorum_satisfied(
        &self,
        role: NodeRole,
        observed_signatures: u8,
        observed_participants: u8,
    ) -> Result<bool, GenesisConfigError> {
        let policy = self
            .role_policies
            .iter()
            .find(|policy| policy.role == role)
            .ok_or(GenesisConfigError::MissingNodeRolePolicy {
                role: role.as_str().to_string(),
            })?;

        Ok(observed_participants >= policy.multisig_participants
            && observed_signatures >= policy.multisig_threshold)
    }

    fn validate_for_network_class(
        &self,
        network_class: NetworkClass,
    ) -> Result<(), GenesisConfigError> {
        if self.role_policies.len() != NodeRole::all().len()
            || self.seal_layers.is_empty()
            || self.treasury_reward_bps > 10_000
            || self.governance_epoch_blocks == 0
        {
            return Err(GenesisConfigError::InvalidNodePolicy);
        }

        let mut seen = HashSet::with_capacity(self.role_policies.len());
        for role_policy in &self.role_policies {
            role_policy.validate()?;
            if !seen.insert(role_policy.role) {
                return Err(GenesisConfigError::DuplicateNodeRolePolicy {
                    role: role_policy.role.as_str().to_string(),
                });
            }
        }

        for role in NodeRole::all() {
            if !seen.contains(&role) {
                return Err(GenesisConfigError::MissingNodeRolePolicy {
                    role: role.as_str().to_string(),
                });
            }
        }

        let mut seal_layer_ids = HashSet::with_capacity(self.seal_layers.len());
        let mut previous_epoch = 0_u64;
        for (index, layer) in self.seal_layers.iter().enumerate() {
            layer.validate()?;
            if !seal_layer_ids.insert(layer.layer_id.as_str()) {
                return Err(GenesisConfigError::DuplicateSealLayerPolicy {
                    layer_id: layer.layer_id.clone(),
                });
            }

            if index > 0 && layer.activation_epoch < previous_epoch {
                return Err(GenesisConfigError::InvalidSealLayerPolicy {
                    layer_id: layer.layer_id.clone(),
                });
            }

            previous_epoch = layer.activation_epoch;
        }

        if network_class == NetworkClass::PublicMainnet {
            let quorum = self
                .role_policies
                .iter()
                .find(|policy| policy.role == NodeRole::Quorum)
                .ok_or(GenesisConfigError::MissingNodeRolePolicy {
                    role: NodeRole::Quorum.as_str().to_string(),
                })?;

            if quorum.multisig_threshold < 4 {
                return Err(GenesisConfigError::WeakNodeRolePolicy {
                    role: NodeRole::Quorum.as_str().to_string(),
                    reason: "public mainnet quorum threshold must be at least 4-of-N",
                });
            }
        }

        Ok(())
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) -> Result<(), GenesisConfigError> {
        let mut ordered = self.role_policies.clone();
        ordered.sort_by_key(|policy| policy.role);

        enc.usize(ordered.len())?;
        for policy in &ordered {
            policy.encode_canonical(enc);
        }

        let mut ordered_layers = self.seal_layers.clone();
        ordered_layers.sort_by(|left, right| left.layer_id.cmp(&right.layer_id));
        enc.usize(ordered_layers.len())?;
        for layer in &ordered_layers {
            layer.encode_canonical(enc);
        }

        enc.u16(self.treasury_reward_bps);
        enc.u64(self.governance_epoch_blocks);
        Ok(())
    }
}

/// Extensible seal-layer policy for retroactive hardening without history rewrites.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealLayerPolicy {
    pub layer_id: String,
    pub commitment_hash: String,
    pub activation_epoch: u64,
    pub quantum_hardened: bool,
}

impl SealLayerPolicy {
    fn validate(&self) -> Result<(), GenesisConfigError> {
        validate_identifier(&self.layer_id).map_err(|_| GenesisConfigError::InvalidSealLayerPolicy {
            layer_id: self.layer_id.clone(),
        })?;
        validate_algorithm_name(&self.commitment_hash)?;

        if self.activation_epoch == 0 && self.layer_id != "seal-v1" {
            return Err(GenesisConfigError::InvalidSealLayerPolicy {
                layer_id: self.layer_id.clone(),
            });
        }

        Ok(())
    }

    fn encode_canonical(&self, enc: &mut CanonicalEncoder) {
        enc.str(&self.layer_id);
        enc.str(&self.commitment_hash);
        enc.u64(self.activation_epoch);
        enc.u8(u8::from(self.quantum_hardened));
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

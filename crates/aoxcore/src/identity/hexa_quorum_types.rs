pub const MAX_ACTOR_ID_LEN: usize = 128;

/// Maximum accepted signature payload length in hexadecimal characters.
///
/// This bound is intentionally generous enough for large post-quantum or
/// composite signature surfaces while rejecting obviously malformed input.
pub const MAX_SIGNATURE_HEX_LEN: usize = 16384;

/// Canonical proof lanes used by the AOXC HexaQuorum engine.
///
/// Security rationale:
/// Each lane represents a distinct trust axis rather than a repeated signature.
/// The goal is to prevent false confidence created by multiple signatures from
/// the same trust source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ApprovalLane {
    /// Primary cryptographic identity signature.
    IdentitySig,

    /// Secondary device-bound or session-bound signature.
    DeviceSig,

    /// Time-delayed reconfirmation proving persistence of intent.
    TimeLockSig,

    /// Economic commitment or slashable locked stake.
    StakeLock,

    /// Proof that the actor is authorized for the action domain.
    RoleProof,

    /// DAO, guardian, council, or governance co-signature.
    DaoCosign,
}

impl ApprovalLane {
    /// Returns a stable symbolic lane code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::IdentitySig => "IDENTITY_SIG",
            Self::DeviceSig => "DEVICE_SIG",
            Self::TimeLockSig => "TIMELOCK_SIG",
            Self::StakeLock => "STAKE_LOCK",
            Self::RoleProof => "ROLE_PROOF",
            Self::DaoCosign => "DAO_COSIGN",
        }
    }

    /// Returns all canonical lanes in deterministic evaluation order.
    #[must_use]
    pub const fn all() -> [ApprovalLane; 6] {
        [
            ApprovalLane::IdentitySig,
            ApprovalLane::DeviceSig,
            ApprovalLane::TimeLockSig,
            ApprovalLane::StakeLock,
            ApprovalLane::RoleProof,
            ApprovalLane::DaoCosign,
        ]
    }
}

/// Canonical proof submitted by an actor into the HexaQuorum engine.
///
/// Notes:
/// - `signature` is optional because not every lane must be cryptographic.
///   For example, `StakeLock` may represent an already-verified economic proof.
/// - `weight` is lane-specific contribution weight, not governance stake.
/// - `stake` is an economic quantity that can be aggregated across actors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalProof {
    pub actor_id: String,
    pub lane: ApprovalLane,
    pub signature: Option<String>,
    pub timestamp: u64,
    pub weight: u64,
    pub stake: u128,
}

impl ApprovalProof {
    /// Performs local semantic validation for a proof.
    pub fn validate(&self) -> Result<(), HexaQuorumError> {
        validate_proof(self)
    }

    /// Returns whether the proof currently carries a non-empty signature.
    #[must_use]
    pub fn has_signature(&self) -> bool {
        self.signature
            .as_ref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
    }
}

/// Policy definition for a HexaQuorum decision.
///
/// Security model:
/// A decision may require:
/// - specific minimum contributions in each lane,
/// - a minimum number of distinct actors,
/// - a minimum total economic stake,
/// - a minimum total weighted score.
///
/// This makes the system much harder to game than a plain M-of-N multisig.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuorumPolicy {
    pub min_identity: u8,
    pub min_device: u8,
    pub min_timelock: u8,
    pub min_stake_lock: u8,
    pub min_role: u8,
    pub min_dao: u8,
    pub min_distinct_actors: u8,
    pub min_total_stake: u128,
    pub min_total_score: u64,
}

impl Default for QuorumPolicy {
    fn default() -> Self {
        Self::strict_default()
    }
}

impl QuorumPolicy {
    /// Default balanced policy for general protected actions.
    #[must_use]
    pub fn strict_default() -> Self {
        Self {
            min_identity: 2,
            min_device: 1,
            min_timelock: 1,
            min_stake_lock: 1,
            min_role: 1,
            min_dao: 1,
            min_distinct_actors: 2,
            min_total_stake: 1,
            min_total_score: 10,
        }
    }

    /// Example policy for sensitive treasury actions.
    #[must_use]
    pub fn treasury_default() -> Self {
        Self {
            min_identity: 3,
            min_device: 2,
            min_timelock: 2,
            min_stake_lock: 2,
            min_role: 2,
            min_dao: 1,
            min_distinct_actors: 3,
            min_total_stake: 1_000,
            min_total_score: 25,
        }
    }

    /// Example policy for validator admission or rotation.
    #[must_use]
    pub fn validator_default() -> Self {
        Self {
            min_identity: 2,
            min_device: 1,
            min_timelock: 1,
            min_stake_lock: 1,
            min_role: 2,
            min_dao: 1,
            min_distinct_actors: 2,
            min_total_stake: 500,
            min_total_score: 16,
        }
    }

    /// Returns the minimum required count for a specific lane.
    #[must_use]
    pub const fn min_for_lane(&self, lane: ApprovalLane) -> u8 {
        match lane {
            ApprovalLane::IdentitySig => self.min_identity,
            ApprovalLane::DeviceSig => self.min_device,
            ApprovalLane::TimeLockSig => self.min_timelock,
            ApprovalLane::StakeLock => self.min_stake_lock,
            ApprovalLane::RoleProof => self.min_role,
            ApprovalLane::DaoCosign => self.min_dao,
        }
    }

    /// Validates the policy as a self-consistent decision contract.
    ///
    /// Validation policy:
    /// - at least one security threshold must be active,
    /// - distinct actor threshold must be non-zero.
    pub fn validate(&self) -> Result<(), HexaQuorumError> {
        let has_any_lane_requirement = ApprovalLane::all()
            .into_iter()
            .any(|lane| self.min_for_lane(lane) > 0);

        let has_any_global_requirement =
            self.min_distinct_actors > 0 || self.min_total_stake > 0 || self.min_total_score > 0;

        if self.min_distinct_actors == 0 {
            return Err(HexaQuorumError::InvalidPolicy);
        }

        if !(has_any_lane_requirement || has_any_global_requirement) {
            return Err(HexaQuorumError::InvalidPolicy);
        }

        Ok(())
    }
}

/// Detailed evaluation result for a HexaQuorum decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuorumResult {
    pub passed: bool,
    pub distinct_actors: usize,
    pub total_score: u64,
    pub total_stake: u128,
    pub lane_counts: LaneCounts,
    pub missing_lanes: Vec<ApprovalLane>,
    pub rejection_reasons: Vec<String>,
}

/// Lane count snapshot used by result reporting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LaneCounts {
    pub identity: u8,
    pub device: u8,
    pub timelock: u8,
    pub stake_lock: u8,
    pub role: u8,
    pub dao: u8,
}

impl LaneCounts {
    /// Returns the count for a given lane.
    #[must_use]
    pub const fn get(&self, lane: ApprovalLane) -> u8 {
        match lane {
            ApprovalLane::IdentitySig => self.identity,
            ApprovalLane::DeviceSig => self.device,
            ApprovalLane::TimeLockSig => self.timelock,
            ApprovalLane::StakeLock => self.stake_lock,
            ApprovalLane::RoleProof => self.role,
            ApprovalLane::DaoCosign => self.dao,
        }
    }

    /// Increments the count for a given lane using saturating arithmetic.
    pub fn increment(&mut self, lane: ApprovalLane) {
        match lane {
            ApprovalLane::IdentitySig => self.identity = self.identity.saturating_add(1),
            ApprovalLane::DeviceSig => self.device = self.device.saturating_add(1),
            ApprovalLane::TimeLockSig => self.timelock = self.timelock.saturating_add(1),
            ApprovalLane::StakeLock => self.stake_lock = self.stake_lock.saturating_add(1),
            ApprovalLane::RoleProof => self.role = self.role.saturating_add(1),
            ApprovalLane::DaoCosign => self.dao = self.dao.saturating_add(1),
        }
    }
}

/// HexaQuorum error surface.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum HexaQuorumError {
    EmptyActorId,
    InvalidActorId,
    MissingSignatureForSignedLane,
    InvalidSignatureFormat,
    DuplicateActorLaneProof,
    InvalidWeight,
    InvalidTimestamp,
    InvalidStake,
    InvalidPolicy,
}

impl HexaQuorumError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyActorId => "HEXA_QUORUM_EMPTY_ACTOR_ID",
            Self::InvalidActorId => "HEXA_QUORUM_INVALID_ACTOR_ID",
            Self::MissingSignatureForSignedLane => "HEXA_QUORUM_MISSING_SIGNATURE_FOR_SIGNED_LANE",
            Self::InvalidSignatureFormat => "HEXA_QUORUM_INVALID_SIGNATURE_FORMAT",
            Self::DuplicateActorLaneProof => "HEXA_QUORUM_DUPLICATE_ACTOR_LANE_PROOF",
            Self::InvalidWeight => "HEXA_QUORUM_INVALID_WEIGHT",
            Self::InvalidTimestamp => "HEXA_QUORUM_INVALID_TIMESTAMP",
            Self::InvalidStake => "HEXA_QUORUM_INVALID_STAKE",
            Self::InvalidPolicy => "HEXA_QUORUM_INVALID_POLICY",
        }
    }
}

impl fmt::Display for HexaQuorumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyActorId => {
                write!(
                    f,
                    "hexa quorum proof validation failed: actor_id must not be empty"
                )
            }
            Self::InvalidActorId => {
                write!(
                    f,
                    "hexa quorum proof validation failed: actor_id is not canonical"
                )
            }
            Self::MissingSignatureForSignedLane => {
                write!(
                    f,
                    "hexa quorum proof validation failed: signature is required for this lane"
                )
            }
            Self::InvalidSignatureFormat => {
                write!(
                    f,
                    "hexa quorum proof validation failed: signature format is invalid"
                )
            }
            Self::DuplicateActorLaneProof => {
                write!(
                    f,
                    "hexa quorum proof validation failed: duplicate actor proof in the same lane"
                )
            }
            Self::InvalidWeight => {
                write!(
                    f,
                    "hexa quorum proof validation failed: proof weight must be greater than zero"
                )
            }
            Self::InvalidTimestamp => {
                write!(
                    f,
                    "hexa quorum proof validation failed: timestamp must be greater than zero"
                )
            }
            Self::InvalidStake => {
                write!(
                    f,
                    "hexa quorum proof validation failed: stake value is invalid for this lane"
                )
            }
            Self::InvalidPolicy => {
                write!(f, "hexa quorum policy validation failed")
            }
        }
    }
}

impl std::error::Error for HexaQuorumError {}

/// Canonical HexaQuorum engine.
///
/// This engine evaluates a collection of proofs against a multi-axis trust policy.
#[derive(Debug, Clone, Default)]
pub struct HexaQuorum {
    proofs: Vec<ApprovalProof>,
}


use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Canonical proof lanes used by the AOXC HexaQuorum engine.
///
/// Security rationale:
/// Each lane represents a distinct trust axis rather than a repeated signature.
/// The goal is to prevent false confidence created by multiple signatures from
/// the same trust source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    MissingSignatureForSignedLane,
    DuplicateActorLaneProof,
    InvalidWeight,
}

impl HexaQuorumError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyActorId => "HEXA_QUORUM_EMPTY_ACTOR_ID",
            Self::MissingSignatureForSignedLane => "HEXA_QUORUM_MISSING_SIGNATURE_FOR_SIGNED_LANE",
            Self::DuplicateActorLaneProof => "HEXA_QUORUM_DUPLICATE_ACTOR_LANE_PROOF",
            Self::InvalidWeight => "HEXA_QUORUM_INVALID_WEIGHT",
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
            Self::MissingSignatureForSignedLane => {
                write!(
                    f,
                    "hexa quorum proof validation failed: signature is required for this lane"
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

impl HexaQuorum {
    /// Creates an empty HexaQuorum accumulator.
    #[must_use]
    pub fn new() -> Self {
        Self { proofs: Vec::new() }
    }

    /// Returns the number of stored proofs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.proofs.len()
    }

    /// Returns true if no proofs have been added.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.proofs.is_empty()
    }

    /// Returns all stored proofs.
    #[must_use]
    pub fn proofs(&self) -> &[ApprovalProof] {
        &self.proofs
    }

    /// Adds a proof after validating its local invariants.
    ///
    /// Duplicate proofs from the same actor in the same lane are rejected.
    pub fn add_proof(&mut self, proof: ApprovalProof) -> Result<(), HexaQuorumError> {
        validate_proof(&proof)?;

        if self
            .proofs
            .iter()
            .any(|existing| existing.actor_id == proof.actor_id && existing.lane == proof.lane)
        {
            return Err(HexaQuorumError::DuplicateActorLaneProof);
        }

        self.proofs.push(proof);
        Ok(())
    }

    /// Adds multiple proofs atomically.
    pub fn add_proofs<I>(&mut self, proofs: I) -> Result<(), HexaQuorumError>
    where
        I: IntoIterator<Item = ApprovalProof>,
    {
        for proof in proofs {
            self.add_proof(proof)?;
        }

        Ok(())
    }

    /// Clears all accumulated proofs.
    pub fn clear(&mut self) {
        self.proofs.clear();
    }

    /// Evaluates the current proof basket against the supplied policy.
    #[must_use]
    pub fn evaluate(&self, policy: &QuorumPolicy) -> QuorumResult {
        let mut lane_counts = LaneCounts::default();
        let mut missing_lanes = Vec::new();
        let mut rejection_reasons = Vec::new();

        let mut distinct_actors: HashSet<&str> = HashSet::new();
        let mut unique_stake_actors: HashMap<&str, u128> = HashMap::new();

        for proof in &self.proofs {
            distinct_actors.insert(proof.actor_id.as_str());
            lane_counts.increment(proof.lane);

            unique_stake_actors
                .entry(proof.actor_id.as_str())
                .and_modify(|current| {
                    if proof.stake > *current {
                        *current = proof.stake;
                    }
                })
                .or_insert(proof.stake);
        }

        let total_score = self
            .proofs
            .iter()
            .fold(0u64, |acc, proof| acc.saturating_add(proof.weight));

        let total_stake = unique_stake_actors
            .values()
            .fold(0u128, |acc, stake| acc.saturating_add(*stake));

        for lane in [
            ApprovalLane::IdentitySig,
            ApprovalLane::DeviceSig,
            ApprovalLane::TimeLockSig,
            ApprovalLane::StakeLock,
            ApprovalLane::RoleProof,
            ApprovalLane::DaoCosign,
        ] {
            let observed = lane_counts.get(lane);
            let required = policy.min_for_lane(lane);

            if observed < required {
                missing_lanes.push(lane);
                rejection_reasons.push(format!(
                    "lane {} below threshold: observed {} required {}",
                    lane.code(),
                    observed,
                    required
                ));
            }
        }

        if distinct_actors.len() < policy.min_distinct_actors as usize {
            rejection_reasons.push(format!(
                "distinct actor threshold not met: observed {} required {}",
                distinct_actors.len(),
                policy.min_distinct_actors
            ));
        }

        if total_stake < policy.min_total_stake {
            rejection_reasons.push(format!(
                "total stake threshold not met: observed {} required {}",
                total_stake, policy.min_total_stake
            ));
        }

        if total_score < policy.min_total_score {
            rejection_reasons.push(format!(
                "total score threshold not met: observed {} required {}",
                total_score, policy.min_total_score
            ));
        }

        let passed = rejection_reasons.is_empty();

        QuorumResult {
            passed,
            distinct_actors: distinct_actors.len(),
            total_score,
            total_stake,
            lane_counts,
            missing_lanes,
            rejection_reasons,
        }
    }
}

/// Validates a single proof before it is added to the accumulator.
fn validate_proof(proof: &ApprovalProof) -> Result<(), HexaQuorumError> {
    if proof.actor_id.trim().is_empty() {
        return Err(HexaQuorumError::EmptyActorId);
    }

    if proof.weight == 0 {
        return Err(HexaQuorumError::InvalidWeight);
    }

    if lane_requires_signature(proof.lane)
        && proof
            .signature
            .as_ref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
    {
        return Err(HexaQuorumError::MissingSignatureForSignedLane);
    }

    Ok(())
}

/// Returns true when a lane requires a cryptographic signature payload.
#[must_use]
const fn lane_requires_signature(lane: ApprovalLane) -> bool {
    matches!(
        lane,
        ApprovalLane::IdentitySig
            | ApprovalLane::DeviceSig
            | ApprovalLane::TimeLockSig
            | ApprovalLane::DaoCosign
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proof(
        actor_id: &str,
        lane: ApprovalLane,
        weight: u64,
        stake: u128,
        with_sig: bool,
    ) -> ApprovalProof {
        ApprovalProof {
            actor_id: actor_id.to_string(),
            lane,
            signature: if with_sig {
                Some("ABCDEF".to_string())
            } else {
                None
            },
            timestamp: 1_700_000_000,
            weight,
            stake,
        }
    }

    #[test]
    fn duplicate_actor_same_lane_is_rejected() {
        let mut quorum = HexaQuorum::new();

        quorum
            .add_proof(proof("actor-1", ApprovalLane::IdentitySig, 5, 100, true))
            .expect("first proof must succeed");

        let result = quorum.add_proof(proof("actor-1", ApprovalLane::IdentitySig, 5, 100, true));

        assert_eq!(result, Err(HexaQuorumError::DuplicateActorLaneProof));
    }

    #[test]
    fn missing_signature_for_signed_lane_is_rejected() {
        let mut quorum = HexaQuorum::new();

        let result = quorum.add_proof(proof("actor-1", ApprovalLane::IdentitySig, 5, 100, false));

        assert_eq!(result, Err(HexaQuorumError::MissingSignatureForSignedLane));
    }

    #[test]
    fn valid_policy_passes_with_sufficient_multiaxis_proofs() {
        let mut quorum = HexaQuorum::new();
        let policy = QuorumPolicy::strict_default();

        quorum
            .add_proofs([
                proof("actor-1", ApprovalLane::IdentitySig, 3, 100, true),
                proof("actor-2", ApprovalLane::IdentitySig, 3, 200, true),
                proof("actor-1", ApprovalLane::DeviceSig, 1, 100, true),
                proof("actor-2", ApprovalLane::TimeLockSig, 1, 200, true),
                proof("actor-1", ApprovalLane::StakeLock, 2, 100, false),
                proof("actor-2", ApprovalLane::RoleProof, 2, 200, false),
                proof("dao-1", ApprovalLane::DaoCosign, 4, 0, true),
            ])
            .expect("proof basket must be valid");

        let result = quorum.evaluate(&policy);

        assert!(result.passed);
        assert!(result.rejection_reasons.is_empty());
        assert_eq!(result.distinct_actors, 3);
        assert_eq!(result.total_stake, 300);
    }

    #[test]
    fn policy_fails_when_lanes_are_missing() {
        let mut quorum = HexaQuorum::new();
        let policy = QuorumPolicy::strict_default();

        quorum
            .add_proof(proof("actor-1", ApprovalLane::IdentitySig, 5, 100, true))
            .expect("proof must succeed");

        let result = quorum.evaluate(&policy);

        assert!(!result.passed);
        assert!(!result.missing_lanes.is_empty());
        assert!(!result.rejection_reasons.is_empty());
    }

    #[test]
    fn stake_is_counted_once_per_actor_using_max_observed_value() {
        let mut quorum = HexaQuorum::new();
        let policy = QuorumPolicy {
            min_identity: 1,
            min_device: 1,
            min_timelock: 0,
            min_stake_lock: 1,
            min_role: 0,
            min_dao: 0,
            min_distinct_actors: 1,
            min_total_stake: 100,
            min_total_score: 3,
        };

        quorum
            .add_proofs([
                proof("actor-1", ApprovalLane::IdentitySig, 1, 50, true),
                proof("actor-1", ApprovalLane::DeviceSig, 1, 80, true),
                proof("actor-1", ApprovalLane::StakeLock, 1, 120, false),
            ])
            .expect("proof basket must be valid");

        let result = quorum.evaluate(&policy);

        assert!(result.passed);
        assert_eq!(result.total_stake, 120);
    }
}

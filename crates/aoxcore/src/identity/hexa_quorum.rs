// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Maximum accepted actor identifier length inside the HexaQuorum engine.
///
/// The bound is intentionally conservative and suitable for operator,
/// validator, governance, or DAO-facing actor identifiers.
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
    ///
    /// Atomicity policy:
    /// - if any proof is invalid, no proof is added,
    /// - if any duplicate `(actor_id, lane)` exists either against the current
    ///   basket or inside the supplied batch, no proof is added.
    pub fn add_proofs<I>(&mut self, proofs: I) -> Result<(), HexaQuorumError>
    where
        I: IntoIterator<Item = ApprovalProof>,
    {
        let incoming: Vec<ApprovalProof> = proofs.into_iter().collect();

        let mut seen: BTreeSet<(String, ApprovalLane)> = self
            .proofs
            .iter()
            .map(|proof| (proof.actor_id.clone(), proof.lane))
            .collect();

        for proof in &incoming {
            validate_proof(proof)?;

            let key = (proof.actor_id.clone(), proof.lane);
            if !seen.insert(key) {
                return Err(HexaQuorumError::DuplicateActorLaneProof);
            }
        }

        self.proofs.extend(incoming);
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

        let mut distinct_actors: BTreeSet<&str> = BTreeSet::new();
        let mut unique_stake_actors: BTreeMap<&str, u128> = BTreeMap::new();

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

        if let Err(error) = policy.validate() {
            rejection_reasons.push(format!("policy invalid: {}", error.code()));

            return QuorumResult {
                passed: false,
                distinct_actors: distinct_actors.len(),
                total_score,
                total_stake,
                lane_counts,
                missing_lanes,
                rejection_reasons,
            };
        }

        for lane in ApprovalLane::all() {
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
    validate_actor_id(&proof.actor_id)?;

    if proof.timestamp == 0 {
        return Err(HexaQuorumError::InvalidTimestamp);
    }

    if proof.weight == 0 {
        return Err(HexaQuorumError::InvalidWeight);
    }

    if lane_requires_signature(proof.lane) {
        let signature = proof
            .signature
            .as_ref()
            .ok_or(HexaQuorumError::MissingSignatureForSignedLane)?;

        validate_signature_hex(signature)?;
    }

    if proof.lane == ApprovalLane::StakeLock && proof.stake == 0 {
        return Err(HexaQuorumError::InvalidStake);
    }

    Ok(())
}

/// Validates the actor identifier accepted by the quorum engine.
///
/// Compatibility note:
/// This module preserves a bounded string-based actor identifier contract
/// rather than importing a stricter actor-id parser directly.
fn validate_actor_id(actor_id: &str) -> Result<(), HexaQuorumError> {
    if actor_id.is_empty() || actor_id.trim().is_empty() {
        return Err(HexaQuorumError::EmptyActorId);
    }

    if actor_id != actor_id.trim() {
        return Err(HexaQuorumError::InvalidActorId);
    }

    if actor_id.len() > MAX_ACTOR_ID_LEN {
        return Err(HexaQuorumError::InvalidActorId);
    }

    if !actor_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(HexaQuorumError::InvalidActorId);
    }

    Ok(())
}

/// Validates a signature payload as bounded hexadecimal text.
fn validate_signature_hex(signature: &str) -> Result<(), HexaQuorumError> {
    if signature.is_empty() || signature.trim().is_empty() {
        return Err(HexaQuorumError::MissingSignatureForSignedLane);
    }

    if signature != signature.trim() {
        return Err(HexaQuorumError::InvalidSignatureFormat);
    }

    if signature.len() > MAX_SIGNATURE_HEX_LEN || !signature.len().is_multiple_of(2) {
        return Err(HexaQuorumError::InvalidSignatureFormat);
    }

    if !signature.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(HexaQuorumError::InvalidSignatureFormat);
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
                Some("ABCDEF12".to_string())
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
    fn invalid_signature_format_is_rejected() {
        let mut quorum = HexaQuorum::new();

        let result = quorum.add_proof(ApprovalProof {
            actor_id: "actor-1".to_string(),
            lane: ApprovalLane::IdentitySig,
            signature: Some("NOT_HEX!".to_string()),
            timestamp: 1_700_000_000,
            weight: 5,
            stake: 100,
        });

        assert_eq!(result, Err(HexaQuorumError::InvalidSignatureFormat));
    }

    #[test]
    fn zero_timestamp_is_rejected() {
        let mut quorum = HexaQuorum::new();

        let result = quorum.add_proof(ApprovalProof {
            actor_id: "actor-1".to_string(),
            lane: ApprovalLane::StakeLock,
            signature: None,
            timestamp: 0,
            weight: 5,
            stake: 100,
        });

        assert_eq!(result, Err(HexaQuorumError::InvalidTimestamp));
    }

    #[test]
    fn zero_stake_for_stake_lane_is_rejected() {
        let mut quorum = HexaQuorum::new();

        let result = quorum.add_proof(ApprovalProof {
            actor_id: "actor-1".to_string(),
            lane: ApprovalLane::StakeLock,
            signature: None,
            timestamp: 1_700_000_000,
            weight: 5,
            stake: 0,
        });

        assert_eq!(result, Err(HexaQuorumError::InvalidStake));
    }

    #[test]
    fn add_proofs_is_atomic() {
        let mut quorum = HexaQuorum::new();

        let result = quorum.add_proofs([
            proof("actor-1", ApprovalLane::IdentitySig, 5, 100, true),
            proof("actor-1", ApprovalLane::IdentitySig, 5, 100, true),
        ]);

        assert_eq!(result, Err(HexaQuorumError::DuplicateActorLaneProof));
        assert_eq!(quorum.len(), 0);
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

    #[test]
    fn invalid_policy_is_reported_as_rejection() {
        let quorum = HexaQuorum::new();
        let policy = QuorumPolicy {
            min_identity: 0,
            min_device: 0,
            min_timelock: 0,
            min_stake_lock: 0,
            min_role: 0,
            min_dao: 0,
            min_distinct_actors: 0,
            min_total_stake: 0,
            min_total_score: 0,
        };

        let result = quorum.evaluate(&policy);

        assert!(!result.passed);
        assert_eq!(
            result.rejection_reasons,
            vec!["policy invalid: HEXA_QUORUM_INVALID_POLICY".to_string()]
        );
    }

    #[test]
    fn lane_counts_are_recorded_correctly() {
        let mut quorum = HexaQuorum::new();
        let policy = QuorumPolicy {
            min_identity: 1,
            min_device: 1,
            min_timelock: 0,
            min_stake_lock: 0,
            min_role: 0,
            min_dao: 0,
            min_distinct_actors: 1,
            min_total_stake: 0,
            min_total_score: 2,
        };

        quorum
            .add_proofs([
                proof("actor-1", ApprovalLane::IdentitySig, 1, 0, true),
                proof("actor-1", ApprovalLane::DeviceSig, 1, 0, true),
            ])
            .expect("proof basket must be valid");

        let result = quorum.evaluate(&policy);

        assert!(result.passed);
        assert_eq!(result.lane_counts.identity, 1);
        assert_eq!(result.lane_counts.device, 1);
        assert_eq!(result.lane_counts.timelock, 0);
    }
}

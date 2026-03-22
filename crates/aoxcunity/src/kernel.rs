use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::block::Block;
use crate::constitutional::{
    ConstitutionalSeal, ContinuityCertificate, ExecutionCertificate, LegitimacyCertificate,
};
use crate::error::ConsensusError;
use crate::safety::{
    JustificationRef, LockState, SafeToVote, SafetyViolation, evaluate_safe_to_vote,
};
use crate::seal::QuorumCertificate;
use crate::state::ConsensusState;
use crate::store::ConsensusEvidence;
use crate::validator::ValidatorId;
use crate::vote::{Vote, VoteKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedVote {
    pub vote: Vote,
    pub verification_tag: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutVote {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub epoch: u64,
    pub timeout_round: u64,
    pub voter: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedTimeoutVote {
    pub timeout_vote: TimeoutVote,
    pub verification_tag: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusEvent {
    AdmitBlock(Block),
    AdmitVerifiedVote(VerifiedVote),
    AdmitTimeoutVote(VerifiedTimeoutVote),
    ObserveLegitimacy(LegitimacyCertificate),
    AdvanceRound { height: u64, round: u64 },
    EvaluateFinality { block_hash: [u8; 32] },
    PruneFinalizedState { finalized_height: u64 },
    RecoverPersistedEvent { event_hash: [u8; 32] },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelCertificate {
    Execution(QuorumCertificate),
    Legitimacy(LegitimacyCertificate),
    Continuity(ContinuityCertificate),
    Constitutional(ConstitutionalSeal),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelEffect {
    BlockAccepted([u8; 32]),
    VoteAccepted([u8; 32]),
    TimeoutAccepted([u8; 32]),
    BlockFinalized([u8; 32]),
    RoundAdvanced { height: u64, round: u64 },
    StateRecovered([u8; 32]),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KernelRejection {
    UnknownParent,
    DuplicateArtifact,
    InvalidSignature,
    StaleArtifact,
    FinalityConflict,
    InvariantViolation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PruningAction {
    pub pruned_blocks: usize,
    pub pruned_votes: usize,
    pub pruned_timeouts: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvariantStatus {
    pub conflicting_finality_detected: bool,
    pub stale_branch_reactivated: bool,
    pub replay_diverged: bool,
}

impl InvariantStatus {
    #[must_use]
    pub const fn healthy() -> Self {
        Self {
            conflicting_finality_detected: false,
            stale_branch_reactivated: false,
            replay_diverged: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionResult {
    pub accepted_effects: Vec<KernelEffect>,
    pub rejected_reason: Option<KernelRejection>,
    pub emitted_certificates: Vec<KernelCertificate>,
    pub pruning_actions: Vec<PruningAction>,
    pub invariant_status: InvariantStatus,
}

impl TransitionResult {
    #[must_use]
    pub fn accepted(effect: KernelEffect) -> Self {
        Self {
            accepted_effects: vec![effect],
            rejected_reason: None,
            emitted_certificates: Vec::new(),
            pruning_actions: Vec::new(),
            invariant_status: InvariantStatus::healthy(),
        }
    }

    #[must_use]
    pub fn rejected(reason: KernelRejection) -> Self {
        Self {
            accepted_effects: Vec::new(),
            rejected_reason: Some(reason),
            emitted_certificates: Vec::new(),
            pruning_actions: Vec::new(),
            invariant_status: InvariantStatus::healthy(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TimeoutVoteKey {
    block_hash: [u8; 32],
    height: u64,
    round: u64,
    epoch: u64,
    timeout_round: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TimeoutConflictKey {
    voter: ValidatorId,
    height: u64,
    round: u64,
    epoch: u64,
    timeout_round: u64,
}

#[derive(Debug, Clone)]
pub struct ConsensusEngine {
    pub state: ConsensusState,
    pub lock_state: LockState,
    pub current_epoch: u64,
    pub current_height: u64,
    pub legitimacy_by_block: BTreeMap<[u8; 32], LegitimacyCertificate>,
    pub continuity_by_block: BTreeMap<[u8; 32], ContinuityCertificate>,
    timeout_votes: BTreeMap<TimeoutVoteKey, BTreeMap<ValidatorId, VerifiedTimeoutVote>>,
    timeout_conflicts: BTreeMap<TimeoutConflictKey, [u8; 32]>,
    pub evidence_buffer: Vec<ConsensusEvidence>,
    pub replayed_event_hashes: BTreeSet<[u8; 32]>,
}

impl ConsensusEngine {
    #[must_use]
    pub fn new(state: ConsensusState) -> Self {
        Self {
            state,
            lock_state: LockState::default(),
            current_epoch: 0,
            current_height: 0,
            legitimacy_by_block: BTreeMap::new(),
            continuity_by_block: BTreeMap::new(),
            timeout_votes: BTreeMap::new(),
            timeout_conflicts: BTreeMap::new(),
            evidence_buffer: Vec::new(),
            replayed_event_hashes: BTreeSet::new(),
        }
    }

    #[must_use]
    pub fn apply_event(&mut self, event: ConsensusEvent) -> TransitionResult {
        match event {
            ConsensusEvent::AdmitBlock(block) => self.apply_admit_block(block),
            ConsensusEvent::AdmitVerifiedVote(verified_vote) => {
                self.apply_admit_verified_vote(verified_vote)
            }
            ConsensusEvent::AdmitTimeoutVote(timeout_vote) => self.apply_timeout_vote(timeout_vote),
            ConsensusEvent::ObserveLegitimacy(certificate) => {
                self.apply_legitimacy_certificate(certificate)
            }
            ConsensusEvent::AdvanceRound { height, round } => {
                self.apply_advance_round(height, round)
            }
            ConsensusEvent::EvaluateFinality { block_hash } => {
                self.apply_evaluate_finality(block_hash)
            }
            ConsensusEvent::PruneFinalizedState { finalized_height } => {
                self.apply_prune_finalized_state(finalized_height)
            }
            ConsensusEvent::RecoverPersistedEvent { event_hash } => {
                self.apply_recover_persisted_event(event_hash)
            }
        }
    }

    #[must_use]
    pub fn apply_events<I>(&mut self, events: I) -> Vec<TransitionResult>
    where
        I: IntoIterator<Item = ConsensusEvent>,
    {
        events
            .into_iter()
            .map(|event| self.apply_event(event))
            .collect()
    }

    #[must_use]
    pub fn state_fingerprint(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"AOXC_CONSENSUS_ENGINE_STATE_V1");
        hasher.update(self.current_epoch.to_le_bytes());
        hasher.update(self.current_height.to_le_bytes());
        hasher.update(self.state.round.round.to_le_bytes());

        if let Some(lock) = self.lock_state.current() {
            hasher.update([1]);
            hasher.update(lock.block_hash);
            hasher.update(lock.height.to_le_bytes());
            hasher.update(lock.round.to_le_bytes());
            hasher.update(lock.epoch.to_le_bytes());
            hasher.update(lock.certificate_hash);
        } else {
            hasher.update([0]);
        }

        let mut block_hashes: Vec<_> = self.state.blocks.keys().copied().collect();
        block_hashes.sort();
        hasher.update((block_hashes.len() as u64).to_le_bytes());
        for block_hash in block_hashes {
            let block = &self.state.blocks[&block_hash];
            hasher.update(block.hash);
            hasher.update(block.header.parent_hash);
            hasher.update(block.header.height.to_le_bytes());
            hasher.update(block.header.round.to_le_bytes());
        }

        hasher.update((self.legitimacy_by_block.len() as u64).to_le_bytes());
        for certificate in self.legitimacy_by_block.values() {
            hasher.update(certificate.certificate_hash);
        }
        hasher.update((self.continuity_by_block.len() as u64).to_le_bytes());
        for certificate in self.continuity_by_block.values() {
            hasher.update(certificate.certificate_hash);
        }

        hasher.update((self.evidence_buffer.len() as u64).to_le_bytes());
        for evidence in &self.evidence_buffer {
            hasher.update(evidence.evidence_hash);
            hasher.update(evidence.related_block_hash);
            hasher.update((evidence.reason.len() as u64).to_le_bytes());
            hasher.update(evidence.reason.as_bytes());
        }

        hasher.update((self.replayed_event_hashes.len() as u64).to_le_bytes());
        for event_hash in &self.replayed_event_hashes {
            hasher.update(event_hash);
        }

        hasher.finalize().into()
    }

    fn apply_admit_block(&mut self, block: Block) -> TransitionResult {
        let block_hash = block.hash;
        let block_height = block.header.height;
        match self.state.admit_block(block) {
            Ok(()) => {
                self.current_height = self.current_height.max(block_height);
                TransitionResult::accepted(KernelEffect::BlockAccepted(block_hash))
            }
            Err(error) => TransitionResult::rejected(map_consensus_error(&error)),
        }
    }

    fn apply_admit_verified_vote(&mut self, verified_vote: VerifiedVote) -> TransitionResult {
        let candidate = justification_from_vote(&verified_vote.vote, self.current_epoch);
        if let SafeToVote::No(violation) = evaluate_safe_to_vote(&self.lock_state, &candidate) {
            return TransitionResult::rejected(map_safety_violation(violation));
        }

        match self.state.add_vote(verified_vote.vote.clone()) {
            Ok(()) => {
                if matches!(verified_vote.vote.kind, VoteKind::Commit) {
                    self.lock_state.advance_to(candidate);
                }
                self.current_height = self.current_height.max(verified_vote.vote.height);
                TransitionResult::accepted(KernelEffect::VoteAccepted(
                    verified_vote.vote.block_hash,
                ))
            }
            Err(error) => {
                if matches!(error, ConsensusError::EquivocatingVote) {
                    self.evidence_buffer
                        .push(equivocation_evidence(verified_vote.vote.block_hash, "vote"));
                }
                TransitionResult::rejected(map_consensus_error(&error))
            }
        }
    }

    fn apply_timeout_vote(&mut self, timeout_vote: VerifiedTimeoutVote) -> TransitionResult {
        let vote = timeout_vote.timeout_vote.clone();
        if !self.state.blocks.contains_key(&vote.block_hash) {
            return TransitionResult::rejected(KernelRejection::StaleArtifact);
        }
        if !self
            .state
            .rotation
            .contains_active_vote_eligible_validator(vote.voter)
        {
            return TransitionResult::rejected(KernelRejection::InvalidSignature);
        }

        let conflict_key = TimeoutConflictKey {
            voter: vote.voter,
            height: vote.height,
            round: vote.round,
            epoch: vote.epoch,
            timeout_round: vote.timeout_round,
        };
        if let Some(existing_block_hash) = self.timeout_conflicts.get(&conflict_key) {
            if *existing_block_hash == vote.block_hash {
                return TransitionResult::rejected(KernelRejection::DuplicateArtifact);
            }
            self.evidence_buffer
                .push(equivocation_evidence(vote.block_hash, "timeout"));
            return TransitionResult::rejected(KernelRejection::InvariantViolation);
        }

        let key = TimeoutVoteKey {
            block_hash: vote.block_hash,
            height: vote.height,
            round: vote.round,
            epoch: vote.epoch,
            timeout_round: vote.timeout_round,
        };
        self.timeout_conflicts.insert(conflict_key, vote.block_hash);
        self.timeout_votes
            .entry(key)
            .or_default()
            .insert(vote.voter, timeout_vote);

        let mut result = TransitionResult::accepted(KernelEffect::TimeoutAccepted(vote.block_hash));
        if let Some(certificate) = self.maybe_build_continuity_certificate(key) {
            self.lock_state.advance_to(JustificationRef {
                block_hash: certificate.block_hash,
                height: certificate.height,
                round: certificate.timeout_round,
                epoch: certificate.epoch,
                certificate_hash: certificate.certificate_hash,
            });
            self.state
                .round
                .advance_to(certificate.timeout_round.saturating_add(1));
            self.current_epoch = self.current_epoch.max(certificate.epoch);
            self.current_height = self.current_height.max(certificate.height);
            self.continuity_by_block
                .insert(certificate.block_hash, certificate.clone());
            result.accepted_effects.push(KernelEffect::RoundAdvanced {
                height: certificate.height,
                round: self.state.round.round,
            });
            result
                .emitted_certificates
                .push(KernelCertificate::Continuity(certificate));
        }
        result
    }

    fn apply_legitimacy_certificate(
        &mut self,
        certificate: LegitimacyCertificate,
    ) -> TransitionResult {
        self.current_epoch = self.current_epoch.max(certificate.authority_epoch);
        self.legitimacy_by_block
            .insert(certificate.block_hash, certificate.clone());

        TransitionResult {
            accepted_effects: Vec::new(),
            rejected_reason: None,
            emitted_certificates: vec![KernelCertificate::Legitimacy(certificate)],
            pruning_actions: Vec::new(),
            invariant_status: InvariantStatus::healthy(),
        }
    }

    fn apply_advance_round(&mut self, height: u64, round: u64) -> TransitionResult {
        if round < self.state.round.round {
            return TransitionResult::rejected(KernelRejection::StaleArtifact);
        }

        self.current_height = self.current_height.max(height);
        self.state.round.advance_to(round);
        TransitionResult::accepted(KernelEffect::RoundAdvanced { height, round })
    }

    fn apply_evaluate_finality(&mut self, block_hash: [u8; 32]) -> TransitionResult {
        let Some(finalized_round) = self.state.finalizable_round(block_hash) else {
            return TransitionResult::rejected(KernelRejection::StaleArtifact);
        };

        let Some(seal) = self.state.try_finalize(block_hash, finalized_round) else {
            return TransitionResult::rejected(KernelRejection::FinalityConflict);
        };

        let execution = ExecutionCertificate::new(
            self.current_epoch,
            self.state.rotation.validator_set_hash(),
            seal.certificate.clone(),
        );
        let mut result = TransitionResult::accepted(KernelEffect::BlockFinalized(block_hash));
        result
            .emitted_certificates
            .push(KernelCertificate::Execution(
                execution.quorum_certificate.clone(),
            ));

        if let (Some(legitimacy), Some(continuity)) = (
            self.legitimacy_by_block.get(&block_hash),
            self.continuity_by_block.get(&block_hash),
        ) && let Some(constitutional) =
            ConstitutionalSeal::compose(&execution, legitimacy, continuity)
        {
            result
                .emitted_certificates
                .push(KernelCertificate::Constitutional(constitutional));
        }
        result
    }

    fn apply_prune_finalized_state(&mut self, finalized_height: u64) -> TransitionResult {
        let before_blocks = self.state.blocks.len();
        let before_timeouts = self.timeout_votes.len();
        let pruned_blocks = prune_state_to_height(&mut self.state, finalized_height);
        self.timeout_votes
            .retain(|key, _| key.height >= finalized_height);
        self.timeout_conflicts
            .retain(|key, _| key.height >= finalized_height);
        let pruned_timeouts = before_timeouts.saturating_sub(self.timeout_votes.len());
        let result = TransitionResult {
            accepted_effects: Vec::new(),
            rejected_reason: None,
            emitted_certificates: Vec::new(),
            pruning_actions: vec![PruningAction {
                pruned_blocks,
                pruned_votes: before_blocks
                    .saturating_sub(self.state.blocks.len())
                    .saturating_sub(pruned_blocks),
                pruned_timeouts,
            }],
            invariant_status: InvariantStatus::healthy(),
        };
        result
    }

    fn apply_recover_persisted_event(&mut self, event_hash: [u8; 32]) -> TransitionResult {
        if !self.replayed_event_hashes.insert(event_hash) {
            return TransitionResult::rejected(KernelRejection::DuplicateArtifact);
        }

        TransitionResult::accepted(KernelEffect::StateRecovered(event_hash))
    }

    fn maybe_build_continuity_certificate(
        &self,
        key: TimeoutVoteKey,
    ) -> Option<ContinuityCertificate> {
        let votes = self.timeout_votes.get(&key)?;
        let signers: Vec<ValidatorId> = votes.keys().copied().collect();
        let observed_power: u64 = signers
            .iter()
            .filter_map(|validator| self.state.rotation.eligible_voting_power_of(*validator))
            .sum();
        let total_power = self.state.rotation.total_voting_power();
        if !self.state.quorum.is_reached(observed_power, total_power) {
            return None;
        }

        Some(ContinuityCertificate::new(
            key.block_hash,
            key.height,
            key.round,
            key.epoch,
            key.timeout_round,
            observed_power,
            signers,
        ))
    }
}

fn prune_state_to_height(state: &mut ConsensusState, finalized_height: u64) -> usize {
    let before_blocks = state.blocks.len();
    state.blocks.retain(|hash, block| {
        block.header.height >= finalized_height
            || state
                .fork_choice
                .finalized_head()
                .is_some_and(|finalized| finalized == *hash)
    });
    state.vote_pool.prune_blocks(|hash| {
        state
            .blocks
            .get(&hash)
            .is_some_and(|block| block.header.height >= finalized_height)
    });
    before_blocks.saturating_sub(state.blocks.len())
}

fn map_consensus_error(error: &ConsensusError) -> KernelRejection {
    match error {
        ConsensusError::UnknownParent => KernelRejection::UnknownParent,
        ConsensusError::DuplicateBlock | ConsensusError::DuplicateVote => {
            KernelRejection::DuplicateArtifact
        }
        ConsensusError::EquivocatingVote => KernelRejection::InvariantViolation,
        ConsensusError::VoteForUnknownBlock
        | ConsensusError::StaleVote
        | ConsensusError::HeightRegression
        | ConsensusError::InvalidParentHeight
        | ConsensusError::InvalidGenesisParent => KernelRejection::StaleArtifact,
        ConsensusError::ValidatorNotFound
        | ConsensusError::InactiveValidator
        | ConsensusError::NonVotingValidator
        | ConsensusError::InvalidQuorumThreshold
        | ConsensusError::EmptyValidatorSet
        | ConsensusError::DuplicateValidator
        | ConsensusError::BlockBuild(_) => KernelRejection::InvalidSignature,
    }
}

fn map_safety_violation(violation: SafetyViolation) -> KernelRejection {
    match violation {
        SafetyViolation::LockRegression
        | SafetyViolation::EpochRegression
        | SafetyViolation::RoundRegression => KernelRejection::InvariantViolation,
    }
}

fn justification_from_vote(vote: &Vote, epoch: u64) -> JustificationRef {
    JustificationRef {
        block_hash: vote.block_hash,
        height: vote.height,
        round: vote.round,
        epoch,
        certificate_hash: vote.block_hash,
    }
}

fn equivocation_evidence(block_hash: [u8; 32], reason: &str) -> ConsensusEvidence {
    ConsensusEvidence {
        evidence_hash: block_hash,
        related_block_hash: block_hash,
        reason: format!("{reason}_equivocation"),
    }
}

#[cfg(test)]
mod tests {
    use crate::block::{Block, BlockBody, BlockBuilder};
    use crate::constitutional::LegitimacyCertificate;
    use crate::quorum::QuorumThreshold;
    use crate::rotation::ValidatorRotation;
    use crate::store::PersistedConsensusEvent;
    use crate::validator::{Validator, ValidatorRole};
    use crate::vote::{Vote, VoteKind};

    use super::{
        ConsensusEngine, ConsensusEvent, InvariantStatus, KernelCertificate, KernelEffect,
        KernelRejection, TimeoutVote, TransitionResult, VerifiedTimeoutVote, VerifiedVote,
    };

    fn validator(id: u8, power: u64) -> Validator {
        Validator::new([id; 32], power, ValidatorRole::Validator)
    }

    #[derive(Clone)]
    struct DeterministicRng(u64);

    impl DeterministicRng {
        fn new(seed: u64) -> Self {
            Self(seed)
        }

        fn next_u64(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }
    }

    fn pick_u8_inclusive(rng: &mut DeterministicRng, start: u8, end: u8) -> u8 {
        start + (rng.next_u64() % u64::from(end - start + 1)) as u8
    }

    fn pick_usize(rng: &mut DeterministicRng, upper_exclusive: usize) -> usize {
        (rng.next_u64() % upper_exclusive as u64) as usize
    }

    fn pick_u64_inclusive(rng: &mut DeterministicRng, start: u64, end: u64) -> u64 {
        start + (rng.next_u64() % (end - start + 1))
    }

    fn pick_bool(rng: &mut DeterministicRng, numerator: u64, denominator: u64) -> bool {
        (rng.next_u64() % denominator) < numerator
    }

    fn engine() -> ConsensusEngine {
        let rotation =
            ValidatorRotation::new(vec![validator(1, 4), validator(2, 3), validator(3, 3)])
                .unwrap();
        let quorum = QuorumThreshold::new(2, 3).unwrap();
        ConsensusEngine::new(crate::state::ConsensusState::new(rotation, quorum))
    }

    fn make_block(parent_hash: [u8; 32], height: u64, proposer: [u8; 32], round: u64) -> Block {
        BlockBuilder::build(
            1,
            parent_hash,
            height,
            round,
            height,
            height + 1,
            proposer,
            BlockBody::default(),
        )
        .unwrap()
    }

    fn commit_vote(voter: u8, block: &Block, round: u64) -> ConsensusEvent {
        ConsensusEvent::AdmitVerifiedVote(VerifiedVote {
            vote: Vote {
                voter: [voter; 32],
                block_hash: block.hash,
                height: block.header.height,
                round,
                kind: VoteKind::Commit,
            },
            verification_tag: [voter.wrapping_add(20); 32],
        })
    }

    #[test]
    fn accepted_transition_result_is_explicit_and_healthy() {
        let result = TransitionResult::accepted(KernelEffect::BlockAccepted([1u8; 32]));

        assert_eq!(result.accepted_effects.len(), 1);
        assert!(result.rejected_reason.is_none());
        assert_eq!(result.invariant_status, InvariantStatus::healthy());
    }

    #[test]
    fn rejected_transition_result_carries_explicit_reason() {
        let result = TransitionResult::rejected(KernelRejection::StaleArtifact);

        assert!(result.accepted_effects.is_empty());
        assert_eq!(result.rejected_reason, Some(KernelRejection::StaleArtifact));
    }

    #[test]
    fn deterministic_event_stream_produces_same_results_and_state() {
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let child = make_block(genesis.hash, 1, [2u8; 32], 1);
        let events = vec![
            ConsensusEvent::AdmitBlock(genesis.clone()),
            ConsensusEvent::AdmitBlock(child.clone()),
            commit_vote(1, &child, 1),
            commit_vote(2, &child, 1),
            commit_vote(3, &child, 1),
            ConsensusEvent::EvaluateFinality {
                block_hash: child.hash,
            },
        ];

        let mut a = engine();
        let mut b = engine();
        let results_a: Vec<_> = events
            .iter()
            .cloned()
            .map(|event| a.apply_event(event))
            .collect();
        let results_b: Vec<_> = events
            .into_iter()
            .map(|event| b.apply_event(event))
            .collect();

        assert_eq!(results_a, results_b);
        assert_eq!(
            a.state.fork_choice.finalized_head(),
            b.state.fork_choice.finalized_head()
        );
        assert_eq!(a.state.round, b.state.round);
        assert_eq!(a.lock_state, b.lock_state);
    }

    #[test]
    fn admit_block_duplicate_is_mapped_to_duplicate_artifact() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis.clone()));

        let duplicate = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        assert_eq!(
            duplicate.rejected_reason,
            Some(KernelRejection::DuplicateArtifact)
        );
    }

    #[test]
    fn admit_block_unknown_parent_is_mapped_to_unknown_parent() {
        let mut engine = engine();
        let orphan = make_block([9u8; 32], 1, [1u8; 32], 1);

        let rejected = engine.apply_event(ConsensusEvent::AdmitBlock(orphan));
        assert_eq!(
            rejected.rejected_reason,
            Some(KernelRejection::UnknownParent)
        );
    }

    #[test]
    fn admit_verified_vote_duplicate_maps_to_duplicate_artifact() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis.clone()));

        let event = commit_vote(1, &genesis, 0);
        assert_eq!(engine.apply_event(event.clone()).rejected_reason, None);
        let duplicate = engine.apply_event(event);
        assert_eq!(
            duplicate.rejected_reason,
            Some(KernelRejection::DuplicateArtifact)
        );
    }

    #[test]
    fn vote_equivocation_emits_evidence() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let a = make_block(genesis.hash, 1, [2u8; 32], 1);
        let b = make_block(genesis.hash, 1, [3u8; 32], 1);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(a.clone()));
        engine.state.blocks.insert(b.hash, b.clone());
        engine
            .state
            .fork_choice
            .insert_block(crate::fork_choice::BlockMeta {
                hash: b.hash,
                parent: b.header.parent_hash,
                height: b.header.height,
                seal: None,
            });

        let first = commit_vote(1, &a, 1);
        let second = commit_vote(1, &b, 1);
        assert_eq!(engine.apply_event(first).rejected_reason, None);
        let rejected = engine.apply_event(second);

        assert_eq!(
            rejected.rejected_reason,
            Some(KernelRejection::InvariantViolation)
        );
        assert_eq!(engine.evidence_buffer.len(), 1);
        assert_eq!(engine.evidence_buffer[0].reason, "vote_equivocation");
    }

    #[test]
    fn safety_rejection_does_not_mutate_engine_state() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let child = make_block(genesis.hash, 1, [2u8; 32], 2);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis.clone()));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(child.clone()));
        assert_eq!(
            engine
                .apply_event(commit_vote(1, &child, 2))
                .rejected_reason,
            None
        );
        let before_lock = engine.lock_state.clone();
        let before_height = engine.current_height;
        let before_votes = engine
            .state
            .vote_pool
            .count_for_block_kind(child.hash, VoteKind::Commit);

        let rejected = engine.apply_event(commit_vote(2, &genesis, 0));

        assert_eq!(
            rejected.rejected_reason,
            Some(KernelRejection::InvariantViolation)
        );
        assert_eq!(engine.lock_state, before_lock);
        assert_eq!(engine.current_height, before_height);
        assert_eq!(
            engine
                .state
                .vote_pool
                .count_for_block_kind(child.hash, VoteKind::Commit),
            before_votes
        );
    }

    #[test]
    fn timeout_quorum_emits_continuity_certificate_and_advances_round() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [2u8; 32], 1);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));

        let timeout_event = |voter: u8| {
            ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
                timeout_vote: TimeoutVote {
                    block_hash: block.hash,
                    height: 1,
                    round: 1,
                    epoch: 4,
                    timeout_round: 2,
                    voter: [voter; 32],
                },
                verification_tag: [voter.wrapping_add(10); 32],
            })
        };

        assert!(
            engine
                .apply_event(timeout_event(1))
                .emitted_certificates
                .is_empty()
        );
        let result = engine.apply_event(timeout_event(2));

        assert!(
            result
                .accepted_effects
                .contains(&KernelEffect::RoundAdvanced {
                    height: 1,
                    round: 3
                })
        );
        assert!(
            result
                .emitted_certificates
                .iter()
                .any(|certificate| matches!(certificate, KernelCertificate::Continuity(_)))
        );
        assert_eq!(engine.state.round.round, 3);
    }

    #[test]
    fn advance_round_stale_rejection_does_not_mutate_round_or_height() {
        let mut engine = engine();
        let accepted = engine.apply_event(ConsensusEvent::AdvanceRound {
            height: 5,
            round: 4,
        });
        assert_eq!(accepted.rejected_reason, None);
        let rejected = engine.apply_event(ConsensusEvent::AdvanceRound {
            height: 1,
            round: 3,
        });

        assert_eq!(
            rejected.rejected_reason,
            Some(KernelRejection::StaleArtifact)
        );
        assert_eq!(engine.state.round.round, 4);
        assert_eq!(engine.current_height, 5);
    }

    #[test]
    fn duplicate_timeout_vote_is_rejected_as_duplicate_artifact() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [2u8; 32], 1);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));
        let event = ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
            timeout_vote: TimeoutVote {
                block_hash: block.hash,
                height: 1,
                round: 1,
                epoch: 0,
                timeout_round: 2,
                voter: [1u8; 32],
            },
            verification_tag: [1u8; 32],
        });

        assert_eq!(engine.apply_event(event.clone()).rejected_reason, None);
        let duplicate = engine.apply_event(event);
        assert_eq!(
            duplicate.rejected_reason,
            Some(KernelRejection::DuplicateArtifact)
        );
    }

    #[test]
    fn unknown_timeout_block_is_rejected_as_stale_artifact() {
        let mut engine = engine();
        let rejected = engine.apply_event(ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
            timeout_vote: TimeoutVote {
                block_hash: [99u8; 32],
                height: 1,
                round: 1,
                epoch: 0,
                timeout_round: 2,
                voter: [1u8; 32],
            },
            verification_tag: [1u8; 32],
        }));

        assert_eq!(
            rejected.rejected_reason,
            Some(KernelRejection::StaleArtifact)
        );
    }

    #[test]
    fn timeout_rejects_inactive_and_non_eligible_validators() {
        let inactive_rotation = ValidatorRotation::new(vec![
            {
                let mut validator = validator(1, 5);
                validator.active = false;
                validator
            },
            validator(2, 5),
        ])
        .unwrap();
        let observer_rotation = ValidatorRotation::new(vec![
            Validator::new([1u8; 32], 5, ValidatorRole::Observer),
            validator(2, 5),
        ])
        .unwrap();

        for rotation in [inactive_rotation, observer_rotation] {
            let quorum = QuorumThreshold::new(2, 3).unwrap();
            let mut engine =
                ConsensusEngine::new(crate::state::ConsensusState::new(rotation, quorum));
            let genesis = make_block([0u8; 32], 0, [2u8; 32], 0);
            let block = make_block(genesis.hash, 1, [2u8; 32], 1);
            let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
            let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));

            let rejected =
                engine.apply_event(ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
                    timeout_vote: TimeoutVote {
                        block_hash: block.hash,
                        height: 1,
                        round: 1,
                        epoch: 0,
                        timeout_round: 2,
                        voter: [1u8; 32],
                    },
                    verification_tag: [1u8; 32],
                }));
            assert_eq!(
                rejected.rejected_reason,
                Some(KernelRejection::InvalidSignature)
            );
        }
    }

    #[test]
    fn timeout_quorum_tracks_observed_power_correctly_for_skewed_weights() {
        let rotation =
            ValidatorRotation::new(vec![validator(1, 8), validator(2, 1), validator(3, 1)])
                .unwrap();
        let quorum = QuorumThreshold::new(2, 3).unwrap();
        let mut engine = ConsensusEngine::new(crate::state::ConsensusState::new(rotation, quorum));
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [1u8; 32], 1);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));

        let result = engine.apply_event(ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
            timeout_vote: TimeoutVote {
                block_hash: block.hash,
                height: 1,
                round: 1,
                epoch: 0,
                timeout_round: 2,
                voter: [1u8; 32],
            },
            verification_tag: [1u8; 32],
        }));

        let continuity = result
            .emitted_certificates
            .into_iter()
            .find_map(|certificate| {
                if let KernelCertificate::Continuity(certificate) = certificate {
                    Some(certificate)
                } else {
                    None
                }
            });
        assert_eq!(continuity.unwrap().observed_power, 8);
    }

    #[test]
    fn finality_can_emit_constitutional_seal_when_legitimacy_and_continuity_exist() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [2u8; 32], 1);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));

        let _ = engine.apply_event(ConsensusEvent::ObserveLegitimacy(
            LegitimacyCertificate::new(
                block.hash,
                0,
                [1u8; 32],
                [2u8; 32],
                [3u8; 32],
                vec![[1u8; 32], [2u8; 32], [3u8; 32]],
            ),
        ));
        for voter in [1u8, 2u8, 3u8] {
            let _ = engine.apply_event(ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
                timeout_vote: TimeoutVote {
                    block_hash: block.hash,
                    height: 1,
                    round: 1,
                    epoch: 0,
                    timeout_round: 1,
                    voter: [voter; 32],
                },
                verification_tag: [voter; 32],
            }));
        }
        for voter in [1u8, 2u8, 3u8] {
            let _ = engine.apply_event(commit_vote(voter, &block, 1));
        }

        let result = engine.apply_event(ConsensusEvent::EvaluateFinality {
            block_hash: block.hash,
        });

        assert!(
            result
                .emitted_certificates
                .iter()
                .any(|certificate| matches!(certificate, KernelCertificate::Execution(_)))
        );
        assert!(
            result
                .emitted_certificates
                .iter()
                .any(|certificate| matches!(certificate, KernelCertificate::Constitutional(_)))
        );
        assert_eq!(
            result.accepted_effects,
            vec![KernelEffect::BlockFinalized(block.hash)]
        );
    }

    #[test]
    fn evaluate_finality_distinguishes_stale_from_conflicting_finality() {
        let mut engine = engine();
        let stale = engine.apply_event(ConsensusEvent::EvaluateFinality {
            block_hash: [7u8; 32],
        });
        assert_eq!(stale.rejected_reason, Some(KernelRejection::StaleArtifact));

        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let a = make_block(genesis.hash, 1, [2u8; 32], 1);
        let b = make_block(genesis.hash, 1, [3u8; 32], 1);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(a.clone()));
        engine.state.blocks.insert(b.hash, b.clone());
        engine
            .state
            .fork_choice
            .insert_block(crate::fork_choice::BlockMeta {
                hash: b.hash,
                parent: b.header.parent_hash,
                height: b.header.height,
                seal: None,
            });
        for voter in [1u8, 2u8, 3u8] {
            let _ = engine.apply_event(commit_vote(voter, &a, 1));
        }
        assert_eq!(
            engine
                .apply_event(ConsensusEvent::EvaluateFinality { block_hash: a.hash })
                .rejected_reason,
            None
        );
        engine.state.blocks.insert(b.hash, b.clone());
        engine
            .state
            .fork_choice
            .insert_block(crate::fork_choice::BlockMeta {
                hash: b.hash,
                parent: b.header.parent_hash,
                height: b.header.height,
                seal: None,
            });
        for voter in [1u8, 2u8, 3u8] {
            engine
                .state
                .vote_pool
                .add_vote(Vote {
                    voter: [voter; 32],
                    block_hash: b.hash,
                    height: 1,
                    round: 2,
                    kind: VoteKind::Commit,
                })
                .unwrap();
        }
        let conflicting =
            engine.apply_event(ConsensusEvent::EvaluateFinality { block_hash: b.hash });
        assert_eq!(
            conflicting.rejected_reason,
            Some(KernelRejection::FinalityConflict)
        );
    }

    #[test]
    fn finality_without_legitimacy_or_continuity_emits_only_execution_certificate() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [2u8; 32], 1);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));
        for voter in [1u8, 2u8, 3u8] {
            let _ = engine.apply_event(commit_vote(voter, &block, 1));
        }

        let result = engine.apply_event(ConsensusEvent::EvaluateFinality {
            block_hash: block.hash,
        });
        assert_eq!(
            result
                .emitted_certificates
                .iter()
                .filter(|certificate| matches!(certificate, KernelCertificate::Execution(_)))
                .count(),
            1
        );
        assert!(
            !result
                .emitted_certificates
                .iter()
                .any(|certificate| matches!(certificate, KernelCertificate::Constitutional(_)))
        );
    }

    #[test]
    fn weighted_quorum_can_finalize_with_power_majority_but_not_count_majority() {
        let rotation =
            ValidatorRotation::new(vec![validator(1, 8), validator(2, 1), validator(3, 1)])
                .unwrap();
        let quorum = QuorumThreshold::new(2, 3).unwrap();
        let mut engine = ConsensusEngine::new(crate::state::ConsensusState::new(rotation, quorum));

        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [1u8; 32], 1);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));

        let result = engine.apply_events(vec![commit_vote(1, &block, 1)]);
        assert!(
            result
                .iter()
                .all(|transition| transition.rejected_reason.is_none())
        );

        let finalized = engine.apply_event(ConsensusEvent::EvaluateFinality {
            block_hash: block.hash,
        });
        assert_eq!(finalized.rejected_reason, None);
        assert_eq!(
            finalized.accepted_effects,
            vec![KernelEffect::BlockFinalized(block.hash)]
        );
    }

    #[test]
    fn count_majority_without_weight_majority_cannot_finalize() {
        let rotation =
            ValidatorRotation::new(vec![validator(1, 6), validator(2, 2), validator(3, 2)])
                .unwrap();
        let quorum = QuorumThreshold::new(2, 3).unwrap();
        let mut engine = ConsensusEngine::new(crate::state::ConsensusState::new(rotation, quorum));

        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [1u8; 32], 1);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));
        let _ = engine.apply_events(vec![commit_vote(2, &block, 1), commit_vote(3, &block, 1)]);

        let finalized = engine.apply_event(ConsensusEvent::EvaluateFinality {
            block_hash: block.hash,
        });
        assert_eq!(
            finalized.rejected_reason,
            Some(KernelRejection::StaleArtifact)
        );
    }

    #[test]
    fn timeout_equivocation_is_rejected_and_recorded_as_evidence() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let a = make_block(genesis.hash, 1, [2u8; 32], 1);
        let b = make_block(genesis.hash, 1, [3u8; 32], 1);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(a.clone()));
        engine.state.blocks.insert(b.hash, b.clone());
        engine
            .state
            .fork_choice
            .insert_block(crate::fork_choice::BlockMeta {
                hash: b.hash,
                parent: b.header.parent_hash,
                height: b.header.height,
                seal: None,
            });

        let timeout = |block_hash| {
            ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
                timeout_vote: TimeoutVote {
                    block_hash,
                    height: 1,
                    round: 1,
                    epoch: 0,
                    timeout_round: 2,
                    voter: [1u8; 32],
                },
                verification_tag: [9u8; 32],
            })
        };

        assert_eq!(engine.apply_event(timeout(a.hash)).rejected_reason, None);
        let rejected = engine.apply_event(timeout(b.hash));
        assert_eq!(
            rejected.rejected_reason,
            Some(KernelRejection::InvariantViolation)
        );
        assert_eq!(engine.evidence_buffer.len(), 1);
        assert_eq!(engine.evidence_buffer[0].reason, "timeout_equivocation");
    }

    #[test]
    fn duplicate_recover_persisted_event_is_rejected() {
        let mut engine = engine();
        let first = engine.apply_event(ConsensusEvent::RecoverPersistedEvent {
            event_hash: [1u8; 32],
        });
        let duplicate = engine.apply_event(ConsensusEvent::RecoverPersistedEvent {
            event_hash: [1u8; 32],
        });

        assert_eq!(first.rejected_reason, None);
        assert_eq!(
            duplicate.rejected_reason,
            Some(KernelRejection::DuplicateArtifact)
        );
    }

    #[test]
    fn prune_finalized_state_prunes_timeout_tracking() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [2u8; 32], 1);
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block.clone()));
        let _ = engine.apply_event(ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
            timeout_vote: TimeoutVote {
                block_hash: block.hash,
                height: 1,
                round: 1,
                epoch: 0,
                timeout_round: 2,
                voter: [1u8; 32],
            },
            verification_tag: [1u8; 32],
        }));

        let result = engine.apply_event(ConsensusEvent::PruneFinalizedState {
            finalized_height: 2,
        });
        let action = result.pruning_actions.first().unwrap();
        assert_eq!(action.pruned_timeouts, 1);
        assert_eq!(engine.timeout_votes.len(), 0);
        assert_eq!(engine.timeout_conflicts.len(), 0);
    }

    #[test]
    fn persisted_event_replay_matches_live_state_fingerprint() {
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block = make_block(genesis.hash, 1, [2u8; 32], 1);
        let events = vec![
            PersistedConsensusEvent {
                sequence: 1,
                event_hash: [1u8; 32],
                event: ConsensusEvent::AdmitBlock(genesis),
            },
            PersistedConsensusEvent {
                sequence: 2,
                event_hash: [2u8; 32],
                event: ConsensusEvent::AdmitBlock(block.clone()),
            },
            PersistedConsensusEvent {
                sequence: 3,
                event_hash: [3u8; 32],
                event: commit_vote(1, &block, 1),
            },
            PersistedConsensusEvent {
                sequence: 4,
                event_hash: [4u8; 32],
                event: commit_vote(2, &block, 1),
            },
            PersistedConsensusEvent {
                sequence: 5,
                event_hash: [5u8; 32],
                event: commit_vote(3, &block, 1),
            },
            PersistedConsensusEvent {
                sequence: 6,
                event_hash: [6u8; 32],
                event: ConsensusEvent::RecoverPersistedEvent {
                    event_hash: [6u8; 32],
                },
            },
            PersistedConsensusEvent {
                sequence: 7,
                event_hash: [7u8; 32],
                event: ConsensusEvent::EvaluateFinality {
                    block_hash: block.hash,
                },
            },
        ];

        let mut live = engine();
        let mut replay = engine();
        let live_results = live.apply_events(events.iter().cloned().map(|entry| entry.event));
        let replay_results = replay.apply_events(
            events
                .into_iter()
                .map(|entry| entry.event)
                .collect::<Vec<_>>(),
        );

        assert_eq!(live_results, replay_results);
        assert_eq!(live.state_fingerprint(), replay.state_fingerprint());
    }

    #[test]
    fn seeded_random_event_stream_is_deterministic_across_engines() {
        let mut engine_a = engine();
        let mut engine_b = engine();
        let mut rng = DeterministicRng::new(0xA0C2_2026);

        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let mut chain = vec![genesis.clone()];
        let mut events = vec![ConsensusEvent::AdmitBlock(genesis)];
        let mut parent = chain[0].hash;

        for height in 1..=6 {
            let proposer = [pick_u8_inclusive(&mut rng, 1, 3); 32];
            let round = pick_u64_inclusive(&mut rng, 1, 4);
            let block = make_block(parent, height, proposer, round);
            parent = block.hash;
            chain.push(block.clone());
            events.push(ConsensusEvent::AdmitBlock(block.clone()));

            for _ in 0..8 {
                let block = &chain[pick_usize(&mut rng, chain.len())];
                let voter = pick_u8_inclusive(&mut rng, 1, 3);
                let vote_round = pick_u64_inclusive(&mut rng, 0, block.header.round.max(1));
                if pick_bool(&mut rng, 3, 4) {
                    events.push(ConsensusEvent::AdmitVerifiedVote(VerifiedVote {
                        vote: Vote {
                            voter: [voter; 32],
                            block_hash: block.hash,
                            height: block.header.height,
                            round: vote_round,
                            kind: if pick_bool(&mut rng, 1, 2) {
                                VoteKind::Prepare
                            } else {
                                VoteKind::Commit
                            },
                        },
                        verification_tag: [voter; 32],
                    }));
                } else {
                    events.push(ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
                        timeout_vote: TimeoutVote {
                            block_hash: block.hash,
                            height: block.header.height,
                            round: vote_round,
                            epoch: 0,
                            timeout_round: vote_round.saturating_add(1),
                            voter: [voter; 32],
                        },
                        verification_tag: [voter.wrapping_add(30); 32],
                    }));
                }
            }

            events.push(ConsensusEvent::EvaluateFinality {
                block_hash: chain[pick_usize(&mut rng, chain.len())].hash,
            });
        }

        let results_a = engine_a.apply_events(events.clone());
        let results_b = engine_b.apply_events(events);

        assert_eq!(results_a, results_b);
        assert_eq!(engine_a.state_fingerprint(), engine_b.state_fingerprint());
    }
}

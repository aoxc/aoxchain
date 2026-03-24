use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::block::Block;
use crate::constitutional::{
    ConstitutionalSeal, ContinuityCertificate, ExecutionCertificate, LegitimacyCertificate,
};
use crate::error::ConsensusError;
use crate::safety::{
    JustificationRef, LockState, SafeToVote, SafetyViolation, evaluate_safe_to_vote,
};
use crate::seal::AuthenticatedQuorumCertificate;
use crate::state::ConsensusState;
use crate::store::ConsensusEvidence;
use crate::validator::ValidatorId;
use crate::vote::{VerifiedAuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerifiedVote {
    pub authenticated_vote: VerifiedAuthenticatedVote,
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
#[allow(clippy::large_enum_variant)]
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
    Execution(AuthenticatedQuorumCertificate),
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

    #[must_use]
    fn with_conflicting_finality_detected(mut self) -> Self {
        self.invariant_status.conflicting_finality_detected = true;
        self
    }

    #[must_use]
    fn with_stale_branch_reactivated(mut self) -> Self {
        self.invariant_status.stale_branch_reactivated = true;
        self
    }

    #[must_use]
    fn with_replay_diverged(mut self) -> Self {
        self.invariant_status.replay_diverged = true;
        self
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
    pub network_id: u32,
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
        Self::with_network_id(state, 2626)
    }

    #[must_use]
    pub fn with_network_id(state: ConsensusState, network_id: u32) -> Self {
        Self {
            state,
            network_id,
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

    fn apply_admit_block(&mut self, block: Block) -> TransitionResult {
        let block_hash = block.hash;
        let block_height = block.header.height;
        match self.state.admit_block(block) {
            Ok(()) => {
                self.current_height = self.current_height.max(block_height);
                TransitionResult::accepted(KernelEffect::BlockAccepted(block_hash))
            }
            Err(error) => {
                let result = TransitionResult::rejected(map_consensus_error(&error));
                if matches!(error, ConsensusError::HeightRegression) {
                    return result.with_stale_branch_reactivated();
                }
                result
            }
        }
    }

    fn apply_admit_verified_vote(&mut self, verified_vote: VerifiedVote) -> TransitionResult {
        let candidate = justification_from_vote(
            &verified_vote.authenticated_vote.vote,
            verified_vote.authenticated_vote.context.epoch,
        );
        if let SafeToVote::No(violation) = evaluate_safe_to_vote(&self.lock_state, &candidate) {
            return TransitionResult::rejected(map_safety_violation(violation));
        }

        let expected_context = self.vote_authentication_context();
        match self
            .state
            .add_authenticated_vote(verified_vote.authenticated_vote.clone(), expected_context)
        {
            Ok(()) => {
                if matches!(verified_vote.authenticated_vote.vote.kind, VoteKind::Commit) {
                    self.lock_state.advance_to(candidate);
                }
                self.current_epoch = self
                    .current_epoch
                    .max(verified_vote.authenticated_vote.context.epoch);
                self.current_height = self
                    .current_height
                    .max(verified_vote.authenticated_vote.vote.height);
                TransitionResult::accepted(KernelEffect::VoteAccepted(
                    verified_vote.authenticated_vote.vote.block_hash,
                ))
            }
            Err(error) => {
                if matches!(error, ConsensusError::EquivocatingVote) {
                    self.evidence_buffer.push(equivocation_evidence(
                        verified_vote.authenticated_vote.vote.block_hash,
                        "vote",
                    ));
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
            return TransitionResult::rejected(KernelRejection::InvariantViolation)
                .with_conflicting_finality_detected();
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

        let authenticated_certificate = self.state.authenticated_quorum_certificate(
            block_hash,
            finalized_round,
            self.vote_authentication_context(),
        );
        let Some(seal) = self.state.try_finalize(block_hash, finalized_round) else {
            return TransitionResult::rejected(KernelRejection::FinalityConflict)
                .with_conflicting_finality_detected();
        };

        let execution = ExecutionCertificate::new(
            self.current_epoch,
            self.state.rotation.validator_set_hash(),
            seal.certificate.clone(),
        );
        let mut result = TransitionResult::accepted(KernelEffect::BlockFinalized(block_hash));
        if let Some(certificate) = authenticated_certificate {
            result
                .emitted_certificates
                .push(KernelCertificate::Execution(certificate));
        }

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
        TransitionResult {
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
        }
    }

    fn apply_recover_persisted_event(&mut self, event_hash: [u8; 32]) -> TransitionResult {
        if !self.replayed_event_hashes.insert(event_hash) {
            return TransitionResult::rejected(KernelRejection::DuplicateArtifact)
                .with_replay_diverged();
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

    fn vote_authentication_context(&self) -> VoteAuthenticationContext {
        VoteAuthenticationContext {
            network_id: 2626,
            epoch: self.current_epoch,
            validator_set_root: self.state.rotation.validator_set_hash(),
            signature_scheme: 1,
        }
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
        | ConsensusError::InvalidAuthenticatedContext
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
    use crate::validator::{Validator, ValidatorRole};
    use crate::vote::{VerifiedAuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

    use super::{
        ConsensusEngine, ConsensusEvent, InvariantStatus, KernelCertificate, KernelEffect,
        KernelRejection, TimeoutVote, TransitionResult, VerifiedTimeoutVote, VerifiedVote,
    };

    fn validator(id: u8, power: u64) -> Validator {
        Validator::new([id; 32], power, ValidatorRole::Validator)
    }

    fn engine() -> ConsensusEngine {
        let rotation =
            ValidatorRotation::new(vec![validator(1, 4), validator(2, 3), validator(3, 3)])
                .unwrap();
        let quorum = QuorumThreshold::new(2, 3).unwrap();
        ConsensusEngine::new(crate::state::ConsensusState::new(rotation, quorum))
    }

    fn vote_context(engine: &ConsensusEngine, epoch: u64) -> VoteAuthenticationContext {
        VoteAuthenticationContext {
            network_id: 2626,
            epoch,
            validator_set_root: engine.state.rotation.validator_set_hash(),
            signature_scheme: 1,
        }
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

    fn commit_vote(
        engine: &ConsensusEngine,
        voter: u8,
        block: &Block,
        round: u64,
    ) -> ConsensusEvent {
        ConsensusEvent::AdmitVerifiedVote(VerifiedVote {
            authenticated_vote: VerifiedAuthenticatedVote {
                vote: Vote {
                    voter: [voter; 32],
                    block_hash: block.hash,
                    height: block.header.height,
                    round,
                    kind: VoteKind::Commit,
                },
                context: vote_context(engine, 0),
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
        let mut a = engine();
        let mut b = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let child = make_block(genesis.hash, 1, [2u8; 32], 1);
        let events = vec![
            ConsensusEvent::AdmitBlock(genesis.clone()),
            ConsensusEvent::AdmitBlock(child.clone()),
            commit_vote(&a, 1, &child, 1),
            commit_vote(&a, 2, &child, 1),
            commit_vote(&a, 3, &child, 1),
            ConsensusEvent::EvaluateFinality {
                block_hash: child.hash,
            },
        ];
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
            let _ = engine.apply_event(commit_vote(&engine, voter, &block, 1));
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
        let execution = result
            .emitted_certificates
            .iter()
            .find_map(|certificate| match certificate {
                KernelCertificate::Execution(certificate) => Some(certificate),
                _ => None,
            })
            .expect("authenticated execution certificate must be emitted");
        assert_eq!(execution.network_id, 2626);
        assert_eq!(execution.epoch, 0);
        assert_eq!(
            execution.validator_set_root,
            engine.state.rotation.validator_set_hash()
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
    fn duplicate_recovery_event_sets_replay_diverged_invariant() {
        let mut engine = engine();
        let event_hash = [0xAA; 32];

        let first = engine.apply_event(ConsensusEvent::RecoverPersistedEvent { event_hash });
        let second = engine.apply_event(ConsensusEvent::RecoverPersistedEvent { event_hash });

        assert_eq!(
            first.accepted_effects,
            vec![KernelEffect::StateRecovered(event_hash)]
        );
        assert_eq!(
            second.rejected_reason,
            Some(KernelRejection::DuplicateArtifact)
        );
        assert!(second.invariant_status.replay_diverged);
    }

    #[test]
    fn height_regression_marks_stale_branch_reactivation_invariant() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let canonical = make_block(genesis.hash, 1, [2u8; 32], 1);
        let conflicting = make_block(genesis.hash, 1, [3u8; 32], 1);

        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(canonical));
        let result = engine.apply_event(ConsensusEvent::AdmitBlock(conflicting));

        assert_eq!(result.rejected_reason, Some(KernelRejection::StaleArtifact));
        assert!(result.invariant_status.stale_branch_reactivated);
    }

    #[test]
    fn timeout_equivocation_sets_conflicting_finality_invariant() {
        let mut engine = engine();
        let genesis = make_block([0u8; 32], 0, [1u8; 32], 0);
        let block_a = make_block(genesis.hash, 1, [2u8; 32], 1);
        let block_b = Block {
            hash: [0xBB; 32],
            ..make_block(genesis.hash, 1, [3u8; 32], 1)
        };
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(genesis));
        let _ = engine.apply_event(ConsensusEvent::AdmitBlock(block_a.clone()));
        engine.state.blocks.insert(block_b.hash, block_b.clone());
        engine
            .state
            .fork_choice
            .insert_block(crate::fork_choice::BlockMeta {
                hash: block_b.hash,
                parent: block_b.header.parent_hash,
                height: block_b.header.height,
                seal: None,
            });

        let vote = |block_hash| {
            ConsensusEvent::AdmitTimeoutVote(VerifiedTimeoutVote {
                timeout_vote: TimeoutVote {
                    block_hash,
                    height: 1,
                    round: 1,
                    epoch: 0,
                    timeout_round: 2,
                    voter: [1u8; 32],
                },
                verification_tag: [8u8; 32],
            })
        };

        let first = engine.apply_event(vote(block_a.hash));
        let second = engine.apply_event(vote(block_b.hash));

        assert_eq!(
            first.accepted_effects,
            vec![KernelEffect::TimeoutAccepted(block_a.hash)]
        );
        assert_eq!(
            second.rejected_reason,
            Some(KernelRejection::InvariantViolation)
        );
        assert!(second.invariant_status.conflicting_finality_detected);
    }
}

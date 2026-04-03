// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::{BTreeSet, HashMap};

use crate::block::hash::{compute_block_hash, compute_body_roots};
use crate::block::semantic::{
    validate_block_semantics, validate_capability_section_alignment,
    validate_root_semantic_bindings,
};
use crate::block::{Block, BlockBody};
use crate::error::ConsensusError;
use crate::fork_choice::{BlockMeta, ForkChoice};
use crate::quorum::QuorumThreshold;
use crate::rotation::ValidatorRotation;
use crate::round::RoundState;
use crate::seal::{AuthenticatedQuorumCertificate, BlockSeal, QuorumCertificate};
use crate::validator::{SlashFault, ValidatorId};
use crate::vote::{
    SignedVote, VerifiedAuthenticatedVote, VerifiedVote, Vote, VoteAuthenticationContext,
    VoteAuthenticationError, VoteKind,
};
use crate::vote_pool::VotePool;

/// In-memory consensus state container.
///
/// # Responsibilities
/// - stores blocks admitted into the local consensus view,
/// - maintains fork-choice metadata,
/// - admits and validates votes,
/// - evaluates quorum attainment,
/// - builds deterministic finality artifacts when quorum conditions are satisfied.
///
/// # Security Note
/// This structure is intentionally deterministic and validation-oriented.
/// It does not provide durable persistence in this phase and therefore must not
/// be treated as a crash-recovery authority source.
#[derive(Debug, Clone)]
pub struct ConsensusState {
    pub fork_choice: ForkChoice,
    pub vote_pool: VotePool,
    pub round: RoundState,
    pub rotation: ValidatorRotation,
    pub quorum: QuorumThreshold,
    pub blocks: HashMap<[u8; 32], Block>,
}

impl ConsensusState {
    #[must_use]
    pub fn new(rotation: ValidatorRotation, quorum: QuorumThreshold) -> Self {
        Self {
            fork_choice: ForkChoice::new(),
            vote_pool: VotePool::new(),
            round: RoundState::new(),
            rotation,
            quorum,
            blocks: HashMap::new(),
        }
    }

    /// Admits a block into local consensus state.
    ///
    /// # Genesis Policy
    /// Only height `0` or `1` may use the zero parent hash under the explicit
    /// genesis rule. Every non-genesis block must reference an existing parent
    /// and must advance height by exactly one.
    ///
    /// # Security Semantics
    /// This method rejects:
    /// - duplicate block hashes,
    /// - invalid zero-parent usage,
    /// - unknown parents,
    /// - parent/child height discontinuity,
    /// - local head regressions that violate current fork-choice expectations.
    pub fn admit_block(&mut self, block: Block) -> Result<(), ConsensusError> {
        self.validate_block_integrity(&block)?;

        if self.fork_choice.contains(block.hash) || self.blocks.contains_key(&block.hash) {
            return Err(ConsensusError::DuplicateBlock);
        }

        if let Some(finalized_hash) = self.fork_choice.finalized_head()
            && let Some(finalized_meta) = self.fork_choice.get(finalized_hash)
            && block.header.height <= finalized_meta.height
            && block.hash != finalized_hash
        {
            return Err(ConsensusError::HeightRegression);
        }

        let is_genesis =
            matches!(block.header.height, 0 | 1) && block.header.parent_hash == [0u8; 32];

        if is_genesis {
            if !matches!(block.header.height, 0 | 1) {
                return Err(ConsensusError::InvalidGenesisParent);
            }
        } else {
            if block.header.parent_hash == [0u8; 32] {
                return Err(ConsensusError::InvalidGenesisParent);
            }

            let parent = self
                .blocks
                .get(&block.header.parent_hash)
                .ok_or(ConsensusError::UnknownParent)?;

            if block.header.height != parent.header.height + 1 {
                return Err(ConsensusError::InvalidParentHeight);
            }
        }

        if let Some(head) = self.fork_choice.get_head()
            && let Some(head_meta) = self.fork_choice.get(head)
            && block.header.height < head_meta.height
            && block.header.parent_hash == head_meta.parent
        {
            return Err(ConsensusError::HeightRegression);
        }

        self.fork_choice.insert_block(BlockMeta {
            hash: block.hash,
            parent: block.header.parent_hash,
            height: block.header.height,
            seal: None,
        });

        self.blocks.insert(block.hash, block);
        Ok(())
    }

    fn validate_block_integrity(&self, block: &Block) -> Result<(), ConsensusError> {
        let canonical_hash = compute_block_hash(&block.header);
        if block.hash != canonical_hash {
            return Err(ConsensusError::InvalidBlockHash);
        }

        let computed_roots = compute_body_roots(&block.body);
        let header = &block.header;
        if header.body_root != computed_roots.body_root
            || header.finality_root != computed_roots.finality_root
            || header.authority_root != computed_roots.authority_root
            || header.lane_root != computed_roots.lane_root
            || header.proof_root != computed_roots.proof_root
            || header.identity_root != computed_roots.identity_root
            || header.ai_root != computed_roots.ai_root
            || header.pq_root != computed_roots.pq_root
            || header.external_settlement_root != computed_roots.external_settlement_root
            || header.policy_root != computed_roots.policy_root
            || header.time_seal_root != computed_roots.time_seal_root
            || header.capability_flags != computed_roots.capability_flags
        {
            return Err(ConsensusError::InvalidBlockBodyCommitments);
        }

        validate_block_semantics(
            block.header.timestamp,
            block.header.crypto_epoch,
            &block.body,
        )
        .map_err(|_| ConsensusError::InvalidBlockSemantics)?;

        validate_capability_section_alignment(&block.body, block.header.capability_flags)
            .map_err(|_| ConsensusError::InvalidBlockBodyCommitments)?;

        let empty_roots = compute_body_roots(&BlockBody::default());
        validate_root_semantic_bindings(&block.body, &computed_roots, &empty_roots)
            .map_err(|_| ConsensusError::InvalidBlockBodyCommitments)?;

        Ok(())
    }

    /// Verifies the supplied signed vote and admits the resulting canonical vote.
    pub fn add_signed_vote(
        &mut self,
        signed_vote: SignedVote,
    ) -> Result<(), VoteAuthenticationError> {
        let verified = signed_vote.verify()?;
        self.add_verified_vote(verified)
            .map_err(|_| VoteAuthenticationError::InvalidSignature)
    }

    /// Admits an already-verified vote into state.
    pub fn add_verified_vote(&mut self, verified_vote: VerifiedVote) -> Result<(), ConsensusError> {
        self.add_vote(verified_vote.into_vote())
    }

    /// Admits an authenticated vote only if the authentication context matches
    /// the exact local expectation.
    ///
    /// This prevents replay or cross-context contamination across network,
    /// epoch, validator-set, or signature-scheme boundaries.
    pub fn add_authenticated_vote(
        &mut self,
        verified_vote: VerifiedAuthenticatedVote,
        expected_context: VoteAuthenticationContext,
    ) -> Result<(), ConsensusError> {
        if verified_vote.context != expected_context {
            return Err(ConsensusError::InvalidAuthenticatedContext);
        }

        self.add_vote(verified_vote.vote)
    }

    /// Admits a vote into local consensus state after eligibility validation.
    ///
    /// # Security Semantics
    /// Votes are rejected when:
    /// - the target block is unknown,
    /// - the vote targets stale or conflicting finalized ancestry,
    /// - the voter is unknown,
    /// - the voter is inactive,
    /// - the voter is not eligible to vote.
    ///
    /// Duplicate and equivocating votes are further rejected by `VotePool`.
    pub fn add_vote(&mut self, vote: Vote) -> Result<(), ConsensusError> {
        let target = self
            .blocks
            .get(&vote.block_hash)
            .ok_or(ConsensusError::VoteForUnknownBlock)?;

        if vote.height != target.header.height {
            return Err(ConsensusError::VoteForUnknownBlock);
        }

        self.ensure_vote_targets_live_branch(vote.block_hash, vote.height)?;

        let validator = self
            .rotation
            .validator(vote.voter)
            .ok_or(ConsensusError::ValidatorNotFound)?;

        if !validator.is_eligible_for_vote() {
            if !validator.active {
                return Err(ConsensusError::InactiveValidator);
            }
            return Err(ConsensusError::NonVotingValidator);
        }

        self.vote_pool.add_vote(vote)
    }

    /// Returns observed voting power for a block and vote kind using only
    /// active, eligible voting participants.
    #[must_use]
    pub fn observed_voting_power(&self, block_hash: [u8; 32], kind: VoteKind) -> u64 {
        self.vote_pool
            .votes_for_block_kind(block_hash, kind)
            .into_iter()
            .filter_map(|vote| self.rotation.eligible_voting_power_of(vote.voter))
            .sum()
    }

    /// Returns `true` if quorum is reached for the specified block and vote kind.
    #[must_use]
    pub fn has_quorum(&self, block_hash: [u8; 32], kind: VoteKind) -> bool {
        let observed = self.observed_voting_power(block_hash, kind);
        let total = self.rotation.total_voting_power();
        self.quorum.is_reached(observed, total)
    }

    /// Attempts to finalize a block by constructing a quorum certificate and
    /// marking the block finalized in fork choice.
    ///
    /// # Security Semantics
    /// Finalization succeeds only when a valid quorum certificate can be built
    /// from eligible commit votes matching the exact block, height, and round.
    ///
    /// # Determinism
    /// The returned seal is fully derived from canonical block identity,
    /// deterministic signer normalization, and the fixed quorum policy.
    pub fn try_finalize(
        &mut self,
        block_hash: [u8; 32],
        finalized_round: u64,
    ) -> Option<BlockSeal> {
        let certificate = self.build_quorum_certificate(block_hash, finalized_round)?;
        let seal = BlockSeal {
            block_hash,
            finalized_round,
            attestation_root: certificate.certificate_hash,
            certificate,
        };

        if self.fork_choice.mark_finalized(block_hash, seal.clone()) {
            self.prune_to_finalized_branch(block_hash);
            Some(seal)
        } else {
            None
        }
    }

    /// Builds an authenticated quorum certificate by binding deterministic quorum
    /// evidence to the supplied authentication context.
    pub fn authenticated_quorum_certificate(
        &self,
        block_hash: [u8; 32],
        finalized_round: u64,
        context: VoteAuthenticationContext,
    ) -> Option<AuthenticatedQuorumCertificate> {
        let certificate = self.build_quorum_certificate(block_hash, finalized_round)?;
        Some(AuthenticatedQuorumCertificate::new(
            certificate,
            context.network_id,
            context.epoch,
            context.validator_set_root,
            context.signature_scheme,
        ))
    }

    /// Returns the proposer selected for the supplied height under the current
    /// deterministic rotation policy.
    #[must_use]
    pub fn proposer_for_height(&self, height: u64) -> Option<ValidatorId> {
        self.rotation.proposer(height)
    }

    /// Returns the proposer selected for `(height, round)` using deterministic
    /// weight-aware lottery and caller-supplied entropy.
    #[must_use]
    pub fn proposer_for_round(
        &self,
        height: u64,
        round: u64,
        entropy: [u8; 32],
    ) -> Option<ValidatorId> {
        self.rotation.proposer_with_round(height, round, entropy)
    }

    /// Bonds additional self stake for the validator and increases its voting power.
    pub fn bond_validator(
        &mut self,
        validator_id: ValidatorId,
        amount: u64,
    ) -> Result<(), ConsensusError> {
        let validator = self
            .rotation
            .validator_mut(validator_id)
            .ok_or(ConsensusError::ValidatorNotFound)?;
        validator.bond(amount);
        Ok(())
    }

    /// Starts unbonding and removes stake from immediate voting power.
    pub fn unbond_validator(
        &mut self,
        validator_id: ValidatorId,
        amount: u64,
    ) -> Result<u64, ConsensusError> {
        let validator = self
            .rotation
            .validator_mut(validator_id)
            .ok_or(ConsensusError::ValidatorNotFound)?;
        Ok(validator.unbond(amount))
    }

    /// Adds delegated stake to validator effective voting power.
    pub fn delegate_to_validator(
        &mut self,
        validator_id: ValidatorId,
        amount: u64,
    ) -> Result<(), ConsensusError> {
        let validator = self
            .rotation
            .validator_mut(validator_id)
            .ok_or(ConsensusError::ValidatorNotFound)?;
        validator.delegate(amount);
        Ok(())
    }

    /// Removes delegated stake from validator effective voting power.
    pub fn undelegate_from_validator(
        &mut self,
        validator_id: ValidatorId,
        amount: u64,
    ) -> Result<u64, ConsensusError> {
        let validator = self
            .rotation
            .validator_mut(validator_id)
            .ok_or(ConsensusError::ValidatorNotFound)?;
        Ok(validator.undelegate(amount))
    }

    /// Applies slashing to a validator under a supplied fault model.
    pub fn slash_validator(
        &mut self,
        validator_id: ValidatorId,
        numerator: u64,
        denominator: u64,
        fault: SlashFault,
    ) -> Result<u64, ConsensusError> {
        let validator = self
            .rotation
            .validator_mut(validator_id)
            .ok_or(ConsensusError::ValidatorNotFound)?;
        Ok(validator.slash(numerator, denominator, fault))
    }

    /// Returns the highest round for which the target block currently satisfies
    /// deterministic finalization requirements.
    #[must_use]
    pub fn finalizable_round(&self, block_hash: [u8; 32]) -> Option<u64> {
        let mut rounds: Vec<u64> = self
            .vote_pool
            .votes_for_block_kind(block_hash, VoteKind::Commit)
            .into_iter()
            .map(|vote| vote.round)
            .collect();

        rounds.sort_unstable();
        rounds.dedup();

        rounds
            .into_iter()
            .rev()
            .find(|round| self.build_quorum_certificate(block_hash, *round).is_some())
    }

    /// Builds a deterministic quorum certificate from eligible commit votes.
    ///
    /// # Certificate Rules
    /// - only `Commit` votes participate,
    /// - vote height must match the target block height,
    /// - vote round must match the requested finalization round,
    /// - only eligible validator voting power contributes,
    /// - signer ordering is canonicalized.
    fn build_quorum_certificate(
        &self,
        block_hash: [u8; 32],
        finalized_round: u64,
    ) -> Option<QuorumCertificate> {
        let block = self.blocks.get(&block_hash)?;

        let mut signer_set = BTreeSet::new();
        let mut signers = Vec::new();
        let mut observed_voting_power = 0u64;

        for vote in self
            .vote_pool
            .votes_for_block_kind(block_hash, VoteKind::Commit)
        {
            if vote.height != block.header.height || vote.round != finalized_round {
                continue;
            }

            if !signer_set.insert(vote.voter) {
                continue;
            }

            let voting_power = self.rotation.eligible_voting_power_of(vote.voter)?;
            signers.push(vote.voter);
            observed_voting_power = observed_voting_power.saturating_add(voting_power);
        }

        let total_voting_power = self.rotation.total_voting_power();
        if !self
            .quorum
            .is_reached(observed_voting_power, total_voting_power)
        {
            return None;
        }

        Some(QuorumCertificate::new(
            block_hash,
            block.header.height,
            finalized_round,
            signers,
            observed_voting_power,
            total_voting_power,
            self.quorum.numerator,
            self.quorum.denominator,
        ))
    }

    /// Enforces that the candidate vote remains on the live finalized branch.
    ///
    /// This policy is intentionally strict. Once a finalized head exists, votes
    /// must not target lower-height historical artifacts or conflicting branches.
    fn ensure_vote_targets_live_branch(
        &self,
        block_hash: [u8; 32],
        vote_height: u64,
    ) -> Result<(), ConsensusError> {
        let Some(finalized_hash) = self.fork_choice.finalized_head() else {
            return Ok(());
        };

        let finalized_height = self
            .fork_choice
            .get(finalized_hash)
            .map(|meta| meta.height)
            .unwrap_or_default();

        if vote_height < finalized_height {
            return Err(ConsensusError::StaleVote);
        }

        if block_hash != finalized_hash && !self.fork_choice.is_ancestor(finalized_hash, block_hash)
        {
            return Err(ConsensusError::StaleVote);
        }

        Ok(())
    }

    /// Removes blocks and votes that are no longer on the finalized branch.
    ///
    /// This is a memory-bounding and safety-preserving step. Once finality is
    /// established, conflicting non-finalized branches should not remain vote-active
    /// inside the canonical local state view.
    fn prune_to_finalized_branch(&mut self, finalized_hash: [u8; 32]) {
        self.blocks.retain(|hash, _| {
            *hash == finalized_hash || self.fork_choice.is_ancestor(finalized_hash, *hash)
        });

        self.vote_pool
            .prune_blocks(|hash| self.blocks.contains_key(&hash));
    }
}

#[cfg(test)]
mod tests {
    use ed25519_dalek::{Signer, SigningKey, VerifyingKey};

    use crate::block::{BlockBody, BlockBuilder};
    use crate::error::ConsensusError;
    use crate::quorum::QuorumThreshold;
    use crate::rotation::ValidatorRotation;
    use crate::validator::{Validator, ValidatorRole};
    use crate::vote::{VerifiedAuthenticatedVote, Vote, VoteAuthenticationContext, VoteKind};

    use super::ConsensusState;

    fn validator(id: u8, power: u64, role: ValidatorRole, active: bool) -> Validator {
        let mut validator = Validator::new([id; 32], power, role);
        validator.active = active;
        validator
    }

    fn state_with_validators(validators: Vec<Validator>) -> ConsensusState {
        let rotation = ValidatorRotation::new(validators).unwrap();
        let quorum = QuorumThreshold::new(2, 3).unwrap();
        ConsensusState::new(rotation, quorum)
    }

    fn make_block(parent_hash: [u8; 32], height: u64, proposer: [u8; 32]) -> crate::block::Block {
        BlockBuilder::build(
            1,
            parent_hash,
            height,
            0,
            height,
            height + 1,
            proposer,
            BlockBody::default(),
        )
        .unwrap()
    }

    fn admit_genesis(state: &mut ConsensusState, proposer: [u8; 32]) -> crate::block::Block {
        let genesis = make_block([0u8; 32], 0, proposer);
        state.admit_block(genesis.clone()).unwrap();
        genesis
    }

    fn inject_known_block(state: &mut ConsensusState, block: crate::block::Block) {
        state.blocks.insert(block.hash, block.clone());
        state
            .fork_choice
            .insert_block(crate::fork_choice::BlockMeta {
                hash: block.hash,
                parent: block.header.parent_hash,
                height: block.header.height,
                seal: None,
            });
    }

    #[test]
    fn rejects_vote_from_unknown_validator() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        let err = state
            .add_vote(Vote {
                voter: [9u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Prepare,
            })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            ConsensusError::ValidatorNotFound.to_string()
        );
    }

    #[test]
    fn rejects_vote_from_inactive_validator() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, false)]);
        let block = make_block([0u8; 32], 0, [1u8; 32]);
        inject_known_block(&mut state, block.clone());

        let err = state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Prepare,
            })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            ConsensusError::InactiveValidator.to_string()
        );
    }

    #[test]
    fn rejects_vote_from_non_voting_role() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Observer, true)]);
        let block = make_block([0u8; 32], 0, [1u8; 32]);
        inject_known_block(&mut state, block.clone());

        let err = state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Prepare,
            })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            ConsensusError::NonVotingValidator.to_string()
        );
    }

    #[test]
    fn observed_voting_power_ignores_inactive_and_non_eligible_validators() {
        let validators = vec![
            validator(1, 10, ValidatorRole::Validator, true),
            validator(2, 25, ValidatorRole::Observer, true),
            validator(3, 30, ValidatorRole::Validator, false),
        ];
        let rotation = ValidatorRotation::new(validators).unwrap();
        let mut state = ConsensusState::new(rotation, QuorumThreshold::new(2, 3).unwrap());
        let block = make_block([0u8; 32], 0, [1u8; 32]);
        inject_known_block(&mut state, block.clone());

        state
            .vote_pool
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Commit,
            })
            .unwrap();
        state
            .vote_pool
            .add_vote(Vote {
                voter: [2u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Commit,
            })
            .unwrap();
        state
            .vote_pool
            .add_vote(Vote {
                voter: [3u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Commit,
            })
            .unwrap();

        assert_eq!(
            state.observed_voting_power(block.hash, VoteKind::Commit),
            10
        );
    }

    #[test]
    fn duplicate_vote_is_rejected_as_duplicate() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);
        let vote = Vote {
            voter: [1u8; 32],
            block_hash: block.hash,
            height: 0,
            round: 1,
            kind: VoteKind::Prepare,
        };

        state.add_vote(vote.clone()).unwrap();
        let err = state.add_vote(vote).unwrap_err();

        assert_eq!(err.to_string(), ConsensusError::DuplicateVote.to_string());
    }

    #[test]
    fn conflicting_same_round_vote_is_rejected_as_equivocation() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);
        let child_a = make_block(genesis.hash, 1, [1u8; 32]);
        let child_b = make_block(genesis.hash, 1, [1u8; 32]);

        state.admit_block(child_a.clone()).unwrap();
        state.admit_block(child_b.clone()).unwrap_err();

        let alt_hash = [9u8; 32];
        inject_known_block(
            &mut state,
            crate::block::Block {
                hash: alt_hash,
                ..child_a.clone()
            },
        );

        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: child_a.hash,
                height: 1,
                round: 1,
                kind: VoteKind::Prepare,
            })
            .unwrap();

        let err = state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: alt_hash,
                height: 1,
                round: 1,
                kind: VoteKind::Prepare,
            })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            ConsensusError::EquivocatingVote.to_string()
        );
    }

    #[test]
    fn admits_equal_height_sibling_and_keeps_deterministic_head() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);

        let child_low =
            BlockBuilder::build(1, genesis.hash, 1, 0, 1, 2, [1u8; 32], BlockBody::default())
                .unwrap();
        let child_high =
            BlockBuilder::build(1, genesis.hash, 1, 0, 2, 3, [1u8; 32], BlockBody::default())
                .unwrap();

        state.admit_block(child_low.clone()).unwrap();
        state.admit_block(child_high.clone()).unwrap();

        assert_eq!(
            state.fork_choice.get_head(),
            Some(child_low.hash.max(child_high.hash)),
            "fork-choice head must remain deterministic for equal-height siblings",
        );
    }

    #[test]
    fn conflicting_prepare_and_commit_are_tracked_independently_by_kind() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 2,
                kind: VoteKind::Prepare,
            })
            .unwrap();
        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 2,
                kind: VoteKind::Commit,
            })
            .unwrap();
    }

    #[test]
    fn rejects_non_genesis_block_with_zero_parent() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = make_block([0u8; 32], 2, [1u8; 32]);

        let err = state.admit_block(block).unwrap_err();
        assert_eq!(
            err.to_string(),
            ConsensusError::InvalidGenesisParent.to_string()
        );
    }

    #[test]
    fn rejects_block_with_unknown_parent() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = make_block([7u8; 32], 1, [1u8; 32]);

        let err = state.admit_block(block).unwrap_err();
        assert_eq!(err.to_string(), ConsensusError::UnknownParent.to_string());
    }

    #[test]
    fn rejects_block_with_invalid_parent_height() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);
        let block = make_block(genesis.hash, 2, [1u8; 32]);

        let err = state.admit_block(block).unwrap_err();
        assert_eq!(
            err.to_string(),
            ConsensusError::InvalidParentHeight.to_string()
        );
    }

    #[test]
    fn rejects_duplicate_block_hash() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);

        let err = state.admit_block(genesis).unwrap_err();
        assert_eq!(err.to_string(), ConsensusError::DuplicateBlock.to_string());
    }

    #[test]
    fn accepts_valid_child_with_exact_height_continuity() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);
        let child = make_block(genesis.hash, 1, [1u8; 32]);

        state.admit_block(child.clone()).unwrap();
        assert!(state.fork_choice.contains(child.hash));
    }

    #[test]
    fn rejects_block_with_tampered_hash() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);

        let mut block = make_block([0u8; 32], 0, [1u8; 32]);
        block.hash = [99u8; 32];

        let err = state.admit_block(block).unwrap_err();
        assert_eq!(
            err.to_string(),
            ConsensusError::InvalidBlockHash.to_string()
        );
    }

    #[test]
    fn rejects_block_with_tampered_body_commitments() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);

        let mut block = make_block([0u8; 32], 0, [1u8; 32]);
        block.header.body_root = [77u8; 32];
        block.hash = crate::block::hash::compute_block_hash(&block.header);

        let err = state.admit_block(block).unwrap_err();
        assert_eq!(
            err.to_string(),
            ConsensusError::InvalidBlockBodyCommitments.to_string()
        );
    }

    #[test]
    fn finalization_builds_deterministic_quorum_certificate() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Commit,
            })
            .unwrap();

        let seal = state.try_finalize(block.hash, 0).unwrap();
        assert_eq!(seal.attestation_root, seal.certificate.certificate_hash);
        assert_eq!(seal.certificate.signers, vec![[1u8; 32]]);
        assert_eq!(seal.certificate.height, 0);
        assert_eq!(state.fork_choice.finalized_head(), Some(block.hash));
    }

    #[test]
    fn finalization_rejects_commit_votes_from_wrong_round() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 1,
                kind: VoteKind::Commit,
            })
            .unwrap();

        assert!(state.try_finalize(block.hash, 0).is_none());
    }

    #[test]
    fn finalized_branch_prunes_conflicting_blocks_and_votes() {
        let mut state = state_with_validators(vec![
            validator(1, 1, ValidatorRole::Validator, true),
            validator(2, 1, ValidatorRole::Validator, true),
            validator(3, 1, ValidatorRole::Validator, true),
        ]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);
        let canonical = make_block(genesis.hash, 1, [1u8; 32]);
        let conflicting = crate::block::Block {
            hash: [42u8; 32],
            ..make_block(genesis.hash, 1, [2u8; 32])
        };

        state.admit_block(canonical.clone()).unwrap();
        inject_known_block(&mut state, conflicting.clone());

        for voter in [[1u8; 32], [2u8; 32]] {
            state
                .add_vote(Vote {
                    voter,
                    block_hash: canonical.hash,
                    height: 1,
                    round: 1,
                    kind: VoteKind::Commit,
                })
                .unwrap();
        }

        state
            .vote_pool
            .add_vote(Vote {
                voter: [3u8; 32],
                block_hash: conflicting.hash,
                height: 1,
                round: 1,
                kind: VoteKind::Commit,
            })
            .unwrap();

        state.try_finalize(canonical.hash, 1).unwrap();

        assert!(state.blocks.contains_key(&canonical.hash));
        assert!(!state.blocks.contains_key(&conflicting.hash));
        assert_eq!(
            state
                .vote_pool
                .count_for_block_kind(conflicting.hash, VoteKind::Commit),
            0
        );
    }

    #[test]
    fn rejects_stale_votes_for_non_finalized_branch_after_finalization() {
        let mut state = state_with_validators(vec![
            validator(1, 1, ValidatorRole::Validator, true),
            validator(2, 1, ValidatorRole::Validator, true),
            validator(3, 1, ValidatorRole::Validator, true),
        ]);
        let genesis = admit_genesis(&mut state, [1u8; 32]);
        let canonical = make_block(genesis.hash, 1, [1u8; 32]);
        let child = make_block(canonical.hash, 2, [2u8; 32]);
        let conflicting = crate::block::Block {
            hash: [77u8; 32],
            ..make_block(genesis.hash, 1, [3u8; 32])
        };

        state.admit_block(canonical.clone()).unwrap();
        state.admit_block(child.clone()).unwrap();
        inject_known_block(&mut state, conflicting.clone());

        for voter in [[1u8; 32], [2u8; 32]] {
            state
                .add_vote(Vote {
                    voter,
                    block_hash: canonical.hash,
                    height: 1,
                    round: 1,
                    kind: VoteKind::Commit,
                })
                .unwrap();
        }

        state.try_finalize(canonical.hash, 1).unwrap();

        let err = state
            .add_vote(Vote {
                voter: [3u8; 32],
                block_hash: conflicting.hash,
                height: 1,
                round: 1,
                kind: VoteKind::Commit,
            })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            ConsensusError::VoteForUnknownBlock.to_string()
        );

        state
            .add_vote(Vote {
                voter: [3u8; 32],
                block_hash: child.hash,
                height: 2,
                round: 2,
                kind: VoteKind::Prepare,
            })
            .unwrap();
    }

    #[test]
    fn weighted_quorum_uses_power_not_validator_count() {
        let rotation = ValidatorRotation::new(vec![
            validator(1, 8, ValidatorRole::Validator, true),
            validator(2, 1, ValidatorRole::Validator, true),
            validator(3, 1, ValidatorRole::Validator, true),
        ])
        .unwrap();
        let mut state = ConsensusState::new(rotation, QuorumThreshold::new(2, 3).unwrap());
        let block = admit_genesis(&mut state, [1u8; 32]);

        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Commit,
            })
            .unwrap();

        assert!(state.has_quorum(block.hash, VoteKind::Commit));
        assert!(state.try_finalize(block.hash, 0).is_some());
    }

    #[test]
    fn insufficient_weight_rejects_count_majority_finalization() {
        let rotation = ValidatorRotation::new(vec![
            validator(1, 6, ValidatorRole::Validator, true),
            validator(2, 2, ValidatorRole::Validator, true),
            validator(3, 2, ValidatorRole::Validator, true),
        ])
        .unwrap();
        let mut state = ConsensusState::new(rotation, QuorumThreshold::new(2, 3).unwrap());
        let block = admit_genesis(&mut state, [1u8; 32]);

        for voter in [[2u8; 32], [3u8; 32]] {
            state
                .add_vote(Vote {
                    voter,
                    block_hash: block.hash,
                    height: 0,
                    round: 0,
                    kind: VoteKind::Commit,
                })
                .unwrap();
        }

        assert!(!state.has_quorum(block.hash, VoteKind::Commit));
        assert!(state.try_finalize(block.hash, 0).is_none());
    }

    #[test]
    fn add_signed_vote_accepts_valid_signature_end_to_end() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let voter = signing_key.verifying_key().to_bytes();
        let mut validator = Validator::new(voter, 10, ValidatorRole::Validator);
        validator.active = true;
        let mut state = state_with_validators(vec![validator]);
        let block = admit_genesis(&mut state, voter);
        let vote = Vote {
            voter,
            block_hash: block.hash,
            height: 0,
            round: 0,
            kind: VoteKind::Commit,
        };
        let signature = signing_key.sign(&vote.signing_bytes()).to_bytes().to_vec();

        state
            .add_signed_vote(crate::vote::SignedVote { vote, signature })
            .unwrap();

        assert_eq!(
            state
                .vote_pool
                .count_for_block_kind(block.hash, VoteKind::Commit),
            1
        );
    }

    #[test]
    fn add_signed_vote_rejects_invalid_signature_end_to_end() {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let voter = signing_key.verifying_key().to_bytes();
        let mut validator = Validator::new(voter, 10, ValidatorRole::Validator);
        validator.active = true;
        let mut state = state_with_validators(vec![validator]);
        let block = admit_genesis(&mut state, voter);
        let mut vote = Vote {
            voter,
            block_hash: block.hash,
            height: 0,
            round: 0,
            kind: VoteKind::Commit,
        };
        let signature = signing_key.sign(&vote.signing_bytes()).to_bytes().to_vec();
        vote.round = 1;

        let err = state
            .add_signed_vote(crate::vote::SignedVote { vote, signature })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            crate::vote::VoteAuthenticationError::InvalidSignature.to_string()
        );
    }

    #[test]
    fn add_signed_vote_rejects_malformed_key_end_to_end() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        let malformed_voter = (0u8..=u8::MAX)
            .map(|byte| [byte; 32])
            .find(|candidate| VerifyingKey::from_bytes(candidate).is_err())
            .expect("at least one malformed 32-byte encoding should be rejected");

        let err = state
            .add_signed_vote(crate::vote::SignedVote {
                vote: Vote {
                    voter: malformed_voter,
                    block_hash: block.hash,
                    height: 0,
                    round: 0,
                    kind: VoteKind::Commit,
                },
                signature: vec![0u8; 64],
            })
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            crate::vote::VoteAuthenticationError::MalformedPublicKey.to_string()
        );
    }

    #[test]
    fn authenticated_vote_rejects_context_mismatch() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        let err = state
            .add_authenticated_vote(
                VerifiedAuthenticatedVote {
                    vote: Vote {
                        voter: [1u8; 32],
                        block_hash: block.hash,
                        height: 0,
                        round: 0,
                        kind: VoteKind::Commit,
                    },
                    context: VoteAuthenticationContext {
                        network_id: 2626,
                        epoch: 7,
                        validator_set_root: [9u8; 32],
                        pq_attestation_root: [9u8; 32],
                        signature_scheme: 1,
                    },
                },
                VoteAuthenticationContext {
                    network_id: 2626,
                    epoch: 0,
                    validator_set_root: state.rotation.validator_set_hash(),
                    pq_attestation_root: state.rotation.pq_attestation_root(),
                    signature_scheme: 1,
                },
            )
            .unwrap_err();

        assert_eq!(
            err.to_string(),
            ConsensusError::InvalidAuthenticatedContext.to_string()
        );
    }

    #[test]
    fn try_finalize_is_idempotent_after_first_success() {
        let mut state =
            state_with_validators(vec![validator(1, 10, ValidatorRole::Validator, true)]);
        let block = admit_genesis(&mut state, [1u8; 32]);

        state
            .add_vote(Vote {
                voter: [1u8; 32],
                block_hash: block.hash,
                height: 0,
                round: 0,
                kind: VoteKind::Commit,
            })
            .unwrap();

        let first = state.try_finalize(block.hash, 0);
        let second = state.try_finalize(block.hash, 0);

        assert!(first.is_some());
        assert!(second.is_some());
        assert_eq!(first, second);
    }
}

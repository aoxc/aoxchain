use crate::block::Block;
use crate::error::ConsensusError;
use crate::fork_choice::{BlockMeta, ForkChoice};
use crate::quorum::QuorumThreshold;
use crate::rotation::ValidatorRotation;
use crate::round::RoundState;
use crate::seal::{BlockSeal, QuorumCertificate};
use crate::validator::ValidatorId;
use crate::vote::{Vote, VoteKind};
use crate::vote_pool::VotePool;

/// In-memory consensus state container.
///
/// Responsibilities:
/// - stores blocks admitted into local consensus view,
/// - tracks fork-choice head,
/// - admits votes,
/// - evaluates commit quorum,
/// - marks finalized blocks when quorum is reached.
#[derive(Debug, Clone)]
pub struct ConsensusState {
    pub fork_choice: ForkChoice,
    pub vote_pool: VotePool,
    pub round: RoundState,
    pub rotation: ValidatorRotation,
    pub quorum: QuorumThreshold,
    pub blocks: Vec<Block>,
}

impl ConsensusState {
    pub fn new(rotation: ValidatorRotation, quorum: QuorumThreshold) -> Self {
        Self {
            fork_choice: ForkChoice::new(),
            vote_pool: VotePool::new(),
            round: RoundState::new(),
            rotation,
            quorum,
            blocks: Vec::new(),
        }
    }

    /// Admits a block into local consensus state.
    ///
    /// Genesis policy in this phase is explicit: only height `0` or `1` may
    /// use the zero parent hash. Every non-genesis block must reference an
    /// existing parent and must advance height by exactly one.
    pub fn admit_block(&mut self, block: Block) -> Result<(), ConsensusError> {
        if self.fork_choice.contains(block.hash)
            || self
                .blocks
                .iter()
                .any(|existing| existing.hash == block.hash)
        {
            return Err(ConsensusError::DuplicateBlock);
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
                .iter()
                .find(|candidate| candidate.hash == block.header.parent_hash)
                .ok_or(ConsensusError::UnknownParent)?;

            if block.header.height != parent.header.height + 1 {
                return Err(ConsensusError::InvalidParentHeight);
            }
        }

        if let Some(head) = self.fork_choice.get_head()
            && let Some(head_meta) = self.fork_choice.get(head)
            && block.header.height <= head_meta.height
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

        self.blocks.push(block);
        Ok(())
    }

    pub fn add_vote(&mut self, vote: Vote) -> Result<(), ConsensusError> {
        if !self.fork_choice.contains(vote.block_hash) {
            return Err(ConsensusError::VoteForUnknownBlock);
        }

        let validator = self
            .rotation
            .validator(vote.voter)
            .ok_or(ConsensusError::ValidatorNotFound)?;

        if !validator.active {
            return Err(ConsensusError::InactiveValidator);
        }

        if !validator.is_eligible_for_vote() {
            return Err(ConsensusError::NonVotingValidator);
        }

        self.vote_pool.add_vote(vote)
    }

    pub fn observed_voting_power(&self, block_hash: [u8; 32], kind: VoteKind) -> u64 {
        self.vote_pool
            .votes_for_block(block_hash)
            .iter()
            .filter(|vote| vote.kind == kind)
            .filter_map(|vote| self.rotation.eligible_voting_power_of(vote.voter))
            .sum()
    }

    pub fn has_quorum(&self, block_hash: [u8; 32], kind: VoteKind) -> bool {
        let observed = self.observed_voting_power(block_hash, kind);
        let total = self.rotation.total_voting_power();
        self.quorum.is_reached(observed, total)
    }

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
            Some(seal)
        } else {
            None
        }
    }

    fn build_quorum_certificate(
        &self,
        block_hash: [u8; 32],
        finalized_round: u64,
    ) -> Option<QuorumCertificate> {
        let block = self.blocks.iter().find(|block| block.hash == block_hash)?;

        let mut signers = Vec::new();
        let mut observed_voting_power = 0u64;
        for vote in self.vote_pool.votes_for_block(block_hash) {
            if vote.kind != VoteKind::Commit {
                continue;
            }
            if vote.height != block.header.height || vote.round != finalized_round {
                continue;
            }
            let voting_power = self.rotation.eligible_voting_power_of(vote.voter)?;
            signers.push(vote.voter);
            observed_voting_power = observed_voting_power.saturating_add(voting_power);
        }

        signers.sort();
        signers.dedup();

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

    pub fn proposer_for_height(&self, height: u64) -> Option<ValidatorId> {
        self.rotation.proposer(height)
    }
}

#[cfg(test)]
mod tests {
    use crate::block::{BlockBody, BlockBuilder};
    use crate::error::ConsensusError;
    use crate::quorum::QuorumThreshold;
    use crate::rotation::ValidatorRotation;
    use crate::validator::{Validator, ValidatorRole};
    use crate::vote::{Vote, VoteKind};

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
        state.blocks.push(block.clone());
        state
            .fork_choice
            .insert_block(crate::fork_choice::BlockMeta {
                hash: block.hash,
                parent: block.header.parent_hash,
                height: block.header.height,
                seal: None,
            });

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
        state.blocks.push(block.clone());
        state
            .fork_choice
            .insert_block(crate::fork_choice::BlockMeta {
                hash: block.hash,
                parent: block.header.parent_hash,
                height: block.header.height,
                seal: None,
            });

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
        state.blocks.push(block.clone());
        state
            .fork_choice
            .insert_block(crate::fork_choice::BlockMeta {
                hash: block.hash,
                parent: block.header.parent_hash,
                height: block.header.height,
                seal: None,
            });

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
        state
            .fork_choice
            .insert_block(crate::fork_choice::BlockMeta {
                hash: alt_hash,
                parent: genesis.hash,
                height: 1,
                seal: None,
            });
        state.blocks.push(crate::block::Block {
            hash: alt_hash,
            ..child_a.clone()
        });

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
}

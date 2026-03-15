use crate::block::Block;
use crate::error::ConsensusError;
use crate::fork_choice::{BlockMeta, ForkChoice};
use crate::quorum::QuorumThreshold;
use crate::rotation::ValidatorRotation;
use crate::round::RoundState;
use crate::seal::BlockSeal;
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

    pub fn admit_block(&mut self, block: Block) -> Result<(), ConsensusError> {
        if block.header.height > 0
            && block.header.parent_hash != [0u8; 32]
            && !self.fork_choice.contains(block.header.parent_hash)
            && block.header.height != 1
        {
            return Err(ConsensusError::UnknownParent);
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

        self.vote_pool.add_vote(vote)
    }

    pub fn observed_voting_power(&self, block_hash: [u8; 32], kind: VoteKind) -> u64 {
        self.vote_pool
            .votes_for_block(block_hash)
            .iter()
            .filter(|vote| vote.kind == kind)
            .filter_map(|vote| self.rotation.voting_power_of(vote.voter))
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
        if !self.has_quorum(block_hash, VoteKind::Commit) {
            return None;
        }

        let attestation_root = self.synthetic_attestation_root(block_hash);
        let seal = BlockSeal {
            block_hash,
            finalized_round,
            attestation_root,
        };

        if self.fork_choice.mark_finalized(block_hash, seal.clone()) {
            Some(seal)
        } else {
            None
        }
    }

    fn synthetic_attestation_root(&self, block_hash: [u8; 32]) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(b"AOXC_ATTESTATION_ROOT_V1");
        hasher.update(block_hash);

        for vote in self.vote_pool.votes_for_block(block_hash) {
            if vote.kind == VoteKind::Commit {
                hasher.update(vote.voter);
                hasher.update(vote.height.to_le_bytes());
                hasher.update(vote.round.to_le_bytes());
            }
        }

        hasher.finalize().into()
    }

    pub fn proposer_for_height(&self, height: u64) -> Option<ValidatorId> {
        self.rotation.proposer(height)
    }
}

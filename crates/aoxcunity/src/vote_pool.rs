use std::collections::{HashMap, HashSet};

use crate::error::ConsensusError;
use crate::vote::{Vote, VoteKind};

type VoteKey = ([u8; 32], [u8; 32], u64, u64, VoteKind);

/// In-memory vote pool.
///
/// This structure prevents duplicate vote admission and supports quorum
/// counting by block hash and vote kind.
#[derive(Debug, Clone, Default)]
pub struct VotePool {
    seen: HashSet<VoteKey>,
    votes_by_block: HashMap<[u8; 32], Vec<Vote>>,
}

impl VotePool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_vote(&mut self, vote: Vote) -> Result<(), ConsensusError> {
        let key = vote.unique_key();
        if !self.seen.insert(key) {
            return Err(ConsensusError::DuplicateVote);
        }

        self.votes_by_block
            .entry(vote.block_hash)
            .or_default()
            .push(vote);

        Ok(())
    }

    pub fn votes_for_block(&self, block_hash: [u8; 32]) -> &[Vote] {
        self.votes_by_block
            .get(&block_hash)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn count_for_block_kind(&self, block_hash: [u8; 32], kind: VoteKind) -> usize {
        self.votes_for_block(block_hash)
            .iter()
            .filter(|vote| vote.kind == kind)
            .count()
    }
}

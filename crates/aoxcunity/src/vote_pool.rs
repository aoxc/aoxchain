use std::collections::{HashMap, HashSet};

use crate::error::ConsensusError;
use crate::validator::ValidatorId;
use crate::vote::{Vote, VoteKind};

type VoteKey = ([u8; 32], ValidatorId, u64, u64, VoteKind);
type ValidatorRoundKey = (ValidatorId, u64, u64, VoteKind);

#[derive(Debug, Clone, Default)]
struct BlockVoteBucket {
    prepare: HashMap<ValidatorId, Vote>,
    commit: HashMap<ValidatorId, Vote>,
}

impl BlockVoteBucket {
    fn insert(&mut self, vote: Vote) {
        self.votes_mut(vote.kind).insert(vote.voter, vote);
    }

    fn votes(&self, kind: VoteKind) -> impl Iterator<Item = &Vote> {
        self.votes_map(kind).values()
    }

    fn count(&self, kind: VoteKind) -> usize {
        self.votes_map(kind).len()
    }

    fn all_votes(&self) -> impl Iterator<Item = &Vote> {
        self.prepare.values().chain(self.commit.values())
    }

    fn is_empty(&self) -> bool {
        self.prepare.is_empty() && self.commit.is_empty()
    }

    fn votes_map(&self, kind: VoteKind) -> &HashMap<ValidatorId, Vote> {
        match kind {
            VoteKind::Prepare => &self.prepare,
            VoteKind::Commit => &self.commit,
        }
    }

    fn votes_mut(&mut self, kind: VoteKind) -> &mut HashMap<ValidatorId, Vote> {
        match kind {
            VoteKind::Prepare => &mut self.prepare,
            VoteKind::Commit => &mut self.commit,
        }
    }
}

/// In-memory vote pool.
///
/// This structure prevents duplicate vote admission, rejects validator
/// equivocation for a single round/kind tuple, stores votes in per-kind indexed
/// buckets, and supports pruning once finalized ancestry advances.
#[derive(Debug, Clone, Default)]
pub struct VotePool {
    seen: HashSet<VoteKey>,
    votes_by_block: HashMap<[u8; 32], BlockVoteBucket>,
    votes_by_validator_round: HashMap<ValidatorRoundKey, [u8; 32]>,
}

impl VotePool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_vote(&mut self, vote: Vote) -> Result<(), ConsensusError> {
        let validator_round_key = vote.conflict_key();
        if let Some(existing_block_hash) = self.votes_by_validator_round.get(&validator_round_key) {
            if *existing_block_hash == vote.block_hash {
                return Err(ConsensusError::DuplicateVote);
            }
            return Err(ConsensusError::EquivocatingVote);
        }

        let key = vote.unique_key();
        if !self.seen.insert(key) {
            return Err(ConsensusError::DuplicateVote);
        }

        self.votes_by_validator_round
            .insert(validator_round_key, vote.block_hash);

        self.votes_by_block
            .entry(vote.block_hash)
            .or_default()
            .insert(vote);

        Ok(())
    }

    pub fn votes_for_block_kind(&self, block_hash: [u8; 32], kind: VoteKind) -> Vec<&Vote> {
        self.votes_by_block
            .get(&block_hash)
            .map(|bucket| bucket.votes(kind).collect())
            .unwrap_or_default()
    }

    pub fn count_for_block_kind(&self, block_hash: [u8; 32], kind: VoteKind) -> usize {
        self.votes_by_block
            .get(&block_hash)
            .map(|bucket| bucket.count(kind))
            .unwrap_or(0)
    }

    pub fn prune_blocks<F>(&mut self, mut should_keep: F)
    where
        F: FnMut([u8; 32]) -> bool,
    {
        let mut removed_keys = Vec::new();
        self.votes_by_block.retain(|block_hash, bucket| {
            let keep = should_keep(*block_hash);
            if !keep {
                removed_keys.extend(bucket.all_votes().map(Vote::unique_key));
            }
            keep
        });

        if removed_keys.is_empty() {
            return;
        }

        for key in removed_keys {
            self.seen.remove(&key);
        }

        self.votes_by_validator_round
            .retain(|(validator_id, height, round, kind), block_hash| {
                self.votes_by_block.get(block_hash).is_some_and(|bucket| {
                    bucket.votes(*kind).any(|vote| {
                        vote.voter == *validator_id
                            && vote.height == *height
                            && vote.round == *round
                    })
                })
            });

        self.votes_by_block.retain(|_, bucket| !bucket.is_empty());
    }
}

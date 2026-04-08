// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::block::{Block, BlockBody, BlockBuildError, BlockBuilder};

/// Proposer-side block assembly utility.
///
/// This type encapsulates proposer identity and network identity while
/// delegating canonical hashing to the block builder.
#[derive(Debug, Clone, Copy)]
pub struct Proposer {
    pub network_id: u32,
    pub proposer_id: [u8; 32],
}

impl Proposer {
    pub fn new(network_id: u32, proposer_id: [u8; 32]) -> Self {
        Self {
            network_id,
            proposer_id,
        }
    }

    pub fn propose(
        &self,
        parent_hash: [u8; 32],
        height: u64,
        era: u64,
        round: u64,
        timestamp: u64,
        body: BlockBody,
    ) -> Result<Block, BlockBuildError> {
        BlockBuilder::build(
            self.network_id,
            parent_hash,
            height,
            era,
            round,
            timestamp,
            self.proposer_id,
            body,
        )
    }
}

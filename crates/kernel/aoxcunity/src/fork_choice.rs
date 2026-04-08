// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use std::collections::HashMap;

use crate::seal::BlockSeal;

/// Minimal block metadata required by fork choice.
///
/// The metadata intentionally excludes full block bodies in order to keep
/// the fork-choice structure lightweight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockMeta {
    pub hash: [u8; 32],
    pub parent: [u8; 32],
    pub height: u64,
    pub seal: Option<BlockSeal>,
}

/// Canonical fork-choice state.
///
/// Current policy:
/// - finalized ancestry is the primary safety anchor,
/// - the selected head must remain on the finalized branch if one exists,
/// - otherwise prefer the highest known block by height,
/// - equal-height ties are resolved deterministically by block hash.
#[derive(Debug, Clone, Default)]
pub struct ForkChoice {
    blocks: HashMap<[u8; 32], BlockMeta>,
    head: Option<[u8; 32]>,
    finalized_head: Option<[u8; 32]>,
}

impl ForkChoice {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_block(&mut self, block: BlockMeta) {
        let hash = block.hash;
        self.blocks.insert(hash, block);
        self.recompute_head();
    }

    pub fn mark_finalized(&mut self, block_hash: [u8; 32], seal: BlockSeal) -> bool {
        if let Some(existing_finalized) = self.finalized_head
            && existing_finalized != block_hash
            && !self.is_ancestor(block_hash, existing_finalized)
            && !self.is_ancestor(existing_finalized, block_hash)
        {
            return false;
        }

        if let Some(meta) = self.blocks.get_mut(&block_hash) {
            meta.seal = Some(seal);
            self.finalized_head = Some(block_hash);
            self.recompute_head();
            return true;
        }

        false
    }

    pub fn get_head(&self) -> Option<[u8; 32]> {
        self.head
    }

    pub fn finalized_head(&self) -> Option<[u8; 32]> {
        self.finalized_head
    }

    pub fn contains(&self, block_hash: [u8; 32]) -> bool {
        self.blocks.contains_key(&block_hash)
    }

    pub fn get(&self, block_hash: [u8; 32]) -> Option<&BlockMeta> {
        self.blocks.get(&block_hash)
    }

    pub fn is_ancestor(&self, ancestor: [u8; 32], mut descendant: [u8; 32]) -> bool {
        loop {
            if ancestor == descendant {
                return true;
            }

            let Some(meta) = self.blocks.get(&descendant) else {
                return false;
            };

            if meta.parent == [0u8; 32] {
                return false;
            }
            descendant = meta.parent;
        }
    }

    fn recompute_head(&mut self) {
        let finalized_head = self.finalized_head;
        self.head = self
            .blocks
            .values()
            .filter(|candidate| {
                finalized_head.is_none_or(|finalized| self.is_ancestor(finalized, candidate.hash))
            })
            .max_by(|a, b| a.height.cmp(&b.height).then_with(|| a.hash.cmp(&b.hash)))
            .map(|meta| meta.hash)
            .or(finalized_head);
    }
}

#[cfg(test)]
mod tests {
    use crate::seal::{BlockSeal, QuorumCertificate};

    use super::{BlockMeta, ForkChoice};

    fn meta(hash: u8, parent: u8, height: u64) -> BlockMeta {
        BlockMeta {
            hash: [hash; 32],
            parent: [parent; 32],
            height,
            seal: None,
        }
    }

    #[test]
    fn finalized_head_restricts_head_to_same_ancestral_branch() {
        let mut fork_choice = ForkChoice::new();
        fork_choice.insert_block(meta(1, 0, 1));
        fork_choice.insert_block(meta(2, 1, 2));
        fork_choice.insert_block(meta(3, 1, 2));
        fork_choice.insert_block(meta(4, 2, 3));
        fork_choice.insert_block(meta(5, 3, 4));

        let certificate = QuorumCertificate::new([2; 32], 2, 1, vec![[7; 32]], 1, 1, 1, 1);
        assert!(fork_choice.mark_finalized(
            [2; 32],
            BlockSeal {
                block_hash: [2; 32],
                finalized_round: 1,
                attestation_root: certificate.certificate_hash,
                certificate,
            },
        ));

        assert_eq!(fork_choice.finalized_head(), Some([2; 32]));
        assert_eq!(fork_choice.get_head(), Some([4; 32]));
    }

    #[test]
    fn equal_height_tie_breaker_is_deterministic() {
        let mut fork_choice = ForkChoice::new();
        fork_choice.insert_block(meta(1, 0, 1));
        fork_choice.insert_block(meta(9, 1, 2));
        fork_choice.insert_block(meta(7, 1, 2));

        assert_eq!(fork_choice.get_head(), Some([9; 32]));
    }
}

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
/// - prefer the highest finalized block if any exist,
/// - otherwise prefer the highest known block by height.
#[derive(Debug, Clone, Default)]
pub struct ForkChoice {
    blocks: HashMap<[u8; 32], BlockMeta>,
    head: Option<[u8; 32]>,
}

impl ForkChoice {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_block(&mut self, block: BlockMeta) {
        let hash = block.hash;
        let height = block.height;
        let is_finalized = block.seal.is_some();

        self.blocks.insert(hash, block);

        match self.head {
            None => self.head = Some(hash),
            Some(current_head) => {
                let current = self.blocks.get(&current_head);
                let should_replace = match current {
                    None => true,
                    Some(current_meta) => {
                        let current_finalized = current_meta.seal.is_some();

                        match (is_finalized, current_finalized) {
                            (true, false) => true,
                            (false, true) => false,
                            _ => height > current_meta.height,
                        }
                    }
                };

                if should_replace {
                    self.head = Some(hash);
                }
            }
        }
    }

    pub fn mark_finalized(&mut self, block_hash: [u8; 32], seal: BlockSeal) -> bool {
        if let Some(meta) = self.blocks.get_mut(&block_hash) {
            meta.seal = Some(seal);
            self.head = Some(block_hash);
            return true;
        }

        false
    }

    pub fn get_head(&self) -> Option<[u8; 32]> {
        self.head
    }

    pub fn contains(&self, block_hash: [u8; 32]) -> bool {
        self.blocks.contains_key(&block_hash)
    }

    pub fn get(&self, block_hash: [u8; 32]) -> Option<&BlockMeta> {
        self.blocks.get(&block_hash)
    }
}

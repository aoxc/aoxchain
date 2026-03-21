use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Merkle proof branch direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchDirection {
    Left,
    Right,
}

/// Single Merkle proof step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofStep {
    pub sibling_hash_hex: String,
    pub sibling_direction: BranchDirection,
}

/// Inclusion proof for a state-tree key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateProof {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub leaf_hash_hex: String,
    pub steps: Vec<ProofStep>,
    pub root_hash_hex: String,
}

/// Deterministic ordered state tree with Merkle proof generation.
///
/// This is intentionally not a Patricia trie or sparse Merkle tree. It is a
/// stable, auditable ordered Merkle map suitable for deterministic state-root
/// generation and proof verification in foundational environments.
#[derive(Debug, Default, Clone)]
pub struct StateTree {
    nodes: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl StateTree {
    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.nodes.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.nodes.get(key).map(Vec::as_slice)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn root_hash_hex(&self) -> String {
        let tree = self.build_merkle_layers();
        tree.root_hex
    }

    pub fn generate_proof(&self, key: &[u8]) -> Option<StateProof> {
        let position = self
            .nodes
            .keys()
            .position(|candidate| candidate.as_slice() == key)?;
        let value = self.nodes.get(key)?.clone();

        let mut layers = self.leaf_hashes();
        let leaf_hash = layers.get(position)?.clone();
        let mut index = position;
        let mut steps = Vec::new();

        while layers.len() > 1 {
            let sibling_index = if index % 2 == 0 {
                if index + 1 < layers.len() {
                    Some(index + 1)
                } else {
                    Some(index)
                }
            } else {
                Some(index - 1)
            };

            if let Some(sibling_index) = sibling_index {
                let sibling_hash_hex = layers[sibling_index].clone();
                let sibling_direction = if index % 2 == 0 {
                    BranchDirection::Right
                } else {
                    BranchDirection::Left
                };
                steps.push(ProofStep {
                    sibling_hash_hex,
                    sibling_direction,
                });
            }

            let mut next_layer = Vec::new();
            let mut cursor = 0usize;
            while cursor < layers.len() {
                let left = &layers[cursor];
                let right = if cursor + 1 < layers.len() {
                    &layers[cursor + 1]
                } else {
                    left
                };
                next_layer.push(parent_hash_hex(left, right));
                cursor += 2;
            }

            layers = next_layer;
            index /= 2;
        }

        Some(StateProof {
            key: key.to_vec(),
            value,
            leaf_hash_hex: leaf_hash,
            steps,
            root_hash_hex: self.root_hash_hex(),
        })
    }

    pub fn verify_proof(proof: &StateProof) -> bool {
        let expected_leaf = leaf_hash_hex(&proof.key, &proof.value);
        if expected_leaf != proof.leaf_hash_hex {
            return false;
        }

        let mut current = expected_leaf;
        for step in &proof.steps {
            current = match step.sibling_direction {
                BranchDirection::Left => parent_hash_hex(&step.sibling_hash_hex, &current),
                BranchDirection::Right => parent_hash_hex(&current, &step.sibling_hash_hex),
            };
        }

        current == proof.root_hash_hex
    }

    fn leaf_hashes(&self) -> Vec<String> {
        self.nodes
            .iter()
            .map(|(key, value)| leaf_hash_hex(key, value))
            .collect()
    }

    fn build_merkle_layers(&self) -> MerkleLayers {
        let mut layers = self.leaf_hashes();

        if layers.is_empty() {
            let empty = hex::encode(Sha256::digest(b"AOXC_STATE_EMPTY_ROOT_V1"));
            return MerkleLayers { root_hex: empty };
        }

        while layers.len() > 1 {
            let mut next_layer = Vec::new();
            let mut cursor = 0usize;
            while cursor < layers.len() {
                let left = &layers[cursor];
                let right = if cursor + 1 < layers.len() {
                    &layers[cursor + 1]
                } else {
                    left
                };
                next_layer.push(parent_hash_hex(left, right));
                cursor += 2;
            }
            layers = next_layer;
        }

        MerkleLayers {
            root_hex: layers[0].clone(),
        }
    }
}

#[derive(Debug)]
struct MerkleLayers {
    root_hex: String,
}

fn leaf_hash_hex(key: &[u8], value: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC_STATE_LEAF_V1");
    hasher.update((key.len() as u64).to_le_bytes());
    hasher.update(key);
    hasher.update((value.len() as u64).to_le_bytes());
    hasher.update(value);
    hex::encode(hasher.finalize())
}

fn parent_hash_hex(left_hex: &str, right_hex: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"AOXC_STATE_NODE_V1");
    hasher.update(left_hex.as_bytes());
    hasher.update(right_hex.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tree_has_deterministic_root() {
        let tree = StateTree::default();
        let root = tree.root_hash_hex();
        let expected = hex::encode(Sha256::digest(b"AOXC_STATE_EMPTY_ROOT_V1"));
        assert_eq!(root, expected);
    }

    #[test]
    fn root_changes_when_state_changes() {
        let mut tree = StateTree::default();
        let empty_root = tree.root_hash_hex();

        tree.insert(b"a".to_vec(), b"1".to_vec());
        let root_after_insert = tree.root_hash_hex();

        assert_ne!(empty_root, root_after_insert);
    }

    #[test]
    fn proof_verification_succeeds_for_inserted_key() {
        let mut tree = StateTree::default();
        tree.insert(b"alice".to_vec(), b"10".to_vec());
        tree.insert(b"bob".to_vec(), b"20".to_vec());
        tree.insert(b"carol".to_vec(), b"30".to_vec());

        let proof = tree
            .generate_proof(b"bob")
            .expect("proof must exist for inserted key");

        assert!(StateTree::verify_proof(&proof));
        assert_eq!(proof.root_hash_hex, tree.root_hash_hex());
    }

    #[test]
    fn proof_verification_fails_when_value_is_tampered() {
        let mut tree = StateTree::default();
        tree.insert(b"alice".to_vec(), b"10".to_vec());
        tree.insert(b"bob".to_vec(), b"20".to_vec());

        let mut proof = tree
            .generate_proof(b"alice")
            .expect("proof must exist for inserted key");
        proof.value = b"999".to_vec();

        assert!(!StateTree::verify_proof(&proof));
    }

    #[test]
    fn proof_is_absent_for_missing_key() {
        let tree = StateTree::default();
        assert!(tree.generate_proof(b"missing").is_none());
    }
}

//! Deterministic Merkle commitments and inclusion proofs for AOXC-VM.
//!
//! This module provides a production-ready binary Merkle tree with:
//! - explicit domain separation,
//! - deterministic odd-node policy (duplicate-last),
//! - proof generation and verification,
//! - strict proof-shape validation.

use sha2::{Digest, Sha256};

const DOMAIN_EMPTY_ROOT_V1: &[u8] = b"AOXCVM_MERKLE_EMPTY_ROOT_V1";
const DOMAIN_LEAF_V1: &[u8] = b"AOXCVM_MERKLE_LEAF_V1";
const DOMAIN_INTERNAL_V1: &[u8] = b"AOXCVM_MERKLE_NODE_V1";

/// Canonical digest size for this Merkle construction.
pub const MERKLE_HASH_BYTES: usize = 32;

/// Canonical Merkle hash type.
pub type MerkleHash = [u8; MERKLE_HASH_BYTES];

/// Neighbor direction for a proof step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiblingDirection {
    Left,
    Right,
}

/// One proof step from a leaf toward the root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProofStep {
    pub sibling_hash: MerkleHash,
    pub sibling_direction: SiblingDirection,
}

/// Inclusion proof for a single leaf in an ordered collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MerkleProof {
    /// Leaf position in the original ordered list.
    pub leaf_index: usize,
    /// Canonical leaf hash for `leaf_value`.
    pub leaf_hash: MerkleHash,
    /// Hash path from the leaf toward the root.
    pub steps: Vec<MerkleProofStep>,
    /// Root hash this proof commits to.
    pub root_hash: MerkleHash,
}

/// Error class for proof construction and verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MerkleError {
    /// The input set did not contain any leaves.
    EmptyLeaves,
    /// Requested leaf index is out of bounds for provided leaves.
    IndexOutOfBounds,
    /// The proof shape is invalid for the stated index and leaf count.
    InvalidProofShape,
}

/// Computes the deterministic empty-tree root.
#[must_use]
pub fn empty_root() -> MerkleHash {
    hash_domain_payload(DOMAIN_EMPTY_ROOT_V1, &[])
}

/// Computes the Merkle root for ordered leaf payloads.
///
/// Policy:
/// - Empty input => deterministic `empty_root()`.
/// - Odd level width => duplicate the final node on that level.
#[must_use]
pub fn root_for_leaves(leaves: &[Vec<u8>]) -> MerkleHash {
    if leaves.is_empty() {
        return empty_root();
    }

    let mut level: Vec<MerkleHash> = leaves.iter().map(|leaf| hash_leaf(leaf)).collect();
    while level.len() > 1 {
        level = fold_level(&level);
    }

    level[0]
}

/// Builds an inclusion proof for `leaf_index` in the ordered `leaves` set.
pub fn build_proof(leaves: &[Vec<u8>], leaf_index: usize) -> Result<MerkleProof, MerkleError> {
    if leaves.is_empty() {
        return Err(MerkleError::EmptyLeaves);
    }
    if leaf_index >= leaves.len() {
        return Err(MerkleError::IndexOutOfBounds);
    }

    let mut levels = Vec::new();
    levels.push(
        leaves
            .iter()
            .map(|leaf| hash_leaf(leaf))
            .collect::<Vec<_>>(),
    );

    while levels.last().is_some_and(|level| level.len() > 1) {
        let next = fold_level(levels.last().expect("level exists"));
        levels.push(next);
    }

    let mut current_index = leaf_index;
    let mut steps = Vec::new();

    for level in &levels[..levels.len() - 1] {
        let sibling_index = if current_index % 2 == 0 {
            if current_index + 1 < level.len() {
                current_index + 1
            } else {
                current_index
            }
        } else {
            current_index - 1
        };

        let sibling_direction = if current_index % 2 == 0 {
            SiblingDirection::Right
        } else {
            SiblingDirection::Left
        };

        steps.push(MerkleProofStep {
            sibling_hash: level[sibling_index],
            sibling_direction,
        });

        current_index /= 2;
    }

    Ok(MerkleProof {
        leaf_index,
        leaf_hash: hash_leaf(&leaves[leaf_index]),
        steps,
        root_hash: *levels
            .last()
            .and_then(|level| level.first())
            .expect("non-empty tree has root"),
    })
}

/// Verifies that a proof is valid for (`leaf_count`, `leaf_value`, `root_hash`).
///
/// The verifier rejects malformed proof shape (wrong depth for the provided
/// index/count relationship) before checking hash equality.
pub fn verify_proof(
    leaf_count: usize,
    leaf_value: &[u8],
    proof: &MerkleProof,
    expected_root: &MerkleHash,
) -> Result<bool, MerkleError> {
    if leaf_count == 0 {
        return Err(MerkleError::EmptyLeaves);
    }
    if proof.leaf_index >= leaf_count {
        return Err(MerkleError::IndexOutOfBounds);
    }

    let expected_steps = expected_proof_steps(leaf_count);
    if proof.steps.len() != expected_steps {
        return Err(MerkleError::InvalidProofShape);
    }

    let leaf_hash = hash_leaf(leaf_value);
    if leaf_hash != proof.leaf_hash {
        return Ok(false);
    }

    let mut idx = proof.leaf_index;
    let mut current = leaf_hash;

    for step in &proof.steps {
        if idx % 2 == 0 {
            if step.sibling_direction != SiblingDirection::Right {
                return Err(MerkleError::InvalidProofShape);
            }
            current = hash_internal_node(&current, &step.sibling_hash);
        } else {
            if step.sibling_direction != SiblingDirection::Left {
                return Err(MerkleError::InvalidProofShape);
            }
            current = hash_internal_node(&step.sibling_hash, &current);
        }
        idx /= 2;
    }

    Ok(current == *expected_root && current == proof.root_hash)
}

fn fold_level(level: &[MerkleHash]) -> Vec<MerkleHash> {
    let mut out = Vec::with_capacity(level.len().div_ceil(2));
    let mut i = 0usize;
    while i < level.len() {
        let left = level[i];
        let right = if i + 1 < level.len() {
            level[i + 1]
        } else {
            left
        };
        out.push(hash_internal_node(&left, &right));
        i += 2;
    }
    out
}

fn expected_proof_steps(leaf_count: usize) -> usize {
    let mut width = leaf_count;
    let mut depth = 0usize;
    while width > 1 {
        width = width.div_ceil(2);
        depth += 1;
    }
    depth
}

fn hash_leaf(leaf: &[u8]) -> MerkleHash {
    let mut payload = Vec::with_capacity(8 + leaf.len());
    payload.extend_from_slice(&(leaf.len() as u64).to_be_bytes());
    payload.extend_from_slice(leaf);
    hash_domain_payload(DOMAIN_LEAF_V1, &payload)
}

fn hash_internal_node(left: &MerkleHash, right: &MerkleHash) -> MerkleHash {
    let mut payload = [0_u8; MERKLE_HASH_BYTES * 2];
    payload[..MERKLE_HASH_BYTES].copy_from_slice(left);
    payload[MERKLE_HASH_BYTES..].copy_from_slice(right);
    hash_domain_payload(DOMAIN_INTERNAL_V1, &payload)
}

fn hash_domain_payload(domain: &[u8], payload: &[u8]) -> MerkleHash {
    let mut hasher = Sha256::new();
    hasher.update((domain.len() as u32).to_be_bytes());
    hasher.update(domain);
    hasher.update(payload);
    let digest = hasher.finalize();

    let mut out = [0_u8; MERKLE_HASH_BYTES];
    out.copy_from_slice(&digest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_leaves() -> Vec<Vec<u8>> {
        vec![
            b"a".to_vec(),
            b"b".to_vec(),
            b"c".to_vec(),
            b"d".to_vec(),
            b"e".to_vec(),
        ]
    }

    #[test]
    fn empty_root_is_deterministic() {
        assert_eq!(empty_root(), empty_root());
        assert_ne!(empty_root(), [0_u8; MERKLE_HASH_BYTES]);
    }

    #[test]
    fn root_is_deterministic_and_order_sensitive() {
        let leaves = sample_leaves();
        let mut reordered = sample_leaves();
        reordered.swap(1, 2);

        let a = root_for_leaves(&leaves);
        let b = root_for_leaves(&leaves);
        let c = root_for_leaves(&reordered);

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn proof_roundtrip_verifies() {
        let leaves = sample_leaves();
        let root = root_for_leaves(&leaves);
        let proof = build_proof(&leaves, 3).expect("proof must build");

        let ok = verify_proof(leaves.len(), &leaves[3], &proof, &root).expect("valid shape");
        assert!(ok);
    }

    #[test]
    fn proof_rejects_tampered_leaf() {
        let leaves = sample_leaves();
        let root = root_for_leaves(&leaves);
        let proof = build_proof(&leaves, 1).expect("proof must build");

        let ok = verify_proof(leaves.len(), b"tampered", &proof, &root).expect("valid shape");
        assert!(!ok);
    }

    #[test]
    fn proof_rejects_wrong_direction_shape() {
        let leaves = sample_leaves();
        let root = root_for_leaves(&leaves);
        let mut proof = build_proof(&leaves, 0).expect("proof must build");

        proof.steps[0].sibling_direction = SiblingDirection::Left;

        let err = verify_proof(leaves.len(), &leaves[0], &proof, &root)
            .expect_err("shape must be rejected");
        assert_eq!(err, MerkleError::InvalidProofShape);
    }

    #[test]
    fn proof_rejects_wrong_depth_shape() {
        let leaves = sample_leaves();
        let root = root_for_leaves(&leaves);
        let mut proof = build_proof(&leaves, 0).expect("proof must build");

        let _ = proof.steps.pop();

        let err = verify_proof(leaves.len(), &leaves[0], &proof, &root)
            .expect_err("shape must be rejected");
        assert_eq!(err, MerkleError::InvalidProofShape);
    }

    #[test]
    fn build_proof_rejects_empty_or_invalid_index() {
        assert_eq!(build_proof(&[], 0), Err(MerkleError::EmptyLeaves));

        let leaves = sample_leaves();
        assert_eq!(
            build_proof(&leaves, leaves.len()),
            Err(MerkleError::IndexOutOfBounds)
        );
    }
}

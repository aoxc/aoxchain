// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! core/src/block/hash.rs
//!
//! Canonical AOVM block-domain hashing and commitment module.
//!
//! This module provides deterministic, domain-separated, versioned hashing
//! primitives for block headers, tasks, task leaves, internal Merkle nodes,
//! and task-root commitments.
//!
//! Security and design objectives:
//! - Stable outputs across platforms and architectures
//! - Explicit domain separation for all structured hash types
//! - Forward-compatible encoding discipline
//! - Panic-free canonical hashing paths where fallibility is possible
//! - Deterministic Merkle-style task-root commitments
//! - Minimal ambiguity around empty collections and structural encodings
//!
//! Security notes:
//! - BLAKE3 is the canonical digest primitive for this module.
//! - All structured hashes are namespace-bound and domain-separated.
//! - Variable-length fields are length-prefixed using canonical fixed-width encoding.
//! - A protocol hash-format version is embedded into all structured digests.
//! - This module strengthens commitment determinism but does not, by itself,
//!   provide end-to-end protocol security. Full security also depends on
//!   authorization, identity, execution, networking, and state-transition layers.

use blake3::Hasher;

use super::{BlockError, BlockHeader, Task};

/// Canonical hash output size in bytes.
pub const HASH_SIZE: usize = 32;

/// Canonical zero hash constant.
///
/// Operational note:
/// This constant is exposed for callers that require an explicit all-zero
/// sentinel. Structured empty commitments should prefer [`empty_task_root`]
/// rather than relying on an all-zero placeholder.
pub const ZERO_HASH: [u8; HASH_SIZE] = [0u8; HASH_SIZE];

/// Version tag for the canonical structured hashing format.
///
/// Governance note:
/// This value must be incremented whenever the canonical encoding or
/// commitment layout changes in a backward-incompatible manner.
pub const HASH_FORMAT_VERSION: u8 = 1;

/// Global namespace used for all block-domain hash derivations.
///
/// Security rationale:
/// A top-level namespace reduces the risk of cross-module hash confusion
/// if a downstream system reuses similar domain tags.
const PROTOCOL_HASH_NAMESPACE: &[u8] = b"AOXC/AOVM/BLOCK/HASH";

/// Domain separator for generic utility hashing.
const DOMAIN_GENERIC: &[u8] = b"GENERIC";

/// Domain separator for block-header hashing.
const DOMAIN_HEADER: &[u8] = b"HEADER";

/// Domain separator for canonical task hashing.
const DOMAIN_TASK: &[u8] = b"TASK";

/// Domain separator for task-root hashing.
const DOMAIN_TASK_ROOT: &[u8] = b"TASK_ROOT";

/// Domain separator for the canonical empty task root.
const DOMAIN_EMPTY_TASK_ROOT: &[u8] = b"EMPTY_TASK_ROOT";

/// Domain separator for task-leaf hashing.
const DOMAIN_TASK_LEAF: &[u8] = b"TASK_LEAF";

/// Domain separator for internal Merkle-node hashing.
const DOMAIN_TASK_INTERNAL: &[u8] = b"TASK_INTERNAL";

/// Returns a fresh BLAKE3 hasher initialized with namespace, domain, and
/// format-version tags.
///
/// Encoding contract:
/// - namespace
/// - separator byte
/// - domain
/// - separator byte
/// - hash format version
#[inline]
fn new_tagged_hasher(domain: &[u8]) -> Hasher {
    let mut hasher = Hasher::new();
    hasher.update(PROTOCOL_HASH_NAMESPACE);
    hasher.update(&[0x00]);
    hasher.update(domain);
    hasher.update(&[0x00]);
    hasher.update(&[HASH_FORMAT_VERSION]);
    hasher
}

/// Encodes a `u8` using canonical little-endian-compatible byte form.
#[inline]
fn update_u8(hasher: &mut Hasher, value: u8) {
    hasher.update(&[value]);
}

/// Encodes a `u16` using canonical little-endian representation.
#[inline]
fn update_u16(hasher: &mut Hasher, value: u16) {
    hasher.update(&value.to_le_bytes());
}

/// Encodes a `u64` using canonical little-endian representation.
#[inline]
fn update_u64(hasher: &mut Hasher, value: u64) {
    hasher.update(&value.to_le_bytes());
}

/// Encodes a fixed 32-byte value directly into the hash stream.
#[inline]
fn update_bytes32(hasher: &mut Hasher, value: &[u8; HASH_SIZE]) {
    hasher.update(value);
}

/// Encodes a variable-length byte slice with a canonical 64-bit length prefix.
///
/// Failure mode:
/// Returns [`BlockError::LengthOverflow`] if the slice length cannot be
/// represented as a `u64`.
#[inline]
fn update_bytes(hasher: &mut Hasher, value: &[u8]) -> Result<(), BlockError> {
    let len = u64::try_from(value.len()).map_err(|_| BlockError::LengthOverflow)?;
    update_u64(hasher, len);
    hasher.update(value);
    Ok(())
}

/// Finalizes the hasher into the canonical fixed-size digest representation.
#[inline]
fn finalize_hash(hasher: Hasher) -> [u8; HASH_SIZE] {
    *hasher.finalize().as_bytes()
}

/// Computes a domain-separated generic BLAKE3 hash over an arbitrary byte slice.
///
/// Intended use:
/// This function is suitable for utility hashing where no richer structured
/// commitment exists. Consensus-critical domain objects should prefer their
/// dedicated structured hashing functions.
pub fn compute_hash(data: &[u8]) -> Result<[u8; HASH_SIZE], BlockError> {
    let mut hasher = new_tagged_hasher(DOMAIN_GENERIC);
    update_bytes(&mut hasher, data)?;
    Ok(finalize_hash(hasher))
}

/// Returns the canonical empty task root.
///
/// Security rationale:
/// The empty task root is domain-separated and intentionally distinct from
/// [`ZERO_HASH`] to avoid ambiguity between “no tasks” and “unset value”.
#[must_use]
pub fn empty_task_root() -> [u8; HASH_SIZE] {
    let hasher = new_tagged_hasher(DOMAIN_EMPTY_TASK_ROOT);
    finalize_hash(hasher)
}

/// Computes the canonical hash of a block header.
///
/// Infallibility rationale:
/// The block header contains only fixed-width fields for hashing purposes,
/// so canonical encoding cannot overflow in this function.
#[must_use]
pub fn hash_header(header: &BlockHeader) -> [u8; HASH_SIZE] {
    let mut hasher = new_tagged_hasher(DOMAIN_HEADER);

    update_u64(&mut hasher, header.height);
    update_u64(&mut hasher, header.timestamp);
    update_bytes32(&mut hasher, &header.prev_hash);
    update_bytes32(&mut hasher, &header.state_root);
    update_bytes32(&mut hasher, &header.producer);
    update_u8(&mut hasher, header.block_type.code());

    finalize_hash(hasher)
}

/// Computes the canonical structured hash of a task.
///
/// Failure mode:
/// Returns [`BlockError::LengthOverflow`] if the task payload length cannot
/// be encoded into the canonical variable-length representation.
pub fn try_hash_task(task: &Task) -> Result<[u8; HASH_SIZE], BlockError> {
    let mut hasher = new_tagged_hasher(DOMAIN_TASK);

    update_bytes32(&mut hasher, &task.task_id);
    update_u8(&mut hasher, task.capability.code());
    update_u16(&mut hasher, task.target_outpost.code());
    update_bytes(&mut hasher, &task.payload)?;

    Ok(finalize_hash(hasher))
}

/// Computes the canonical structured hash of a task.
///
/// Panic policy:
/// This function is intentionally infallible for ergonomic callers that have
/// already validated task construction invariants. Production callers that
/// require explicit error handling should prefer [`try_hash_task`].
#[must_use]
pub fn hash_task(task: &Task) -> [u8; HASH_SIZE] {
    try_hash_task(task).expect("BLOCK_HASH: task payload exceeded canonical encoding limits")
}

/// Computes the canonical task-leaf hash used by task-root aggregation.
///
/// Security rationale:
/// The leaf hash is domain-separated from both the raw task hash and internal
/// Merkle nodes to prevent structural ambiguity.
pub fn try_hash_task_leaf(task: &Task) -> Result<[u8; HASH_SIZE], BlockError> {
    let task_hash = try_hash_task(task)?;

    let mut hasher = new_tagged_hasher(DOMAIN_TASK_LEAF);
    update_bytes32(&mut hasher, &task_hash);

    Ok(finalize_hash(hasher))
}

/// Computes the canonical task-leaf hash used by task-root aggregation.
///
/// Panic policy:
/// This function is intentionally infallible for callers operating on already
/// validated tasks. Fallible callers should prefer [`try_hash_task_leaf`].
#[must_use]
pub fn hash_task_leaf(task: &Task) -> [u8; HASH_SIZE] {
    try_hash_task_leaf(task).expect("BLOCK_HASH: task leaf exceeded canonical encoding limits")
}

/// Computes the canonical internal-node hash for the task Merkle tree.
///
/// Security rationale:
/// Internal-node hashing is domain-separated from leaf and root hashing to
/// prevent ambiguous reuse of the same byte material across tree layers.
#[must_use]
pub fn hash_internal_node(left: &[u8; HASH_SIZE], right: &[u8; HASH_SIZE]) -> [u8; HASH_SIZE] {
    let mut hasher = new_tagged_hasher(DOMAIN_TASK_INTERNAL);
    update_bytes32(&mut hasher, left);
    update_bytes32(&mut hasher, right);
    finalize_hash(hasher)
}

/// Computes the canonical task-root commitment for an ordered slice of tasks.
///
/// Commitment policy:
/// - Empty slice => structured empty root
/// - Non-empty slice => Merkle-style binary tree over canonical task leaves
/// - Final root => domain-separated binder over task count and Merkle root
///
/// Merkle policy:
/// - Leaves preserve the exact caller-supplied order
/// - Odd leaf counts duplicate the final leaf at that level
/// - The final root is additionally bound to task count to avoid ambiguity
pub fn calculate_task_root(tasks: &[Task]) -> Result<[u8; HASH_SIZE], BlockError> {
    if tasks.is_empty() {
        return Ok(empty_task_root());
    }

    let task_count = u64::try_from(tasks.len()).map_err(|_| BlockError::LengthOverflow)?;

    let mut leaves = Vec::with_capacity(tasks.len());
    for task in tasks {
        leaves.push(try_hash_task_leaf(task)?);
    }

    let merkle_root = merkle_root_from_leaves(leaves);

    let mut hasher = new_tagged_hasher(DOMAIN_TASK_ROOT);
    update_u64(&mut hasher, task_count);
    update_bytes32(&mut hasher, &merkle_root);

    Ok(finalize_hash(hasher))
}

/// Computes the canonical Merkle root over a non-empty owned leaf buffer.
///
/// Call contract:
/// Callers must provide at least one leaf. Empty collections must be handled
/// by [`empty_task_root`] at the outer API boundary.
fn merkle_root_from_leaves(mut leaves: Vec<[u8; HASH_SIZE]>) -> [u8; HASH_SIZE] {
    debug_assert!(!leaves.is_empty(), "leaf buffer must not be empty");

    while leaves.len() > 1 {
        let mut next_level = Vec::with_capacity(leaves.len().div_ceil(2));

        let mut i = 0usize;
        while i < leaves.len() {
            let left = leaves[i];
            let right = if i + 1 < leaves.len() {
                leaves[i + 1]
            } else {
                leaves[i]
            };

            next_level.push(hash_internal_node(&left, &right));
            i += 2;
        }

        leaves = next_level;
    }

    leaves[0]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{BlockHeader, BlockType, Capability, TargetOutpost, Task};

    fn bytes32(v: u8) -> [u8; 32] {
        [v; 32]
    }

    fn sample_header() -> BlockHeader {
        BlockHeader {
            height: 42,
            timestamp: 1_700_000_000,
            prev_hash: bytes32(1),
            state_root: bytes32(2),
            producer: bytes32(3),
            block_type: BlockType::Active,
        }
    }

    fn sample_task(payload: Vec<u8>) -> Task {
        Task {
            task_id: bytes32(9),
            capability: Capability::UserSigned,
            target_outpost: TargetOutpost::EthMainnetGateway,
            payload,
        }
    }

    #[test]
    fn header_hash_is_deterministic() {
        let header = sample_header();

        let a = hash_header(&header);
        let b = hash_header(&header);

        assert_eq!(a, b);
    }

    #[test]
    fn header_hash_changes_when_height_changes() {
        let mut a = sample_header();
        let mut b = sample_header();
        b.height += 1;

        assert_ne!(hash_header(&a), hash_header(&b));
        a.height += 1;
        assert_eq!(hash_header(&a), hash_header(&b));
    }

    #[test]
    fn task_hash_changes_when_payload_changes() {
        let a = sample_task(vec![1, 2, 3]);
        let b = sample_task(vec![1, 2, 3, 4]);

        assert_ne!(hash_task(&a), hash_task(&b));
    }

    #[test]
    fn task_hash_changes_when_target_changes() {
        let a = Task {
            task_id: bytes32(9),
            capability: Capability::UserSigned,
            target_outpost: TargetOutpost::EthMainnetGateway,
            payload: vec![1, 2, 3],
        };

        let b = Task {
            task_id: bytes32(9),
            capability: Capability::UserSigned,
            target_outpost: TargetOutpost::AovmNative,
            payload: vec![1, 2, 3],
        };

        assert_ne!(hash_task(&a), hash_task(&b));
    }

    #[test]
    fn task_leaf_hash_is_domain_separated_from_task_hash() {
        let task = sample_task(vec![1, 2, 3]);

        let task_hash = hash_task(&task);
        let leaf_hash = hash_task_leaf(&task);

        assert_ne!(task_hash, leaf_hash);
    }

    #[test]
    fn empty_task_root_is_stable_and_non_zero() {
        let a = calculate_task_root(&[]).expect("empty task root must calculate");
        let b = calculate_task_root(&[]).expect("empty task root must calculate");

        assert_eq!(a, b);
        assert_eq!(a, empty_task_root());
        assert_ne!(a, ZERO_HASH);
    }

    #[test]
    fn task_root_changes_when_order_changes() {
        let t1 = sample_task(vec![1]);
        let t2 = sample_task(vec![2]);

        let root_a =
            calculate_task_root(&[t1.clone(), t2.clone()]).expect("task root must calculate");
        let root_b = calculate_task_root(&[t2, t1]).expect("task root must calculate");

        assert_ne!(root_a, root_b);
    }

    #[test]
    fn task_root_changes_when_task_count_changes() {
        let t1 = sample_task(vec![1]);
        let t2 = sample_task(vec![2]);

        let root_a =
            calculate_task_root(std::slice::from_ref(&t1)).expect("task root must calculate");
        let root_b = calculate_task_root(&[t1, t2]).expect("task root must calculate");

        assert_ne!(root_a, root_b);
    }

    #[test]
    fn task_root_changes_when_payload_changes() {
        let a = sample_task(vec![1, 2, 3]);
        let b = sample_task(vec![1, 2, 3, 4]);

        let root_a = calculate_task_root(&[a]).expect("task root must calculate");
        let root_b = calculate_task_root(&[b]).expect("task root must calculate");

        assert_ne!(root_a, root_b);
    }

    #[test]
    fn generic_hash_is_domain_separated_from_task_hash() {
        let payload = vec![1, 2, 3, 4, 5];
        let task = sample_task(payload.clone());

        let a = compute_hash(&payload).expect("generic hash must calculate");
        let b = hash_task(&task);

        assert_ne!(a, b);
    }

    #[test]
    fn internal_node_hash_is_order_sensitive() {
        let left = bytes32(0x11);
        let right = bytes32(0x22);

        let a = hash_internal_node(&left, &right);
        let b = hash_internal_node(&right, &left);

        assert_ne!(a, b);
    }

    #[test]
    fn merkle_root_is_stable_for_odd_leaf_count() {
        let t1 = sample_task(vec![1]);
        let t2 = sample_task(vec![2]);
        let t3 = sample_task(vec![3]);

        let a = calculate_task_root(&[t1.clone(), t2.clone(), t3.clone()])
            .expect("task root must calculate");
        let b = calculate_task_root(&[t1, t2, t3]).expect("task root must calculate");

        assert_eq!(a, b);
        assert_ne!(a, ZERO_HASH);
    }

    #[test]
    fn header_hash_is_domain_separated_from_generic_hash() {
        let header = sample_header();

        let header_digest = hash_header(&header);
        let encoded = b"HEADER-LIKE-BYTES";
        let generic_digest = compute_hash(encoded).expect("generic hash must calculate");

        assert_ne!(header_digest, generic_digest);
    }
}

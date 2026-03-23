//! core/src/block/hash.rs
//!
//! AOVM Cryptographic Hashing and Commitment Module.
//!
//! This module provides deterministic, domain-separated, versioned hashing
//! primitives for block headers, tasks, and task collections.
//!
//! Security and design objectives:
//! - Stable hash outputs across platforms
//! - Explicit domain separation to prevent cross-type hash confusion
//! - Forward-compatible encoding discipline
//! - Panic-free canonical hashing paths
//! - Merkle-style task-root commitments suitable for future proof systems
//!
//! Security notes:
//! - BLAKE3 is used as the canonical digest primitive.
//! - All structured hashes are domain-separated.
//! - Variable-length fields are length-prefixed using fixed-width encoding.
//! - A protocol hash version is embedded into every structured hash.
//! - This module is quantum-aware but does not by itself make the entire chain
//!   post-quantum secure. Full post-quantum security also depends on signature,
//!   identity, and handshake layers.

use blake3::Hasher;

use super::{BlockError, BlockHeader, Task};

/// Canonical hash output size in bytes.
pub const HASH_SIZE: usize = 32;

/// Canonical zero hash constant.
///
/// This value is intentionally exposed for callers that require an explicit
/// constant, but structured empty roots should prefer [`empty_task_root`].
pub const ZERO_HASH: [u8; HASH_SIZE] = [0u8; HASH_SIZE];

/// Version tag for the hashing format.
///
/// This value must be incremented if the canonical hashing layout changes in a
/// backward-incompatible manner.
pub const HASH_FORMAT_VERSION: u8 = 1;

/// Global protocol namespace used for all block-domain hash derivations.
const PROTOCOL_HASH_NAMESPACE: &[u8] = b"AOXC/AOVM/BLOCK/HASH";

/// Domain separator for generic hashing.
const DOMAIN_GENERIC: &[u8] = b"GENERIC";

/// Domain separator for header hashing.
const DOMAIN_HEADER: &[u8] = b"HEADER";

/// Domain separator for task hashing.
const DOMAIN_TASK: &[u8] = b"TASK";

/// Domain separator for task collection hashing.
const DOMAIN_TASK_ROOT: &[u8] = b"TASK_ROOT";

/// Domain separator for the canonical empty task root.
const DOMAIN_EMPTY_TASK_ROOT: &[u8] = b"EMPTY_TASK_ROOT";

/// Domain separator for task leaf hashing.
const DOMAIN_TASK_LEAF: &[u8] = b"TASK_LEAF";

/// Domain separator for internal-node hashing in the Merkle tree.
const DOMAIN_TASK_INTERNAL: &[u8] = b"TASK_INTERNAL";

/// Returns a fresh BLAKE3 hasher initialized with namespace, domain, and version tags.
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

/// Encodes a `u8` into the hash stream.
#[inline]
fn update_u8(hasher: &mut Hasher, value: u8) {
    hasher.update(&[value]);
}

/// Encodes a `u16` into the hash stream using little-endian order.
#[inline]
fn update_u16(hasher: &mut Hasher, value: u16) {
    hasher.update(&value.to_le_bytes());
}

/// Encodes a `u64` into the hash stream using little-endian order.
#[inline]
fn update_u64(hasher: &mut Hasher, value: u64) {
    hasher.update(&value.to_le_bytes());
}

/// Encodes a fixed 32-byte value into the hash stream.
#[inline]
fn update_bytes32(hasher: &mut Hasher, value: &[u8; HASH_SIZE]) {
    hasher.update(value);
}

/// Encodes a variable-length byte slice using a 64-bit length prefix.
#[inline]
fn update_bytes(hasher: &mut Hasher, value: &[u8]) -> Result<(), BlockError> {
    let len = u64::try_from(value.len()).map_err(|_| BlockError::LengthOverflow)?;
    update_u64(hasher, len);
    hasher.update(value);
    Ok(())
}

/// Finalizes the hasher into the canonical 32-byte digest format.
#[inline]
fn finalize_hash(hasher: Hasher) -> [u8; HASH_SIZE] {
    *hasher.finalize().as_bytes()
}

/// Computes a generic domain-separated BLAKE3 hash over an arbitrary byte slice.
///
/// This function is intended for non-structured utility hashing. Consensus-
/// critical structures should prefer explicit structured hash functions.
pub fn compute_hash(data: &[u8]) -> Result<[u8; HASH_SIZE], BlockError> {
    let mut hasher = new_tagged_hasher(DOMAIN_GENERIC);
    update_bytes(&mut hasher, data)?;
    Ok(finalize_hash(hasher))
}

/// Returns the canonical empty task root.
///
/// This root is domain-separated and does not equal [`ZERO_HASH`].
#[must_use]
pub fn empty_task_root() -> [u8; HASH_SIZE] {
    let hasher = new_tagged_hasher(DOMAIN_EMPTY_TASK_ROOT);
    finalize_hash(hasher)
}

/// Computes the canonical hash of a block header.
///
/// This operation is infallible because all encoded header fields are fixed-width.
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

/// Computes the canonical hash of a task.
pub fn try_hash_task(task: &Task) -> Result<[u8; HASH_SIZE], BlockError> {
    let mut hasher = new_tagged_hasher(DOMAIN_TASK);

    update_bytes32(&mut hasher, &task.task_id);
    update_u8(&mut hasher, task.capability.code());
    update_u16(&mut hasher, task.target_outpost.code());
    update_bytes(&mut hasher, &task.payload)?;

    Ok(finalize_hash(hasher))
}

/// Computes the canonical hash of a task.
#[must_use]
pub fn hash_task(task: &Task) -> [u8; HASH_SIZE] {
    try_hash_task(task).expect("BLOCK_HASH: task payload exceeded canonical encoding limits")
}

/// Computes the canonical task leaf hash used in task-root aggregation.
///
/// The leaf is domain-separated from both the raw task hash and internal nodes.
pub fn try_hash_task_leaf(task: &Task) -> Result<[u8; HASH_SIZE], BlockError> {
    let task_hash = try_hash_task(task)?;

    let mut hasher = new_tagged_hasher(DOMAIN_TASK_LEAF);
    update_bytes32(&mut hasher, &task_hash);

    Ok(finalize_hash(hasher))
}

/// Computes the canonical task leaf hash used in task-root aggregation.
#[must_use]
pub fn hash_task_leaf(task: &Task) -> [u8; HASH_SIZE] {
    try_hash_task_leaf(task).expect("BLOCK_HASH: task leaf exceeded canonical encoding limits")
}

/// Computes the canonical internal-node hash for task-root tree constructions.
#[must_use]
pub fn hash_internal_node(left: &[u8; HASH_SIZE], right: &[u8; HASH_SIZE]) -> [u8; HASH_SIZE] {
    let mut hasher = new_tagged_hasher(DOMAIN_TASK_INTERNAL);
    update_bytes32(&mut hasher, left);
    update_bytes32(&mut hasher, right);
    finalize_hash(hasher)
}

/// Computes the canonical task-root commitment for a slice of tasks.
///
/// Current strategy:
/// - Empty slice => structured empty root
/// - Non-empty slice => Merkle-style binary tree over task leaves
/// - Final commitment => domain-separated root binder including task count
///
/// Merkle policy:
/// - Leaves are ordered exactly as supplied
/// - When a level has an odd number of nodes, the final node is duplicated
/// - The resulting Merkle root is bound together with the task count under
///   a dedicated root domain to avoid structural ambiguity
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

/// Computes the canonical Merkle root over an owned leaf buffer.
///
/// Callers must supply a non-empty leaf set. Empty roots must use
/// [`empty_task_root`] instead.
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

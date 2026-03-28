// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! core/src/block/assembly.rs
//!
//! Canonical block assembly planning module.
//!
//! This module derives a deterministic execution-oriented assembly plan from a
//! validated active block. The plan groups tasks into canonical execution lanes,
//! produces lane-level commitments, and derives a domain-separated execution
//! root suitable for higher-layer scheduling, reporting, and future proof
//! systems.
//!
//! Design objectives:
//! - Deterministic lane assignment across all nodes
//! - Stable lane ordering and commitment encoding
//! - Canonical execution-root derivation bound to block context
//! - Panic-free overflow handling for counters and byte accounting
//! - Minimal ambiguity around empty or malformed block inputs

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use super::hash::{compute_hash, HASH_SIZE};
use super::{Block, BlockError, BlockType, TargetOutpost, Task};

/// Canonical format version for block assembly commitments.
///
/// Security rationale:
/// Any backward-incompatible change to execution-root encoding, lane layout,
/// or commitment preimage format must increment this version.
pub const ASSEMBLY_FORMAT_VERSION: u8 = 1;

/// Domain separator for the canonical execution root.
const EXECUTION_ROOT_DOMAIN: &[u8] = b"AOXC/AOVM/BLOCK/ASSEMBLY/EXECUTION_ROOT";

/// Domain separator for lane-level commitment roots.
const LANE_ROOT_DOMAIN: &[u8] = b"AOXC/AOVM/BLOCK/ASSEMBLY/LANE_ROOT";

/// Canonical lane identifier.
///
/// Security rationale:
/// Lane IDs are protocol-visible routing artifacts and therefore use a compact
/// fixed-width representation suitable for stable commitment encoding.
pub type LaneId = u16;

/// Enumerates the canonical execution lanes recognized by the block assembler.
///
/// Audit rationale:
/// The lane set is intentionally explicit. Consensus-sensitive grouping must
/// never depend on ad hoc string labels or runtime-discovered categories.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AssemblyLane {
    /// Ethereum-bound external routing lane.
    EthereumGateway = 0,

    /// Solana-bound reward or bridge lane.
    SolanaProgram = 1,

    /// Base settlement lane.
    BaseSettlement = 2,

    /// Native AOVM execution lane.
    NativeExecution = 3,
}

impl AssemblyLane {
    /// Returns the canonical compact numeric representation.
    #[must_use]
    pub const fn code(self) -> LaneId {
        self as LaneId
    }

    /// Maps a canonical target outpost into a canonical assembly lane.
    #[must_use]
    pub const fn from_target_outpost(target: TargetOutpost) -> Self {
        match target {
            TargetOutpost::EthMainnetGateway => Self::EthereumGateway,
            TargetOutpost::SolanaRewardProgram => Self::SolanaProgram,
            TargetOutpost::BaseSettlementRouter => Self::BaseSettlement,
            TargetOutpost::AovmNative => Self::NativeExecution,
        }
    }
}

/// Canonical execution-lane commitment derived from an ordered task subset.
///
/// Security rationale:
/// The lane commitment binds lane identity, ordered task membership, task count,
/// byte totals, and a domain-separated lane root into a single deterministic
/// structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssemblyLaneCommitment {
    /// Canonical lane classification.
    pub lane: AssemblyLane,

    /// Ordered task count assigned to this lane.
    pub task_count: u32,

    /// Aggregate payload size across all tasks assigned to this lane.
    pub total_payload_bytes: u32,

    /// Domain-separated commitment over the ordered lane task set.
    pub lane_root: [u8; HASH_SIZE],
}

/// Canonical block assembly plan derived from an active block.
///
/// Security rationale:
/// This structure is intended to be reproducible from the same validated block
/// on every honest node. It is deliberately immutable and serializable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalBlockAssemblyPlan {
    /// Assembly format version.
    pub format_version: u8,

    /// Height of the source block.
    pub block_height: u64,

    /// Header hash of the source block.
    pub block_hash: [u8; HASH_SIZE],

    /// Previous block hash from the source block header.
    pub prev_hash: [u8; HASH_SIZE],

    /// Block producer identity.
    pub producer: [u8; HASH_SIZE],

    /// Number of tasks in the source block.
    pub task_count: u32,

    /// Aggregate payload size across all block tasks.
    pub total_payload_bytes: u32,

    /// Canonical lane commitments sorted by lane code.
    pub lane_commitments: Vec<AssemblyLaneCommitment>,

    /// Domain-separated execution root derived from block context and lane commitments.
    pub execution_root: [u8; HASH_SIZE],
}

/// Error surface for canonical block assembly.
///
/// Audit rationale:
/// Assembly failures are structured so callers can distinguish malformed input,
/// overflow conditions, and cryptographic commitment failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssemblyError {
    /// Assembly requires an active block.
    BlockTypeNotAssemblable { found: BlockType },

    /// Assembly requires at least one task.
    EmptyTaskSet,

    /// Task count exceeded the supported `u32` encoding range.
    TaskCountOverflow { count: usize },

    /// Aggregate payload bytes exceeded the supported `u32` encoding range.
    PayloadLengthOverflow { size: usize },

    /// A task-level hashing operation failed.
    TaskHashingFailed { index: usize, source: BlockError },

    /// A domain-separated commitment could not be derived.
    CommitmentHashingFailed { source: BlockError },
}

impl fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockTypeNotAssemblable { found } => {
                write!(f, "block type {:?} is not eligible for canonical assembly", found)
            }
            Self::EmptyTaskSet => write!(f, "canonical assembly requires at least one task"),
            Self::TaskCountOverflow { count } => {
                write!(f, "task count exceeds supported range: {}", count)
            }
            Self::PayloadLengthOverflow { size } => {
                write!(f, "payload length exceeds supported range: {}", size)
            }
            Self::TaskHashingFailed { index, source } => {
                write!(f, "task hashing failed at index {}: {}", index, source)
            }
            Self::CommitmentHashingFailed { source } => {
                write!(f, "assembly commitment hashing failed: {}", source)
            }
        }
    }
}

impl std::error::Error for AssemblyError {}

impl CanonicalBlockAssemblyPlan {
    /// Derives a canonical assembly plan from a validated active block.
    ///
    /// Validation contract:
    /// - the block itself must validate successfully,
    /// - only active blocks are assemblable,
    /// - the task set must be non-empty,
    /// - task count and byte totals must fit into canonical encoded bounds,
    /// - lane commitments are emitted in stable lane order,
    /// - the execution root is derived from block context and lane commitments.
    pub fn from_block(block: &Block) -> Result<Self, AssemblyError> {
        block
            .validate()
            .map_err(|source| AssemblyError::TaskHashingFailed { index: 0, source })?;

        if block.header.block_type != BlockType::Active {
            return Err(AssemblyError::BlockTypeNotAssemblable {
                found: block.header.block_type,
            });
        }

        if block.tasks.is_empty() {
            return Err(AssemblyError::EmptyTaskSet);
        }

        let task_count = u32::try_from(block.tasks.len()).map_err(|_| {
            AssemblyError::TaskCountOverflow {
                count: block.tasks.len(),
            }
        })?;

        let total_payload_bytes_usize = block.total_payload_bytes();
        let total_payload_bytes = u32::try_from(total_payload_bytes_usize).map_err(|_| {
            AssemblyError::PayloadLengthOverflow {
                size: total_payload_bytes_usize,
            }
        })?;

        let lane_commitments = build_lane_commitments(&block.tasks)?;
        let execution_root = derive_execution_root(block, &lane_commitments)?;

        Ok(Self {
            format_version: ASSEMBLY_FORMAT_VERSION,
            block_height: block.header.height,
            block_hash: block.header_hash(),
            prev_hash: block.header.prev_hash,
            producer: block.header.producer,
            task_count,
            total_payload_bytes,
            lane_commitments,
            execution_root,
        })
    }
}

/// Builds deterministic lane commitments from an ordered task slice.
///
/// Determinism guarantees:
/// - lane grouping is canonical via `BTreeMap`,
/// - task order inside a lane preserves original block order,
/// - each lane root commits to lane identity, task order, and task hashes.
pub fn build_lane_commitments(
    tasks: &[Task],
) -> Result<Vec<AssemblyLaneCommitment>, AssemblyError> {
    let mut lanes: BTreeMap<AssemblyLane, Vec<(usize, &Task)>> = BTreeMap::new();

    for (index, task) in tasks.iter().enumerate() {
        let lane = AssemblyLane::from_target_outpost(task.target_outpost);
        lanes.entry(lane).or_default().push((index, task));
    }

    let mut commitments = Vec::with_capacity(lanes.len());

    for (lane, lane_tasks) in lanes {
        let task_count = u32::try_from(lane_tasks.len()).map_err(|_| {
            AssemblyError::TaskCountOverflow {
                count: lane_tasks.len(),
            }
        })?;

        let mut total_payload_bytes_usize = 0usize;
        for (_, task) in &lane_tasks {
            total_payload_bytes_usize = total_payload_bytes_usize
                .checked_add(task.payload_len())
                .ok_or(AssemblyError::PayloadLengthOverflow {
                    size: usize::MAX,
                })?;
        }

        let total_payload_bytes =
            u32::try_from(total_payload_bytes_usize).map_err(|_| {
                AssemblyError::PayloadLengthOverflow {
                    size: total_payload_bytes_usize,
                }
            })?;

        let lane_root = derive_lane_root(lane, &lane_tasks)?;

        commitments.push(AssemblyLaneCommitment {
            lane,
            task_count,
            total_payload_bytes,
            lane_root,
        });
    }

    Ok(commitments)
}

/// Derives the canonical root for a single lane.
///
/// Commitment preimage:
/// - assembly format version
/// - lane code
/// - ordered task count
/// - ordered entries of:
///   - original block ordinal
///   - canonical task hash
///   - task identifier
///   - payload length
fn derive_lane_root(
    lane: AssemblyLane,
    lane_tasks: &[(usize, &Task)],
) -> Result<[u8; HASH_SIZE], AssemblyError> {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(LANE_ROOT_DOMAIN);
    bytes.push(0x00);
    bytes.push(ASSEMBLY_FORMAT_VERSION);
    bytes.extend_from_slice(&lane.code().to_le_bytes());

    let lane_task_count = u32::try_from(lane_tasks.len()).map_err(|_| {
        AssemblyError::TaskCountOverflow {
            count: lane_tasks.len(),
        }
    })?;
    bytes.extend_from_slice(&lane_task_count.to_le_bytes());

    for (index, task) in lane_tasks {
        let task_hash = task
            .hash()
            .map_err(|source| AssemblyError::TaskHashingFailed {
                index: *index,
                source,
            })?;

        let ordinal = u32::try_from(*index).map_err(|_| AssemblyError::TaskCountOverflow {
            count: *index,
        })?;

        let payload_len = u32::try_from(task.payload_len()).map_err(|_| {
            AssemblyError::PayloadLengthOverflow {
                size: task.payload_len(),
            }
        })?;

        bytes.extend_from_slice(&ordinal.to_le_bytes());
        bytes.extend_from_slice(&task_hash);
        bytes.extend_from_slice(&task.task_id);
        bytes.extend_from_slice(&payload_len.to_le_bytes());
    }

    compute_hash(&bytes).map_err(|source| AssemblyError::CommitmentHashingFailed { source })
}

/// Derives the canonical execution root for the entire block assembly plan.
///
/// Commitment preimage:
/// - execution-root domain
/// - assembly format version
/// - block height
/// - block hash
/// - previous hash
/// - producer
/// - task count
/// - total payload bytes
/// - lane commitment count
/// - ordered lane commitments
pub fn derive_execution_root(
    block: &Block,
    lane_commitments: &[AssemblyLaneCommitment],
) -> Result<[u8; HASH_SIZE], AssemblyError> {
    let task_count = u32::try_from(block.tasks.len()).map_err(|_| {
        AssemblyError::TaskCountOverflow {
            count: block.tasks.len(),
        }
    })?;

    let total_payload_bytes_usize = block.total_payload_bytes();
    let total_payload_bytes =
        u32::try_from(total_payload_bytes_usize).map_err(|_| {
            AssemblyError::PayloadLengthOverflow {
                size: total_payload_bytes_usize,
            }
        })?;

    let lane_count = u32::try_from(lane_commitments.len()).map_err(|_| {
        AssemblyError::TaskCountOverflow {
            count: lane_commitments.len(),
        }
    })?;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(EXECUTION_ROOT_DOMAIN);
    bytes.push(0x00);
    bytes.push(ASSEMBLY_FORMAT_VERSION);
    bytes.extend_from_slice(&block.header.height.to_le_bytes());
    bytes.extend_from_slice(&block.header_hash());
    bytes.extend_from_slice(&block.header.prev_hash);
    bytes.extend_from_slice(&block.header.producer);
    bytes.extend_from_slice(&task_count.to_le_bytes());
    bytes.extend_from_slice(&total_payload_bytes.to_le_bytes());
    bytes.extend_from_slice(&lane_count.to_le_bytes());

    for commitment in lane_commitments {
        bytes.extend_from_slice(&commitment.lane.code().to_le_bytes());
        bytes.extend_from_slice(&commitment.task_count.to_le_bytes());
        bytes.extend_from_slice(&commitment.total_payload_bytes.to_le_bytes());
        bytes.extend_from_slice(&commitment.lane_root);
    }

    compute_hash(&bytes).map_err(|source| AssemblyError::CommitmentHashingFailed { source })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{Capability, TargetOutpost, Task};

    fn bytes32(v: u8) -> [u8; 32] {
        [v; 32]
    }

    fn task(task_id: u8, target: TargetOutpost, payload_len: usize) -> Task {
        Task::new(
            bytes32(task_id),
            Capability::UserSigned,
            target,
            vec![7u8; payload_len],
        )
        .expect("task must construct successfully")
    }

    fn active_block(tasks: Vec<Task>) -> Block {
        Block::new_active_with_timestamp(7, 1000, bytes32(10), bytes32(20), bytes32(30), tasks)
            .expect("active block must construct successfully")
    }

    #[test]
    fn assembly_plan_constructs_for_valid_active_block() {
        let block = active_block(vec![
            task(1, TargetOutpost::EthMainnetGateway, 10),
            task(2, TargetOutpost::AovmNative, 20),
        ]);

        let plan = CanonicalBlockAssemblyPlan::from_block(&block)
            .expect("assembly plan must construct successfully");

        assert_eq!(plan.format_version, ASSEMBLY_FORMAT_VERSION);
        assert_eq!(plan.block_height, 7);
        assert_eq!(plan.task_count, 2);
        assert_eq!(plan.total_payload_bytes, 30);
        assert_eq!(plan.lane_commitments.len(), 2);
        assert_ne!(plan.execution_root, ZERO_HASH);
    }

    #[test]
    fn assembly_rejects_heartbeat_block() {
        let block = Block::new_heartbeat_with_timestamp(1, 100, bytes32(10), bytes32(30))
            .expect("heartbeat block must construct successfully");

        let err = CanonicalBlockAssemblyPlan::from_block(&block)
            .expect_err("heartbeat block must not be assemblable");

        assert_eq!(
            err,
            AssemblyError::BlockTypeNotAssemblable {
                found: BlockType::Heartbeat
            }
        );
    }

    #[test]
    fn lane_commitments_are_emitted_in_canonical_lane_order() {
        let tasks = vec![
            task(1, TargetOutpost::AovmNative, 10),
            task(2, TargetOutpost::EthMainnetGateway, 10),
            task(3, TargetOutpost::BaseSettlementRouter, 10),
        ];

        let commitments =
            build_lane_commitments(&tasks).expect("lane commitments must build successfully");

        let lanes: Vec<AssemblyLane> = commitments.iter().map(|c| c.lane).collect();
        assert_eq!(
            lanes,
            vec![
                AssemblyLane::EthereumGateway,
                AssemblyLane::BaseSettlement,
                AssemblyLane::NativeExecution
            ]
        );
    }

    #[test]
    fn execution_root_is_deterministic_for_identical_block_input() {
        let block = active_block(vec![
            task(1, TargetOutpost::EthMainnetGateway, 10),
            task(2, TargetOutpost::SolanaRewardProgram, 12),
            task(3, TargetOutpost::AovmNative, 14),
        ]);

        let plan_a = CanonicalBlockAssemblyPlan::from_block(&block)
            .expect("first assembly plan must construct");
        let plan_b = CanonicalBlockAssemblyPlan::from_block(&block)
            .expect("second assembly plan must construct");

        assert_eq!(plan_a, plan_b);
    }

    #[test]
    fn execution_root_changes_when_task_order_changes() {
        let block_a = active_block(vec![
            task(1, TargetOutpost::EthMainnetGateway, 10),
            task(2, TargetOutpost::EthMainnetGateway, 10),
        ]);

        let block_b = active_block(vec![
            task(2, TargetOutpost::EthMainnetGateway, 10),
            task(1, TargetOutpost::EthMainnetGateway, 10),
        ]);

        let plan_a = CanonicalBlockAssemblyPlan::from_block(&block_a)
            .expect("first assembly plan must construct");
        let plan_b = CanonicalBlockAssemblyPlan::from_block(&block_b)
            .expect("second assembly plan must construct");

        assert_ne!(plan_a.execution_root, plan_b.execution_root);
    }

    #[test]
    fn lane_root_changes_when_payload_changes() {
        let task_a = task(1, TargetOutpost::AovmNative, 10);
        let task_b = task(1, TargetOutpost::AovmNative, 11);

        let root_a = build_lane_commitments(&[task_a])
            .expect("lane commitments must build")[0]
            .lane_root;
        let root_b = build_lane_commitments(&[task_b])
            .expect("lane commitments must build")[0]
            .lane_root;

        assert_ne!(root_a, root_b);
    }
}

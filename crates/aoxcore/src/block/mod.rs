//! core/src/block/mod.rs
//!
//! AOVM Block Domain Module.
//!
//! This module defines the canonical block-domain data structures, invariants,
//! constructors, and validation rules for AOVM.

pub mod error;
pub mod hash;

pub use error::BlockError;
pub use hash::{
    HASH_FORMAT_VERSION, HASH_SIZE, ZERO_HASH, calculate_task_root, compute_hash, empty_task_root,
    hash_header, hash_internal_node, hash_task, hash_task_leaf, try_hash_task, try_hash_task_leaf,
};

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum number of tasks allowed inside a single active block.
pub const MAX_TASKS_PER_BLOCK: usize = 1_024;

/// Maximum payload size allowed for a single task in bytes.
pub const MAX_TASK_PAYLOAD_BYTES: usize = 64 * 1024;

/// Canonical zero state root used by heartbeat blocks.
pub const ZERO_STATE_ROOT: [u8; 32] = [0u8; 32];

/// Enumerates the lifecycle role of a block within the protocol.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlockType {
    /// Regular execution block containing one or more tasks.
    Active = 0,

    /// Lightweight keep-alive block emitted during idle periods.
    Heartbeat = 1,

    /// Maintenance block emitted for pruning and compaction workflows.
    EpochPrune = 2,
}

impl BlockType {
    /// Returns the compact wire-friendly numeric representation.
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }
}

/// Enumerates the authorization or attestation class of a task.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Task authorized by a direct user signature.
    UserSigned = 0,

    /// Task authorized through an AI attestation path.
    AiAttested = 1,

    /// Task authorized through governance or DAO policy.
    DaoApproved = 2,
}

impl Capability {
    /// Returns the compact wire-friendly numeric representation.
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }
}

/// Enumerates the logical destination for routed task execution.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetOutpost {
    /// Ethereum mainnet gateway.
    EthMainnetGateway = 0,

    /// Solana reward program.
    SolanaRewardProgram = 1,

    /// Base settlement router.
    BaseSettlementRouter = 2,

    /// Native AOVM execution lane.
    AovmNative = 3,
}

impl TargetOutpost {
    /// Returns the compact wire-friendly numeric representation.
    #[must_use]
    pub const fn code(self) -> u16 {
        self as u16
    }
}

/// Canonical block header.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Monotonically increasing block height.
    pub height: u64,

    /// Unix timestamp in seconds.
    pub timestamp: u64,

    /// Canonical hash of the previous block header.
    pub prev_hash: [u8; 32],

    /// Cryptographic commitment to the resulting AOVM state.
    pub state_root: [u8; 32],

    /// Block producer identity or public key.
    pub producer: [u8; 32],

    /// Semantic block type.
    pub block_type: BlockType,
}

/// Canonical task object.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier.
    pub task_id: [u8; 32],

    /// Authorization class for the task.
    pub capability: Capability,

    /// Logical execution destination.
    pub target_outpost: TargetOutpost,

    /// Opaque routed payload.
    pub payload: Vec<u8>,
}

impl Task {
    /// Creates a validated task.
    pub fn new(
        task_id: [u8; 32],
        capability: Capability,
        target_outpost: TargetOutpost,
        payload: Vec<u8>,
    ) -> Result<Self, BlockError> {
        let task = Self {
            task_id,
            capability,
            target_outpost,
            payload,
        };

        task.validate()?;
        Ok(task)
    }

    /// Validates task-level invariants.
    pub fn validate(&self) -> Result<(), BlockError> {
        if self.payload.is_empty() {
            return Err(BlockError::EmptyTaskPayload);
        }

        if self.payload.len() > MAX_TASK_PAYLOAD_BYTES {
            return Err(BlockError::TaskPayloadTooLarge {
                size: self.payload.len(),
                max: MAX_TASK_PAYLOAD_BYTES,
            });
        }

        Ok(())
    }

    /// Returns the payload size in bytes.
    #[must_use]
    pub fn payload_len(&self) -> usize {
        self.payload.len()
    }

    /// Returns the canonical task hash.
    pub fn hash(&self) -> Result<[u8; 32], BlockError> {
        hash::hash_task(self)
    }
}

/// Canonical block object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub tasks: Vec<Task>,
}

impl Block {
    /// Creates a validated active block.
    pub fn new_active(
        height: u64,
        prev_hash: [u8; 32],
        state_root: [u8; 32],
        producer: [u8; 32],
        tasks: Vec<Task>,
    ) -> Result<Self, BlockError> {
        Self::build(
            height,
            current_time()?,
            prev_hash,
            state_root,
            producer,
            BlockType::Active,
            tasks,
        )
    }

    /// Creates a validated active block with an explicit timestamp.
    pub fn new_active_with_timestamp(
        height: u64,
        timestamp: u64,
        prev_hash: [u8; 32],
        state_root: [u8; 32],
        producer: [u8; 32],
        tasks: Vec<Task>,
    ) -> Result<Self, BlockError> {
        Self::build(
            height,
            timestamp,
            prev_hash,
            state_root,
            producer,
            BlockType::Active,
            tasks,
        )
    }

    /// Creates a validated heartbeat block.
    pub fn new_heartbeat(
        height: u64,
        prev_hash: [u8; 32],
        producer: [u8; 32],
    ) -> Result<Self, BlockError> {
        Self::build(
            height,
            current_time()?,
            prev_hash,
            ZERO_STATE_ROOT,
            producer,
            BlockType::Heartbeat,
            Vec::new(),
        )
    }

    /// Creates a validated heartbeat block with an explicit timestamp.
    pub fn new_heartbeat_with_timestamp(
        height: u64,
        timestamp: u64,
        prev_hash: [u8; 32],
        producer: [u8; 32],
    ) -> Result<Self, BlockError> {
        Self::build(
            height,
            timestamp,
            prev_hash,
            ZERO_STATE_ROOT,
            producer,
            BlockType::Heartbeat,
            Vec::new(),
        )
    }

    /// Creates a validated epoch-prune block.
    pub fn new_epoch_prune(
        height: u64,
        prev_hash: [u8; 32],
        state_root: [u8; 32],
        producer: [u8; 32],
    ) -> Result<Self, BlockError> {
        Self::build(
            height,
            current_time()?,
            prev_hash,
            state_root,
            producer,
            BlockType::EpochPrune,
            Vec::new(),
        )
    }

    /// Creates a validated epoch-prune block with an explicit timestamp.
    pub fn new_epoch_prune_with_timestamp(
        height: u64,
        timestamp: u64,
        prev_hash: [u8; 32],
        state_root: [u8; 32],
        producer: [u8; 32],
    ) -> Result<Self, BlockError> {
        Self::build(
            height,
            timestamp,
            prev_hash,
            state_root,
            producer,
            BlockType::EpochPrune,
            Vec::new(),
        )
    }

    /// Builds a block and validates all domain invariants before returning it.
    fn build(
        height: u64,
        timestamp: u64,
        prev_hash: [u8; 32],
        state_root: [u8; 32],
        producer: [u8; 32],
        block_type: BlockType,
        tasks: Vec<Task>,
    ) -> Result<Self, BlockError> {
        let block = Self {
            header: BlockHeader {
                height,
                timestamp,
                prev_hash,
                state_root,
                producer,
                block_type,
            },
            tasks,
        };

        block.validate()?;
        Ok(block)
    }

    /// Validates block-level invariants.
    pub fn validate(&self) -> Result<(), BlockError> {
        self.validate_header_semantics()?;

        if self.tasks.len() > MAX_TASKS_PER_BLOCK {
            return Err(BlockError::TooManyTasks {
                count: self.tasks.len(),
                max: MAX_TASKS_PER_BLOCK,
            });
        }

        match self.header.block_type {
            BlockType::Active => {
                if self.tasks.is_empty() {
                    return Err(BlockError::ActiveBlockRequiresTasks);
                }

                for task in &self.tasks {
                    task.validate()?;
                }
            }
            BlockType::Heartbeat => {
                if !self.tasks.is_empty() {
                    return Err(BlockError::HeartbeatBlockMustNotContainTasks);
                }

                if self.header.state_root != ZERO_STATE_ROOT {
                    return Err(BlockError::HeartbeatBlockMustUseZeroStateRoot);
                }
            }
            BlockType::EpochPrune => {
                if !self.tasks.is_empty() {
                    return Err(BlockError::EpochPruneBlockMustNotContainTasks);
                }
            }
        }

        Ok(())
    }

    /// Validates basic header semantics that are independent of chain context.
    fn validate_header_semantics(&self) -> Result<(), BlockError> {
        if self.header.timestamp == 0 {
            return Err(BlockError::InvalidTimestamp);
        }

        if self.header.producer == ZERO_HASH {
            return Err(BlockError::InvalidPreviousHash);
        }

        Ok(())
    }

    /// Validates direct parent linkage against an expected parent block.
    pub fn validate_parent_link(&self, parent: &Block) -> Result<(), BlockError> {
        self.validate()?;
        parent.validate()?;

        if self.header.height != parent.header.height.saturating_add(1) {
            return Err(BlockError::InvalidBlockHeight);
        }

        let expected_prev_hash = parent.header_hash();
        if self.header.prev_hash != expected_prev_hash {
            return Err(BlockError::InvalidPreviousHash);
        }

        if self.header.timestamp < parent.header.timestamp {
            return Err(BlockError::InvalidTimestamp);
        }

        Ok(())
    }

    /// Returns the task count.
    #[must_use]
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Returns the aggregated payload size in bytes using saturating arithmetic.
    #[must_use]
    pub fn total_payload_bytes(&self) -> usize {
        let mut total = 0usize;

        for task in &self.tasks {
            total = total.saturating_add(task.payload_len());
        }

        total
    }

    /// Returns `true` if the block is active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.header.block_type == BlockType::Active
    }

    /// Returns `true` if the block is a heartbeat block.
    #[must_use]
    pub fn is_heartbeat(&self) -> bool {
        self.header.block_type == BlockType::Heartbeat
    }

    /// Returns `true` if the block is an epoch-prune block.
    #[must_use]
    pub fn is_epoch_prune(&self) -> bool {
        self.header.block_type == BlockType::EpochPrune
    }

    /// Returns the compact numeric block type code.
    #[must_use]
    pub fn block_type_code(&self) -> u8 {
        self.header.block_type.code()
    }

    /// Returns the canonical header hash.
    #[must_use]
    pub fn header_hash(&self) -> [u8; 32] {
        hash::hash_header(&self.header)
    }

    /// Returns the canonical task-root commitment.
    pub fn task_root(&self) -> Result<[u8; 32], BlockError> {
        hash::calculate_task_root(&self.tasks)
    }

    /// Returns the canonical task-root commitment.
    pub fn try_task_root(&self) -> Result<[u8; 32], BlockError> {
        self.task_root()
    }

    /// Returns `true` if the block contains duplicate task identifiers.
    ///
    /// This helper is exposed for production callers that want stricter policy
    /// enforcement without changing the current `BlockError` contract.
    #[must_use]
    pub fn has_duplicate_task_ids(&self) -> bool {
        let mut seen: HashSet<[u8; 32]> = HashSet::with_capacity(self.tasks.len());

        for task in &self.tasks {
            if !seen.insert(task.task_id) {
                return true;
            }
        }

        false
    }

    /// Returns `true` when the block contains no tasks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

/// Returns the current Unix timestamp in seconds.
fn current_time() -> Result<u64, BlockError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| BlockError::InvalidSystemTime)?;

    Ok(duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes32(v: u8) -> [u8; 32] {
        [v; 32]
    }

    fn valid_task() -> Task {
        Task::new(
            bytes32(1),
            Capability::UserSigned,
            TargetOutpost::EthMainnetGateway,
            vec![1, 2, 3],
        )
        .expect("valid task must construct successfully")
    }

    #[test]
    fn active_block_constructs_successfully() {
        let block = Block::new_active_with_timestamp(
            1,
            100,
            bytes32(10),
            bytes32(20),
            bytes32(30),
            vec![valid_task()],
        )
        .expect("active block must construct successfully");

        assert!(block.is_active());
        assert_eq!(block.task_count(), 1);
    }

    #[test]
    fn heartbeat_block_constructs_successfully() {
        let block = Block::new_heartbeat_with_timestamp(2, 200, bytes32(11), bytes32(31))
            .expect("heartbeat block must construct successfully");

        assert!(block.is_heartbeat());
        assert_eq!(block.task_count(), 0);
        assert_eq!(block.header.state_root, ZERO_STATE_ROOT);
    }

    #[test]
    fn epoch_prune_block_constructs_successfully() {
        let block =
            Block::new_epoch_prune_with_timestamp(3, 300, bytes32(12), bytes32(22), bytes32(32))
                .expect("epoch-prune block must construct successfully");

        assert!(block.is_epoch_prune());
        assert_eq!(block.task_count(), 0);
    }

    #[test]
    fn active_block_without_tasks_is_rejected() {
        let result =
            Block::new_active_with_timestamp(1, 100, bytes32(10), bytes32(20), bytes32(30), vec![]);

        assert_eq!(result, Err(BlockError::ActiveBlockRequiresTasks));
    }

    #[test]
    fn empty_task_payload_is_rejected() {
        let result = Task::new(
            bytes32(1),
            Capability::AiAttested,
            TargetOutpost::AovmNative,
            Vec::new(),
        );

        assert_eq!(result, Err(BlockError::EmptyTaskPayload));
    }

    #[test]
    fn heartbeat_with_non_zero_state_root_is_rejected() {
        let block = Block {
            header: BlockHeader {
                height: 5,
                timestamp: 123,
                prev_hash: bytes32(10),
                state_root: bytes32(99),
                producer: bytes32(30),
                block_type: BlockType::Heartbeat,
            },
            tasks: Vec::new(),
        };

        assert_eq!(
            block.validate(),
            Err(BlockError::HeartbeatBlockMustUseZeroStateRoot)
        );
    }

    #[test]
    fn zero_timestamp_is_rejected() {
        let block = Block {
            header: BlockHeader {
                height: 1,
                timestamp: 0,
                prev_hash: bytes32(10),
                state_root: bytes32(20),
                producer: bytes32(30),
                block_type: BlockType::Heartbeat,
            },
            tasks: Vec::new(),
        };

        assert_eq!(block.validate(), Err(BlockError::InvalidTimestamp));
    }

    #[test]
    fn zero_producer_is_rejected() {
        let block = Block {
            header: BlockHeader {
                height: 1,
                timestamp: 1,
                prev_hash: bytes32(10),
                state_root: ZERO_STATE_ROOT,
                producer: ZERO_HASH,
                block_type: BlockType::Heartbeat,
            },
            tasks: Vec::new(),
        };

        assert_eq!(block.validate(), Err(BlockError::InvalidPreviousHash));
    }

    #[test]
    fn parent_link_validation_accepts_valid_chain_link() {
        let genesis = Block::new_active_with_timestamp(
            1,
            100,
            bytes32(0),
            bytes32(10),
            bytes32(20),
            vec![valid_task()],
        )
        .expect("genesis-like block must construct successfully");

        let child = Block::new_active_with_timestamp(
            2,
            101,
            genesis.header_hash(),
            bytes32(11),
            bytes32(21),
            vec![valid_task()],
        )
        .expect("child block must construct successfully");

        assert_eq!(child.validate_parent_link(&genesis), Ok(()));
    }

    #[test]
    fn parent_link_validation_rejects_wrong_previous_hash() {
        let parent = Block::new_active_with_timestamp(
            1,
            100,
            bytes32(0),
            bytes32(10),
            bytes32(20),
            vec![valid_task()],
        )
        .expect("parent block must construct successfully");

        let child = Block::new_active_with_timestamp(
            2,
            101,
            bytes32(77),
            bytes32(11),
            bytes32(21),
            vec![valid_task()],
        )
        .expect("child block must construct successfully");

        assert_eq!(
            child.validate_parent_link(&parent),
            Err(BlockError::InvalidPreviousHash)
        );
    }

    #[test]
    fn duplicate_task_id_detection_works() {
        let task = valid_task();

        let block = Block::new_active_with_timestamp(
            1,
            100,
            bytes32(10),
            bytes32(20),
            bytes32(30),
            vec![task.clone(), task],
        )
        .expect("block construction should still succeed under current compatibility policy");

        assert!(block.has_duplicate_task_ids());
    }

    #[test]
    fn try_task_root_succeeds_for_valid_active_block() {
        let block = Block::new_active_with_timestamp(
            1,
            100,
            bytes32(10),
            bytes32(20),
            bytes32(30),
            vec![valid_task()],
        )
        .expect("active block must construct successfully");

        let root = block
            .try_task_root()
            .expect("task root must be computable for a valid block");

        assert_ne!(root, ZERO_HASH);
    }
}

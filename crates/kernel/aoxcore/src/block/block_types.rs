pub const MAX_TASKS_PER_BLOCK: usize = 1_024;

/// Maximum payload size allowed for a single task in bytes.
///
/// Security rationale:
/// This limit constrains task-level resource abuse and prevents oversized
/// execution envelopes from entering the canonical block domain.
pub const MAX_TASK_PAYLOAD_BYTES: usize = 64 * 1024;

/// Maximum aggregate payload size allowed in a single active block.
///
/// Security rationale:
/// This bound limits worst-case block processing cost and prevents excessive
/// payload aggregation from degrading validation or execution performance.
pub const MAX_BLOCK_PAYLOAD_BYTES: usize = 4 * 1024 * 1024;

/// Canonical zero state root used by heartbeat blocks.
///
/// Operational note:
/// Heartbeat blocks do not carry execution semantics and therefore must commit
/// to the protocol-defined zero state root.
pub const ZERO_STATE_ROOT: [u8; 32] = [0u8; 32];

/// Maximum quantum header proof size in bytes.
///
/// Security rationale:
/// This bound prevents oversized post-quantum attestations from causing
/// disproportionate memory or gossip amplification at block-header scope.
pub const MAX_QUANTUM_HEADER_PROOF_BYTES: usize = 4 * 1024;

/// Enumerates the lifecycle role of a block within the protocol.
///
/// Audit rationale:
/// Block type is a policy-bearing field. Validation behavior, task admissibility,
/// and state-root expectations depend directly on this classification.
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
///
/// Audit rationale:
/// Capability expresses the trust path under which a task entered the system
/// and may influence downstream policy or execution interpretation.
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
///
/// Audit rationale:
/// Target outpost participates in canonical hashing and assembly-lane routing.
/// Its numeric representation must therefore remain stable.
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
///
/// Security rationale:
/// The block header binds height, time, parent linkage, state commitment,
/// producer identity, and semantic block type into a consensus-relevant object.
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

    /// Post-quantum signature scheme used for header attestation.
    pub quantum_signature_scheme: crate::protocol::quantum::SignatureScheme,

    /// Post-quantum header attestation proof bytes.
    pub quantum_header_proof: Vec<u8>,

    /// Semantic block type.
    pub block_type: BlockType,
}

/// Canonical task object.
///
/// Security rationale:
/// Tasks are consensus-visible execution carriers. Their identifiers, routing
/// targets, and payload bytes are validated before they enter block construction.
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
    ///
    /// Validation contract:
    /// - payload must be non-empty,
    /// - payload must not exceed the canonical task-level limit.
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
    ///
    /// Production note:
    /// The underlying hashing path is currently infallible for validated tasks,
    /// but the result remains wrapped to preserve API symmetry with other
    /// fallible commitment operations.
    pub fn hash(&self) -> Result<[u8; 32], BlockError> {
        Ok(hash::hash_task(self))
    }
}

/// Canonical block object.
///
/// Security rationale:
/// A block is admitted into the canonical domain only through validated
/// constructors or explicit validation of externally assembled data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub tasks: Vec<Task>,
}

#![allow(clippy::module_name_repetitions)]

use std::collections::{BTreeMap, BTreeSet};

/// @notice Canonical transaction identifier type used by the kernel.
/// @dev This alias may be replaced with the project-specific transaction ID type.
pub type TxId = [u8; 32];

/// @notice Canonical lane identifier type used by the kernel.
/// @dev This alias may be replaced with the project-specific lane ID type.
pub type LaneId = u64;

/// @notice Canonical hash type used by the kernel.
/// @dev This alias may be replaced with the project-specific digest type.
pub type Hash = [u8; 32];

/// @notice Zero hash constant used for empty commitment domains.
pub const ZERO_HASH: Hash = [0u8; 32];

/// @notice Stable kernel error taxonomy.
/// @dev The error surface is intentionally compact at the kernel boundary.
///      Runtime-local detail may be preserved in auxiliary diagnostics if required.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// @dev The referenced lane is not registered in the kernel registry.
    UnknownLane { lane_id: LaneId },

    /// @dev The referenced lane exists but is not currently active.
    InactiveLane { lane_id: LaneId },

    /// @dev The caller or transaction context does not possess the required capability.
    CapabilityDenied {
        lane_id: LaneId,
        capability: Capability,
    },

    /// @dev The execution was rejected by an enabled lane policy.
    LanePolicyViolation {
        lane_id: LaneId,
        reason: String,
    },

    /// @dev The runtime returned a deterministic execution revert.
    ExecutionReverted {
        lane_id: LaneId,
        reason: String,
    },
}

/// @notice Canonical capability model enforced before execution.
/// @dev This model should remain minimal and deterministic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    /// @dev Allows general transaction execution on a lane.
    Execute,

    /// @dev Allows cross-lane message emission on a lane.
    EmitCrossLaneMessage,
}

/// @notice Lane trust classification.
/// @dev This model is intentionally simple and may be extended only if the
///      additional states have deterministic downstream meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel {
    /// @dev The lane is considered fully trusted by local policy.
    Trusted,

    /// @dev The lane is usable but subject to additional scrutiny.
    Restricted,
}

/// @notice Optional lane class metadata.
/// @dev This field is descriptive unless bound to explicit policy rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneClass {
    /// @dev Default execution lane.
    Standard,

    /// @dev Settlement-sensitive lane for privileged workflows.
    Settlement,

    /// @dev Bridge or interoperability lane for cross-domain actions.
    Bridge,
}

/// @notice Optional lane policy configuration.
/// @dev This structure is deliberately narrow. Additional fields should be added
///      only if their enforcement semantics are fully implemented and test-covered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanePolicy {
    /// @dev Whether cross-lane message emission is allowed on this lane.
    pub allow_cross_lane_messages: bool,
}

/// @notice Persistent lane registration data used by the kernel.
/// @dev This object is the authoritative configuration source for lane validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneRegistration {
    /// @dev Unique identifier of the lane.
    pub lane_id: LaneId,

    /// @dev Whether the lane is currently active for execution.
    pub active: bool,

    /// @dev Trust posture assigned to the lane.
    pub trust_level: TrustLevel,

    /// @dev Descriptive lane class metadata.
    pub class: LaneClass,

    /// @dev Capability set allowed on this lane.
    pub capabilities: BTreeSet<Capability>,

    /// @dev Optional policy configuration for advanced enforcement.
    pub policy: Option<LanePolicy>,
}

/// @notice Execution environment provided to the kernel.
/// @dev This object captures the validated caller context and execution envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionEnv {
    /// @dev Transaction identifier.
    pub tx_id: TxId,

    /// @dev Target lane identifier.
    pub lane_id: LaneId,

    /// @dev Capabilities declared or assigned to the current execution context.
    pub capabilities: BTreeSet<Capability>,
}

/// @notice Cross-lane message emitted by runtime execution.
/// @dev Ordering must remain deterministic because message commitments depend on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossLaneMessage {
    /// @dev Destination lane identifier.
    pub destination_lane_id: LaneId,

    /// @dev Opaque message payload committed into the receipt-level root.
    pub payload: Vec<u8>,
}

/// @notice Runtime execution output prior to canonical normalization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneOutput {
    /// @dev Whether the runtime execution committed successfully.
    pub success: bool,

    /// @dev Optional revert reason when execution fails deterministically.
    pub revert_reason: Option<String>,

    /// @dev Gas or execution unit consumption attributed to this transaction.
    pub gas_used: u64,

    /// @dev Post-execution state commitment.
    pub state_root: Hash,

    /// @dev Cross-lane messages emitted during execution.
    pub cross_lane_messages: Vec<CrossLaneMessage>,
}

/// @notice Canonical settlement-oriented execution status.
/// @dev This enum intentionally compresses runtime-local outcomes into a stable
///      settlement domain suitable for downstream verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalStatus {
    /// @dev Execution completed successfully and the state transition was committed.
    Succeeded,

    /// @dev Execution entered the runtime but reverted deterministically.
    Reverted,

    /// @dev Execution was rejected before state transition finalization.
    Rejected,
}

/// @notice Security-oriented annotations attached to canonical receipts.
/// @dev These flags enrich observability and policy context without redefining status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityFlag {
    /// @dev No elevated security condition was observed.
    None,

    /// @dev Execution occurred on a restricted-trust lane.
    RestrictedLane,

    /// @dev Execution was evaluated under an enabled policy regime.
    PolicyConstrained,

    /// @dev Execution emitted one or more cross-lane messages.
    CrossLaneEmission,

    /// @dev Execution was rejected due to a security-relevant validation rule.
    SecurityRejected,
}

/// @notice Canonical settlement receipt emitted by the kernel.
/// @dev This structure is the single authoritative normalized output object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalSettlementReceipt {
    /// @dev Unique transaction identifier.
    pub tx_id: TxId,

    /// @dev Lane on which execution was evaluated.
    pub lane_id: LaneId,

    /// @dev Canonical settlement status.
    pub status: CanonicalStatus,

    /// @dev Optional normalized reason string for diagnostics and auditing.
    pub reason: Option<String>,

    /// @dev Gas or execution unit consumption attributed to execution.
    pub gas_used: u64,

    /// @dev Post-execution state commitment.
    pub state_root: Hash,

    /// @dev Number of emitted cross-lane messages.
    pub cross_lane_message_count: u32,

    /// @dev Deterministic commitment root over the emitted cross-lane message set.
    pub cross_lane_message_root: Hash,

    /// @dev Machine-readable security annotations.
    pub security_flags: Vec<SecurityFlag>,
}

/// @notice Internal receipt model preserved by the kernel.
/// @dev The canonical settlement receipt must always be derived through the
///      `canonical()` method below rather than manual field-by-field reconstruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    /// @dev Transaction identifier.
    pub tx_id: TxId,

    /// @dev Lane identifier.
    pub lane_id: LaneId,

    /// @dev Execution success indicator.
    pub success: bool,

    /// @dev Whether the transaction was rejected before runtime execution.
    pub rejected: bool,

    /// @dev Optional normalized reason string.
    pub reason: Option<String>,

    /// @dev Gas used by execution.
    pub gas_used: u64,

    /// @dev Final state commitment.
    pub state_root: Hash,

    /// @dev Number of emitted cross-lane messages.
    pub cross_lane_message_count: u32,

    /// @dev Deterministic root over emitted cross-lane messages.
    pub cross_lane_message_root: Hash,

    /// @dev Security annotations collected during validation and execution.
    pub security_flags: Vec<SecurityFlag>,
}

impl Receipt {
    /// @notice Converts the internal receipt into the canonical settlement format.
    /// @dev This method is the sole authoritative normalization path and must remain
    ///      the only constructor for `CanonicalSettlementReceipt`.
    pub fn canonical(&self) -> CanonicalSettlementReceipt {
        let status = if self.rejected {
            CanonicalStatus::Rejected
        } else if self.success {
            CanonicalStatus::Succeeded
        } else {
            CanonicalStatus::Reverted
        };

        CanonicalSettlementReceipt {
            tx_id: self.tx_id,
            lane_id: self.lane_id,
            status,
            reason: self.reason.clone(),
            gas_used: self.gas_used,
            state_root: self.state_root,
            cross_lane_message_count: self.cross_lane_message_count,
            cross_lane_message_root: self.cross_lane_message_root,
            security_flags: self.security_flags.clone(),
        }
    }
}

/// @notice Authoritative kernel implementation.
/// @dev The kernel owns lane registration and enforces the single execution path.
#[derive(Debug, Default)]
pub struct Kernel {
    /// @dev Registered lanes indexed by lane identifier.
    lanes: BTreeMap<LaneId, LaneRegistration>,
}

impl Kernel {
    /// @notice Creates a new empty kernel instance.
    pub fn new() -> Self {
        Self {
            lanes: BTreeMap::new(),
        }
    }

    /// @notice Registers or replaces a lane definition.
    /// @dev Replacement is deterministic and explicit. Callers are responsible
    ///      for higher-level governance over lane mutation.
    pub fn register_lane(&mut self, registration: LaneRegistration) {
        self.lanes.insert(registration.lane_id, registration);
    }

    /// @notice Resolves an immutable lane registration reference.
    fn lane(&self, lane_id: LaneId) -> Result<&LaneRegistration, KernelError> {
        self.lanes
            .get(&lane_id)
            .ok_or(KernelError::UnknownLane { lane_id })
    }

    /// @notice Executes a transaction and returns the canonical settlement receipt.
    /// @dev Validation, runtime execution, cross-lane commitment derivation, and
    ///      canonical normalization are deliberately centralized in this method.
    pub fn execute_tx<F>(
        &self,
        env: &ExecutionEnv,
        runtime: F,
    ) -> Result<CanonicalSettlementReceipt, KernelError>
    where
        F: FnOnce(&ExecutionEnv, &LaneRegistration) -> Result<LaneOutput, KernelError>,
    {
        let registration = self.lane(env.lane_id)?;

        self.validate_lane_active(registration)?;
        self.validate_capability(env, registration, Capability::Execute)?;
        self.validate_lane_policy(env, registration)?;

        let mut security_flags = self.derive_pre_execution_flags(registration);

        let output = runtime(env, registration)?;

        if !output.cross_lane_messages.is_empty() {
            self.validate_capability(env, registration, Capability::EmitCrossLaneMessage)?;
            security_flags.push(SecurityFlag::CrossLaneEmission);
        }

        let cross_lane_message_count =
            u32::try_from(output.cross_lane_messages.len()).expect("cross-lane message count exceeds u32");

        let cross_lane_message_root = derive_cross_lane_message_root(&output.cross_lane_messages);

        let receipt = Receipt {
            tx_id: env.tx_id,
            lane_id: env.lane_id,
            success: output.success,
            rejected: false,
            reason: output.revert_reason,
            gas_used: output.gas_used,
            state_root: output.state_root,
            cross_lane_message_count,
            cross_lane_message_root,
            security_flags,
        };

        Ok(receipt.canonical())
    }

    /// @notice Evaluates a transaction rejection path and returns a canonical receipt.
    /// @dev This helper preserves a consistent rejected-receipt structure when a caller
    ///      prefers receipt materialization instead of immediate error propagation.
    pub fn reject_receipt(
        &self,
        env: &ExecutionEnv,
        reason: String,
    ) -> CanonicalSettlementReceipt {
        let receipt = Receipt {
            tx_id: env.tx_id,
            lane_id: env.lane_id,
            success: false,
            rejected: true,
            reason: Some(reason),
            gas_used: 0,
            state_root: ZERO_HASH,
            cross_lane_message_count: 0,
            cross_lane_message_root: ZERO_HASH,
            security_flags: vec![SecurityFlag::SecurityRejected],
        };

        receipt.canonical()
    }

    /// @notice Validates that the lane is active.
    fn validate_lane_active(&self, registration: &LaneRegistration) -> Result<(), KernelError> {
        if registration.active {
            Ok(())
        } else {
            Err(KernelError::InactiveLane {
                lane_id: registration.lane_id,
            })
        }
    }

    /// @notice Validates that both the lane and the execution environment authorize a capability.
    fn validate_capability(
        &self,
        env: &ExecutionEnv,
        registration: &LaneRegistration,
        capability: Capability,
    ) -> Result<(), KernelError> {
        if registration.capabilities.contains(&capability) && env.capabilities.contains(&capability) {
            Ok(())
        } else {
            Err(KernelError::CapabilityDenied {
                lane_id: registration.lane_id,
                capability,
            })
        }
    }

    /// @notice Validates optional lane policy constraints.
    /// @dev This function is intentionally complete for the currently declared
    ///      policy surface and should not reference undefined policy concepts.
    fn validate_lane_policy(
        &self,
        env: &ExecutionEnv,
        registration: &LaneRegistration,
    ) -> Result<(), KernelError> {
        if let Some(policy) = &registration.policy {
            if !policy.allow_cross_lane_messages
                && env.capabilities.contains(&Capability::EmitCrossLaneMessage)
            {
                return Err(KernelError::LanePolicyViolation {
                    lane_id: registration.lane_id,
                    reason: "Cross-lane message emission is disabled by lane policy".to_owned(),
                });
            }
        }

        Ok(())
    }

    /// @notice Derives deterministic pre-execution security flags from lane metadata.
    fn derive_pre_execution_flags(&self, registration: &LaneRegistration) -> Vec<SecurityFlag> {
        let mut flags = Vec::new();

        if registration.trust_level == TrustLevel::Restricted {
            flags.push(SecurityFlag::RestrictedLane);
        }

        if registration.policy.is_some() {
            flags.push(SecurityFlag::PolicyConstrained);
        }

        if flags.is_empty() {
            flags.push(SecurityFlag::None);
        }

        flags
    }
}

/// @notice Derives a deterministic commitment root over emitted cross-lane messages.
/// @dev This implementation intentionally avoids external dependencies and provides
///      a stable placeholder combiner. In production, this should be replaced with
///      the project’s canonical hash primitive, provided the ordering and encoding
///      remain stable and fully documented.
pub fn derive_cross_lane_message_root(messages: &[CrossLaneMessage]) -> Hash {
    if messages.is_empty() {
        return ZERO_HASH;
    }

    let mut acc = [0u8; 32];

    for (index, message) in messages.iter().enumerate() {
        let idx = u64::try_from(index).expect("message index exceeds u64").to_le_bytes();

        for (i, b) in idx.iter().enumerate() {
            acc[i % 32] ^= *b;
        }

        for (i, b) in message.destination_lane_id.to_le_bytes().iter().enumerate() {
            acc[(8 + i) % 32] ^= *b;
        }

        for (i, b) in message.payload.iter().enumerate() {
            acc[(16 + i) % 32] ^= *b;
        }

        let payload_len = u64::try_from(message.payload.len())
            .expect("payload length exceeds u64")
            .to_le_bytes();

        for (i, b) in payload_len.iter().enumerate() {
            acc[(24 + i) % 32] ^= *b;
        }
    }

    acc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx_id(n: u8) -> TxId {
        [n; 32]
    }

    fn hash(n: u8) -> Hash {
        [n; 32]
    }

    fn kernel_with_standard_lane() -> Kernel {
        let mut kernel = Kernel::new();

        let mut capabilities = BTreeSet::new();
        capabilities.insert(Capability::Execute);
        capabilities.insert(Capability::EmitCrossLaneMessage);

        kernel.register_lane(LaneRegistration {
            lane_id: 1,
            active: true,
            trust_level: TrustLevel::Trusted,
            class: LaneClass::Standard,
            capabilities,
            policy: Some(LanePolicy {
                allow_cross_lane_messages: true,
            }),
        });

        kernel
    }

    fn standard_env() -> ExecutionEnv {
        let mut capabilities = BTreeSet::new();
        capabilities.insert(Capability::Execute);
        capabilities.insert(Capability::EmitCrossLaneMessage);

        ExecutionEnv {
            tx_id: tx_id(7),
            lane_id: 1,
            capabilities,
        }
    }

    #[test]
    fn canonical_receipt_maps_success_status() {
        let receipt = Receipt {
            tx_id: tx_id(1),
            lane_id: 1,
            success: true,
            rejected: false,
            reason: None,
            gas_used: 21000,
            state_root: hash(9),
            cross_lane_message_count: 0,
            cross_lane_message_root: ZERO_HASH,
            security_flags: vec![SecurityFlag::None],
        };

        let canonical = receipt.canonical();

        assert_eq!(canonical.status, CanonicalStatus::Succeeded);
        assert_eq!(canonical.cross_lane_message_count, 0);
        assert_eq!(canonical.cross_lane_message_root, ZERO_HASH);
    }

    #[test]
    fn canonical_receipt_maps_revert_status() {
        let receipt = Receipt {
            tx_id: tx_id(2),
            lane_id: 1,
            success: false,
            rejected: false,
            reason: Some("runtime revert".to_owned()),
            gas_used: 5000,
            state_root: hash(3),
            cross_lane_message_count: 0,
            cross_lane_message_root: ZERO_HASH,
            security_flags: vec![SecurityFlag::None],
        };

        let canonical = receipt.canonical();

        assert_eq!(canonical.status, CanonicalStatus::Reverted);
        assert_eq!(canonical.reason.as_deref(), Some("runtime revert"));
    }

    #[test]
    fn canonical_receipt_maps_rejected_status() {
        let kernel = kernel_with_standard_lane();
        let env = standard_env();

        let receipt = kernel.reject_receipt(&env, "policy rejected".to_owned());

        assert_eq!(receipt.status, CanonicalStatus::Rejected);
        assert_eq!(receipt.gas_used, 0);
        assert_eq!(receipt.cross_lane_message_count, 0);
        assert_eq!(receipt.cross_lane_message_root, ZERO_HASH);
    }

    #[test]
    fn execute_tx_produces_cross_lane_commitment() {
        let kernel = kernel_with_standard_lane();
        let env = standard_env();

        let receipt = kernel
            .execute_tx(&env, |_env, _registration| {
                Ok(LaneOutput {
                    success: true,
                    revert_reason: None,
                    gas_used: 42000,
                    state_root: hash(4),
                    cross_lane_messages: vec![
                        CrossLaneMessage {
                            destination_lane_id: 2,
                            payload: b"message-a".to_vec(),
                        },
                        CrossLaneMessage {
                            destination_lane_id: 3,
                            payload: b"message-b".to_vec(),
                        },
                    ],
                })
            })
            .expect("execution must succeed");

        assert_eq!(receipt.status, CanonicalStatus::Succeeded);
        assert_eq!(receipt.cross_lane_message_count, 2);
        assert_ne!(receipt.cross_lane_message_root, ZERO_HASH);
        assert!(receipt.security_flags.contains(&SecurityFlag::CrossLaneEmission));
    }

    #[test]
    fn execute_tx_rejects_unknown_lane() {
        let kernel = Kernel::new();

        let mut capabilities = BTreeSet::new();
        capabilities.insert(Capability::Execute);

        let env = ExecutionEnv {
            tx_id: tx_id(5),
            lane_id: 999,
            capabilities,
        };

        let result = kernel.execute_tx(&env, |_env, _registration| {
            Ok(LaneOutput {
                success: true,
                revert_reason: None,
                gas_used: 1,
                state_root: hash(1),
                cross_lane_messages: Vec::new(),
            })
        });

        assert!(matches!(result, Err(KernelError::UnknownLane { lane_id: 999 })));
    }

    #[test]
    fn execute_tx_rejects_inactive_lane() {
        let mut kernel = Kernel::new();

        let mut capabilities = BTreeSet::new();
        capabilities.insert(Capability::Execute);

        kernel.register_lane(LaneRegistration {
            lane_id: 10,
            active: false,
            trust_level: TrustLevel::Trusted,
            class: LaneClass::Standard,
            capabilities: capabilities.clone(),
            policy: None,
        });

        let env = ExecutionEnv {
            tx_id: tx_id(8),
            lane_id: 10,
            capabilities,
        };

        let result = kernel.execute_tx(&env, |_env, _registration| {
            Ok(LaneOutput {
                success: true,
                revert_reason: None,
                gas_used: 1,
                state_root: hash(1),
                cross_lane_messages: Vec::new(),
            })
        });

        assert!(matches!(result, Err(KernelError::InactiveLane { lane_id: 10 })));
    }

    #[test]
    fn execute_tx_rejects_missing_capability() {
        let kernel = kernel_with_standard_lane();

        let env = ExecutionEnv {
            tx_id: tx_id(9),
            lane_id: 1,
            capabilities: BTreeSet::new(),
        };

        let result = kernel.execute_tx(&env, |_env, _registration| {
            Ok(LaneOutput {
                success: true,
                revert_reason: None,
                gas_used: 1,
                state_root: hash(1),
                cross_lane_messages: Vec::new(),
            })
        });

        assert!(matches!(
            result,
            Err(KernelError::CapabilityDenied {
                lane_id: 1,
                capability: Capability::Execute
            })
        ));
    }

    #[test]
    fn execute_tx_enforces_lane_policy() {
        let mut kernel = Kernel::new();

        let mut lane_capabilities = BTreeSet::new();
        lane_capabilities.insert(Capability::Execute);
        lane_capabilities.insert(Capability::EmitCrossLaneMessage);

        kernel.register_lane(LaneRegistration {
            lane_id: 77,
            active: true,
            trust_level: TrustLevel::Restricted,
            class: LaneClass::Bridge,
            capabilities: lane_capabilities.clone(),
            policy: Some(LanePolicy {
                allow_cross_lane_messages: false,
            }),
        });

        let env = ExecutionEnv {
            tx_id: tx_id(10),
            lane_id: 77,
            capabilities: lane_capabilities,
        };

        let result = kernel.execute_tx(&env, |_env, _registration| {
            Ok(LaneOutput {
                success: true,
                revert_reason: None,
                gas_used: 100,
                state_root: hash(2),
                cross_lane_messages: vec![CrossLaneMessage {
                    destination_lane_id: 99,
                    payload: b"x".to_vec(),
                }],
            })
        });

        assert!(matches!(
            result,
            Err(KernelError::LanePolicyViolation { lane_id: 77, .. })
        ));
    }

    #[test]
    fn derive_cross_lane_message_root_returns_zero_for_empty_set() {
        let root = derive_cross_lane_message_root(&[]);
        assert_eq!(root, ZERO_HASH);
    }

    #[test]
    fn derive_cross_lane_message_root_is_deterministic() {
        let messages = vec![
            CrossLaneMessage {
                destination_lane_id: 2,
                payload: b"alpha".to_vec(),
            },
            CrossLaneMessage {
                destination_lane_id: 3,
                payload: b"beta".to_vec(),
            },
        ];

        let root_a = derive_cross_lane_message_root(&messages);
        let root_b = derive_cross_lane_message_root(&messages);

        assert_eq!(root_a, root_b);
    }
}

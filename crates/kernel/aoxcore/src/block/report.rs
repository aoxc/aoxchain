// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

//! core/src/block/report.rs
//!
//! Canonical operator-facing block validation reporting module.
//!
//! This module converts block-domain validation outcomes into stable,
//! serializable, operator-readable reports suitable for CLI surfaces,
//! desktop control planes, observability pipelines, and audit evidence.
//!
//! Design objectives:
//! - Stable machine-readable event codes
//! - Plain-language operator guidance
//! - Deterministic serialization behavior
//! - Clear separation between protocol errors and presentation descriptors
//! - Forward-compatible event reporting structure

use super::{Block, BlockError, BlockType};
use serde::{Deserialize, Serialize};

/// Stable severity categories emitted by block validation workflows.
///
/// Audit rationale:
/// Event severity must remain stable across integrations so that external
/// tooling can classify operational outcomes without parsing human text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationEventType {
    Info,
    Warning,
    Error,
}

/// Stable plain-language descriptor for a block-domain error.
///
/// Audit rationale:
/// The descriptor layer decouples protocol errors from operator-facing
/// explanation. This allows a stable error contract while preserving
/// human-readable operational guidance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorDescriptor {
    pub code: &'static str,
    pub title: &'static str,
    pub plain_message: &'static str,
    pub probable_cause: &'static str,
    pub operator_action: &'static str,
}

/// Single serializable event entry emitted during validation reporting.
///
/// Audit rationale:
/// Events are intentionally normalized into strings so they can be rendered
/// consistently across CLI, desktop, JSON APIs, and external log pipelines.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationEvent {
    pub event_type: ValidationEventType,
    pub code: String,
    pub title: String,
    pub message: String,
    pub action: String,
}

/// Serializable operator-facing validation report.
///
/// Security rationale:
/// This structure is intended for diagnostics and operator guidance. It does
/// not replace canonical protocol validation, but it should remain stable and
/// unambiguous for production monitoring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockValidationReport {
    pub accepted: bool,
    pub block_height: u64,
    pub block_type: BlockType,
    pub task_count: usize,
    pub total_payload_bytes: usize,
    pub primary_error_code: Option<String>,
    pub events: Vec<ValidationEvent>,
}

impl BlockValidationReport {
    /// Serializes the report into a stable pretty-printed JSON document.
    ///
    /// Operational rationale:
    /// This helper is intended for desktop panels, CLI inspection, incident
    /// attachment, and audit evidence generation.
    pub fn to_pretty_json(&self) -> Result<String, BlockError> {
        serde_json::to_string_pretty(self).map_err(|_| BlockError::SerializationFailed)
    }

    /// Serializes the report into a compact JSON document.
    ///
    /// Operational rationale:
    /// This helper is intended for log forwarding, telemetry pipelines, and
    /// network transfer paths where compact representation is preferred.
    pub fn to_json(&self) -> Result<String, BlockError> {
        serde_json::to_string(self).map_err(|_| BlockError::SerializationFailed)
    }

    /// Returns `true` when the report contains at least one error event.
    #[must_use]
    pub fn has_error_events(&self) -> bool {
        self.events
            .iter()
            .any(|event| event.event_type == ValidationEventType::Error)
    }

    /// Returns `true` when the report contains at least one warning event.
    #[must_use]
    pub fn has_warning_events(&self) -> bool {
        self.events
            .iter()
            .any(|event| event.event_type == ValidationEventType::Warning)
    }
}

/// Converts a block-domain error into a stable operator-facing descriptor.
///
/// Audit rationale:
/// This mapping intentionally uses stable codes and formal English operator
/// guidance. Integrators must not depend on title or message wording for
/// protocol-critical behavior; they should rely on the stable error code.
#[must_use]
pub fn describe_block_error(err: BlockError) -> ErrorDescriptor {
    match err {
        BlockError::InvalidSystemTime => ErrorDescriptor {
            code: err.code(),
            title: "Invalid System Time",
            plain_message: "The node could not derive a trustworthy system timestamp for block processing.",
            probable_cause: "The host clock may be drifting, misconfigured, or not synchronized through a reliable time source.",
            operator_action: "Verify host time synchronization, confirm NTP integrity, and re-run the block workflow after remediation.",
        },
        BlockError::ActiveBlockRequiresTasks => ErrorDescriptor {
            code: err.code(),
            title: "Active Block Requires Tasks",
            plain_message: "An active block must contain at least one canonical task.",
            probable_cause: "The block production path was invoked with an empty task set.",
            operator_action: "Inspect mempool selection and block assembly inputs, and prevent empty task sets from reaching active block construction.",
        },
        BlockError::HeartbeatBlockMustNotContainTasks => ErrorDescriptor {
            code: err.code(),
            title: "Heartbeat Block Contains Tasks",
            plain_message: "A heartbeat block is a liveness artifact and must not contain execution tasks.",
            probable_cause: "The block type may have been selected incorrectly during production.",
            operator_action: "Ensure heartbeat emission is isolated from active transaction packaging and validate block-type routing.",
        },
        BlockError::EpochPruneBlockMustNotContainTasks => ErrorDescriptor {
            code: err.code(),
            title: "Epoch-Prune Block Contains Tasks",
            plain_message: "An epoch-prune block is reserved for maintenance operations and must not contain user tasks.",
            probable_cause: "Maintenance and transaction execution flows may have been mixed.",
            operator_action: "Separate pruning workflows from active block packaging and enforce block-type validation before construction.",
        },
        BlockError::HeartbeatBlockMustUseZeroStateRoot => ErrorDescriptor {
            code: err.code(),
            title: "Heartbeat Block Uses Invalid State Root",
            plain_message: "A heartbeat block must commit to the canonical zero state root.",
            probable_cause: "The state-root field was populated with a non-heartbeat value.",
            operator_action: "Force heartbeat production to use ZERO_STATE_ROOT and verify heartbeat-specific construction paths.",
        },
        BlockError::EmptyTaskPayload => ErrorDescriptor {
            code: err.code(),
            title: "Empty Task Payload",
            plain_message: "A canonical task payload must not be empty.",
            probable_cause: "The task may have been serialized incorrectly or constructed without execution data.",
            operator_action: "Enforce non-empty payload checks at task creation boundaries and validate serialization prior to block inclusion.",
        },
        BlockError::TaskPayloadTooLarge { .. } => ErrorDescriptor {
            code: err.code(),
            title: "Task Payload Too Large",
            plain_message: "A single task payload exceeded the permitted canonical size limit.",
            probable_cause: "A large binary blob or oversized execution request was packed into one task.",
            operator_action: "Split the data into smaller units or move large content off-chain and reference it indirectly.",
        },
        BlockError::TooManyTasks { .. } => ErrorDescriptor {
            code: err.code(),
            title: "Too Many Tasks",
            plain_message: "The block exceeded the maximum permitted task count.",
            probable_cause: "Task packaging limits may not have been enforced during block assembly.",
            operator_action: "Apply the maximum task-count gate prior to block construction and review mempool batch selection policy.",
        },
        BlockError::TotalPayloadTooLarge { .. } => ErrorDescriptor {
            code: err.code(),
            title: "Total Block Payload Too Large",
            plain_message: "The aggregate payload size of the block exceeded the permitted canonical limit.",
            probable_cause: "Too many large tasks were aggregated into the same block.",
            operator_action: "Tighten payload-based block packaging policy and enforce aggregate-size checks before finalization.",
        },
        BlockError::LengthOverflow => ErrorDescriptor {
            code: err.code(),
            title: "Length Calculation Overflow",
            plain_message: "A size or length calculation overflow was detected during block validation.",
            probable_cause: "Unexpected input dimensions, malformed data, or adversarial input may have triggered an arithmetic boundary.",
            operator_action: "Inspect input dimensions, review upstream limit enforcement, and investigate the event as a possible anomaly.",
        },
        BlockError::InvalidBlockHeight => ErrorDescriptor {
            code: err.code(),
            title: "Invalid Block Height",
            plain_message: "The block height does not match the expected canonical sequence.",
            probable_cause: "The block may reference an incorrect parent or may be part of a replay or fork inconsistency.",
            operator_action: "Re-validate parent selection, fork-choice output, and chain-link continuity before accepting the block.",
        },
        BlockError::InvalidPreviousHash => ErrorDescriptor {
            code: err.code(),
            title: "Invalid Previous Hash",
            plain_message: "The block previous-hash field is inconsistent with the expected parent block hash.",
            probable_cause: "An incorrect parent may have been selected or the block linkage may have been corrupted.",
            operator_action: "Verify canonical parent linkage and inspect the upstream network or storage source for corruption.",
        },
        BlockError::DuplicateTaskId => ErrorDescriptor {
            code: err.code(),
            title: "Duplicate Task Identifier",
            plain_message: "The block contains the same task identifier more than once.",
            probable_cause: "Task deduplication may have been skipped or bypassed prior to block construction.",
            operator_action: "Enforce task-id uniqueness during admission and packaging, and inspect mempool deduplication controls.",
        },
        BlockError::InvalidTimestamp => ErrorDescriptor {
            code: err.code(),
            title: "Invalid Timestamp",
            plain_message: "The block timestamp is zero or otherwise invalid for canonical processing.",
            probable_cause: "The timestamp source may be broken or the block was constructed with an invalid explicit value.",
            operator_action: "Centralize timestamp derivation and verify time-source integrity before block production.",
        },
        BlockError::InvalidProducer => ErrorDescriptor {
            code: err.code(),
            title: "Invalid Producer Identity",
            plain_message: "The block producer identity is empty, invalid, or inconsistent with the expected validator key material.",
            probable_cause: "Validator role mapping, key management, or signing identity selection may be incorrect.",
            operator_action: "Validate validator key configuration, role binding, and producer identity derivation before retrying.",
        },
        BlockError::InvalidStateRoot => ErrorDescriptor {
            code: err.code(),
            title: "Invalid State Root",
            plain_message: "The block state root does not match the expected canonical state commitment.",
            probable_cause: "State transition computation may be incorrect or state commitment derivation may be inconsistent.",
            operator_action: "Re-run state transition and state-root derivation, and compare the resulting commitment against the block header.",
        },
        BlockError::InvalidTaskRoot => ErrorDescriptor {
            code: err.code(),
            title: "Invalid Task Root",
            plain_message: "The block task root does not match the expected canonical task commitment.",
            probable_cause: "Task ordering, task hashing, or root aggregation may have diverged from canonical rules.",
            operator_action: "Recompute task hashing and ordered root derivation, and verify canonical task ordering inputs.",
        },
        BlockError::HashingFailed => ErrorDescriptor {
            code: err.code(),
            title: "Hashing Failed",
            plain_message: "A cryptographic hashing operation did not complete successfully.",
            probable_cause: "The hashing pipeline may have received malformed or non-canonical input.",
            operator_action: "Inspect canonical encoding, input preparation, and hash-preimage boundaries before retrying.",
        },
        BlockError::SerializationFailed => ErrorDescriptor {
            code: err.code(),
            title: "Serialization Failed",
            plain_message: "The report or related output could not be serialized successfully.",
            probable_cause: "The serialization schema or downstream formatting expectations may be inconsistent.",
            operator_action: "Inspect report serialization boundaries and confirm schema compatibility across consumers.",
        },
    }
}

/// Produces a stable operator-facing block validation report.
///
/// Reporting policy:
/// - a start event is always emitted,
/// - successful validation emits an acceptance event,
/// - failed validation emits one primary error event,
/// - report acceptance is derived exclusively from the absence of a primary error.
#[must_use]
pub fn build_block_validation_report(block: &Block) -> BlockValidationReport {
    let mut events = vec![ValidationEvent {
        event_type: ValidationEventType::Info,
        code: "BLOCK_VALIDATION_STARTED".to_string(),
        title: "Block Validation Started".to_string(),
        message: "The block is being validated against canonical protocol rules.".to_string(),
        action: "Wait for validation to complete before proceeding with downstream processing."
            .to_string(),
    }];

    let result = block.validate();
    let mut primary_error_code = None;

    match result {
        Ok(()) => {
            events.push(ValidationEvent {
                event_type: ValidationEventType::Info,
                code: "BLOCK_VALIDATION_ACCEPTED".to_string(),
                title: "Block Accepted".to_string(),
                message: "The block passed canonical validation successfully.".to_string(),
                action: "The block may proceed to the next pipeline stage.".to_string(),
            });

            if block.task_count() == 0 {
                events.push(ValidationEvent {
                    event_type: ValidationEventType::Warning,
                    code: "BLOCK_CONTAINS_NO_TASKS".to_string(),
                    title: "Block Contains No Tasks".to_string(),
                    message:
                        "The validated block contains no tasks. This may be expected for non-active block types."
                            .to_string(),
                    action:
                        "Confirm that the block type is intentional and consistent with the surrounding workflow."
                            .to_string(),
                });
            }
        }
        Err(err) => {
            let desc = describe_block_error(err);
            primary_error_code = Some(desc.code.to_string());

            events.push(ValidationEvent {
                event_type: ValidationEventType::Error,
                code: desc.code.to_string(),
                title: desc.title.to_string(),
                message: format!(
                    "{} Probable cause: {}",
                    desc.plain_message, desc.probable_cause
                ),
                action: desc.operator_action.to_string(),
            });
        }
    }

    BlockValidationReport {
        accepted: primary_error_code.is_none(),
        block_height: block.header.height,
        block_type: block.header.block_type,
        task_count: block.task_count(),
        total_payload_bytes: block.total_payload_bytes(),
        primary_error_code,
        events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::{Capability, TargetOutpost, Task, ZERO_HASH};

    fn bytes32(v: u8) -> [u8; 32] {
        [v; 32]
    }

    #[test]
    fn error_descriptor_is_operator_readable_for_invalid_block_error() {
        let err = BlockError::ActiveBlockRequiresTasks;
        let desc = describe_block_error(err);

        assert_eq!(desc.code, "BLOCK_ACTIVE_REQUIRES_TASKS");
        assert_eq!(desc.title, "Active Block Requires Tasks");
        assert!(!desc.plain_message.is_empty());
        assert!(!desc.operator_action.is_empty());
    }

    #[test]
    fn validation_report_serializes_for_operator_panels() {
        let task = Task::new(
            bytes32(9),
            Capability::UserSigned,
            TargetOutpost::AovmNative,
            vec![1, 2, 3],
        )
        .expect("task should construct successfully");

        let block =
            Block::new_active_with_timestamp(2, 100, ZERO_HASH, bytes32(8), bytes32(7), vec![task])
                .expect("block should construct successfully");

        let report = block.validate_with_report();
        assert!(report.accepted);
        assert!(!report.has_error_events());

        let json = report
            .to_pretty_json()
            .expect("pretty JSON serialization must succeed");
        assert!(json.contains("BLOCK_VALIDATION_ACCEPTED"));
        assert!(json.contains("\"accepted\": true"));
    }

    #[test]
    fn compact_json_serialization_succeeds() {
        let report = BlockValidationReport {
            accepted: false,
            block_height: 7,
            block_type: BlockType::Active,
            task_count: 0,
            total_payload_bytes: 0,
            primary_error_code: Some("BLOCK_ACTIVE_REQUIRES_TASKS".to_string()),
            events: vec![ValidationEvent {
                event_type: ValidationEventType::Error,
                code: "BLOCK_ACTIVE_REQUIRES_TASKS".to_string(),
                title: "Active Block Requires Tasks".to_string(),
                message: "An active block must contain at least one canonical task.".to_string(),
                action: "Inspect block assembly inputs.".to_string(),
            }],
        };

        let json = report
            .to_json()
            .expect("compact JSON serialization must succeed");

        assert!(json.contains("\"accepted\":false"));
        assert!(json.contains("BLOCK_ACTIVE_REQUIRES_TASKS"));
    }

    #[test]
    fn report_detects_warning_events() {
        let report = BlockValidationReport {
            accepted: true,
            block_height: 1,
            block_type: BlockType::Heartbeat,
            task_count: 0,
            total_payload_bytes: 0,
            primary_error_code: None,
            events: vec![ValidationEvent {
                event_type: ValidationEventType::Warning,
                code: "BLOCK_CONTAINS_NO_TASKS".to_string(),
                title: "Block Contains No Tasks".to_string(),
                message: "The block contains no tasks.".to_string(),
                action: "Confirm the block type.".to_string(),
            }],
        };

        assert!(report.has_warning_events());
        assert!(!report.has_error_events());
    }
}

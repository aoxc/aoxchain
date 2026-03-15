//! core/src/block/error.rs
//!
//! AOVM Block Domain Error Definitions.
//!
//! This module defines the canonical error surface for block, task, validation,
//! chain-link, and hashing-related operations inside the AOVM block domain.
//!
//! Design objectives:
//! - Preserve a stable and auditable symbolic error surface.
//! - Support production-grade telemetry, logging, and incident triage.
//! - Distinguish protocol invariant violations from environmental/runtime issues.
//! - Remain lightweight and dependency-minimal for core domain usage.

use core::fmt;

/// Stable severity classification for block-domain failures.
///
/// This classification is intended for:
/// - operational logging,
/// - metrics tagging,
/// - alert routing,
/// - incident triage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockErrorSeverity {
    /// Informational condition. Typically not used for hard validation failures.
    Info,

    /// Recoverable anomaly or suspicious but non-fatal condition.
    Warning,

    /// Hard validation failure or unrecoverable domain error.
    Error,

    /// Critical failure indicating severe runtime or protocol risk.
    Critical,
}

impl BlockErrorSeverity {
    /// Returns a stable uppercase severity label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warning => "WARN",
            Self::Error => "ERROR",
            Self::Critical => "CRITICAL",
        }
    }
}

/// Stable category classification for block-domain failures.
///
/// This category is suitable for telemetry dimensions and forensic grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockErrorCategory {
    /// Failure related to block header semantics or chain linkage.
    Chain,

    /// Failure related to task structure, payload, or task identity.
    Task,

    /// Failure related to block structure or block composition.
    Block,

    /// Failure related to hashing, encoding, or commitment generation.
    Hashing,

    /// Failure related to local runtime or environment conditions.
    Runtime,
}

impl BlockErrorCategory {
    /// Returns a stable uppercase category label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chain => "CHAIN",
            Self::Task => "TASK",
            Self::Block => "BLOCK",
            Self::Hashing => "HASHING",
            Self::Runtime => "RUNTIME",
        }
    }
}

/// Stable operational classification indicating where the failure originated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockErrorClass {
    /// Failure originated from a protocol invariant violation.
    InvariantViolation,

    /// Failure originated from local runtime or environmental conditions.
    RuntimeFailure,

    /// Failure originated during hashing or canonical encoding workflows.
    CryptographicFailure,

    /// Failure originated from input validation or domain consistency checks.
    ValidationFailure,
}

impl BlockErrorClass {
    /// Returns a stable uppercase class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvariantViolation => "INVARIANT_VIOLATION",
            Self::RuntimeFailure => "RUNTIME_FAILURE",
            Self::CryptographicFailure => "CRYPTOGRAPHIC_FAILURE",
            Self::ValidationFailure => "VALIDATION_FAILURE",
        }
    }
}

/// Canonical error type for AOVM block-domain operations.
///
/// The error surface is intentionally explicit and stable. Variants should not
/// be removed casually because symbolic codes may be consumed by:
/// - node logs,
/// - dashboards,
/// - alerting rules,
/// - audit tooling,
/// - forensic pipelines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum BlockError {
    /// The local system clock is invalid for block timestamp generation.
    InvalidSystemTime,

    /// An active block was created or validated without any tasks.
    ActiveBlockRequiresTasks,

    /// A heartbeat block was found to contain one or more tasks.
    HeartbeatBlockMustNotContainTasks,

    /// An epoch-prune block was found to contain one or more tasks.
    EpochPruneBlockMustNotContainTasks,

    /// A heartbeat block used a non-zero state root.
    HeartbeatBlockMustUseZeroStateRoot,

    /// A task payload was empty.
    EmptyTaskPayload,

    /// A task payload exceeded the configured maximum size.
    TaskPayloadTooLarge {
        /// Observed payload size in bytes.
        size: usize,
        /// Maximum permitted payload size in bytes.
        max: usize,
    },

    /// A block exceeded the configured maximum number of tasks.
    TooManyTasks {
        /// Observed task count.
        count: usize,
        /// Maximum permitted task count.
        max: usize,
    },

    /// The aggregate task payload exceeded the configured maximum size.
    TotalPayloadTooLarge {
        /// Observed aggregate payload size in bytes.
        size: usize,
        /// Maximum permitted aggregate payload size in bytes.
        max: usize,
    },

    /// A fixed-width length boundary was exceeded during canonical encoding or hashing.
    LengthOverflow,

    /// Block height continuity validation failed.
    InvalidBlockHeight,

    /// Previous-hash continuity validation failed.
    InvalidPreviousHash,

    /// Duplicate task identifier detected.
    DuplicateTaskId,

    /// Timestamp validation failed.
    InvalidTimestamp,

    /// Producer identity validation failed.
    InvalidProducer,

    /// State-root validation failed.
    InvalidStateRoot,

    /// Task-root commitment validation failed.
    InvalidTaskRoot,

    /// Hashing operation failed under protocol constraints.
    HashingFailed,

    /// Canonical serialization failed under protocol constraints.
    SerializationFailed,
}

impl BlockError {
    /// Returns a stable symbolic error code suitable for logs, telemetry, and dashboards.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidSystemTime => "BLOCK_INVALID_SYSTEM_TIME",
            Self::ActiveBlockRequiresTasks => "BLOCK_ACTIVE_REQUIRES_TASKS",
            Self::HeartbeatBlockMustNotContainTasks => "BLOCK_HEARTBEAT_MUST_NOT_CONTAIN_TASKS",
            Self::EpochPruneBlockMustNotContainTasks => "BLOCK_EPOCH_PRUNE_MUST_NOT_CONTAIN_TASKS",
            Self::HeartbeatBlockMustUseZeroStateRoot => "BLOCK_HEARTBEAT_MUST_USE_ZERO_STATE_ROOT",
            Self::EmptyTaskPayload => "TASK_EMPTY_PAYLOAD",
            Self::TaskPayloadTooLarge { .. } => "TASK_PAYLOAD_TOO_LARGE",
            Self::TooManyTasks { .. } => "BLOCK_TOO_MANY_TASKS",
            Self::TotalPayloadTooLarge { .. } => "BLOCK_TOTAL_PAYLOAD_TOO_LARGE",
            Self::LengthOverflow => "BLOCK_LENGTH_OVERFLOW",
            Self::InvalidBlockHeight => "BLOCK_INVALID_HEIGHT",
            Self::InvalidPreviousHash => "BLOCK_INVALID_PREVIOUS_HASH",
            Self::DuplicateTaskId => "TASK_DUPLICATE_ID",
            Self::InvalidTimestamp => "BLOCK_INVALID_TIMESTAMP",
            Self::InvalidProducer => "BLOCK_INVALID_PRODUCER",
            Self::InvalidStateRoot => "BLOCK_INVALID_STATE_ROOT",
            Self::InvalidTaskRoot => "BLOCK_INVALID_TASK_ROOT",
            Self::HashingFailed => "BLOCK_HASHING_FAILED",
            Self::SerializationFailed => "BLOCK_SERIALIZATION_FAILED",
        }
    }

    /// Returns the stable category of the error.
    #[must_use]
    pub const fn category(self) -> BlockErrorCategory {
        match self {
            Self::InvalidSystemTime => BlockErrorCategory::Runtime,

            Self::ActiveBlockRequiresTasks
            | Self::HeartbeatBlockMustNotContainTasks
            | Self::EpochPruneBlockMustNotContainTasks
            | Self::HeartbeatBlockMustUseZeroStateRoot
            | Self::TooManyTasks { .. }
            | Self::TotalPayloadTooLarge { .. }
            | Self::InvalidProducer
            | Self::InvalidStateRoot
            | Self::InvalidTaskRoot => BlockErrorCategory::Block,

            Self::EmptyTaskPayload | Self::TaskPayloadTooLarge { .. } | Self::DuplicateTaskId => {
                BlockErrorCategory::Task
            }

            Self::InvalidBlockHeight | Self::InvalidPreviousHash | Self::InvalidTimestamp => {
                BlockErrorCategory::Chain
            }

            Self::LengthOverflow | Self::HashingFailed | Self::SerializationFailed => {
                BlockErrorCategory::Hashing
            }
        }
    }

    /// Returns the operational error class.
    #[must_use]
    pub const fn class(self) -> BlockErrorClass {
        match self {
            Self::InvalidSystemTime => BlockErrorClass::RuntimeFailure,

            Self::HashingFailed | Self::LengthOverflow | Self::SerializationFailed => {
                BlockErrorClass::CryptographicFailure
            }

            Self::ActiveBlockRequiresTasks
            | Self::HeartbeatBlockMustNotContainTasks
            | Self::EpochPruneBlockMustNotContainTasks
            | Self::HeartbeatBlockMustUseZeroStateRoot
            | Self::DuplicateTaskId
            | Self::InvalidBlockHeight
            | Self::InvalidPreviousHash
            | Self::InvalidTimestamp
            | Self::InvalidProducer
            | Self::InvalidStateRoot
            | Self::InvalidTaskRoot => BlockErrorClass::InvariantViolation,

            Self::EmptyTaskPayload
            | Self::TaskPayloadTooLarge { .. }
            | Self::TooManyTasks { .. }
            | Self::TotalPayloadTooLarge { .. } => BlockErrorClass::ValidationFailure,
        }
    }

    /// Returns the recommended operational severity.
    #[must_use]
    pub const fn severity(self) -> BlockErrorSeverity {
        match self {
            Self::InvalidSystemTime | Self::HashingFailed | Self::SerializationFailed => {
                BlockErrorSeverity::Critical
            }

            Self::InvalidBlockHeight
            | Self::InvalidPreviousHash
            | Self::InvalidTimestamp
            | Self::InvalidProducer
            | Self::InvalidStateRoot
            | Self::InvalidTaskRoot
            | Self::DuplicateTaskId
            | Self::HeartbeatBlockMustUseZeroStateRoot
            | Self::ActiveBlockRequiresTasks
            | Self::HeartbeatBlockMustNotContainTasks
            | Self::EpochPruneBlockMustNotContainTasks
            | Self::LengthOverflow => BlockErrorSeverity::Error,

            Self::EmptyTaskPayload
            | Self::TaskPayloadTooLarge { .. }
            | Self::TooManyTasks { .. }
            | Self::TotalPayloadTooLarge { .. } => BlockErrorSeverity::Warning,
        }
    }

    /// Returns `true` if the error represents a protocol invariant violation.
    #[must_use]
    pub const fn is_invariant_violation(self) -> bool {
        matches!(self.class(), BlockErrorClass::InvariantViolation)
    }

    /// Returns `true` if the error is likely to be environmental or runtime-derived.
    #[must_use]
    pub const fn is_runtime_error(self) -> bool {
        matches!(self.class(), BlockErrorClass::RuntimeFailure)
    }

    /// Returns `true` if the failure should be considered security-relevant.
    ///
    /// Security-relevant failures are suitable for:
    /// - elevated telemetry,
    /// - security dashboards,
    /// - incident review,
    /// - abnormal chain-state monitoring.
    #[must_use]
    pub const fn is_security_relevant(self) -> bool {
        matches!(
            self,
            Self::InvalidPreviousHash
                | Self::InvalidBlockHeight
                | Self::InvalidTimestamp
                | Self::InvalidProducer
                | Self::InvalidStateRoot
                | Self::InvalidTaskRoot
                | Self::DuplicateTaskId
                | Self::HashingFailed
                | Self::SerializationFailed
                | Self::LengthOverflow
        )
    }

    /// Returns `true` if the error is safe to classify as deterministic under the same input.
    #[must_use]
    pub const fn is_deterministic(self) -> bool {
        !matches!(self, Self::InvalidSystemTime)
    }

    /// Returns a stable telemetry tuple suitable for metrics labels.
    #[must_use]
    pub const fn telemetry_labels(
        self,
    ) -> (&'static str, &'static str, &'static str, &'static str) {
        (
            self.code(),
            self.category().as_str(),
            self.class().as_str(),
            self.severity().as_str(),
        )
    }

    /// Returns a stable chain-domain namespace string for routing and observability.
    #[must_use]
    pub const fn domain(self) -> &'static str {
        "AOVM_BLOCK_DOMAIN"
    }

    /// Builds a concise, stable, single-line log prefix for production use.
    ///
    /// Example output:
    /// `domain=AOVM_BLOCK_DOMAIN code=BLOCK_INVALID_TIMESTAMP category=CHAIN class=INVARIANT_VIOLATION severity=ERROR`
    #[must_use]
    pub fn log_prefix(self) -> String {
        let (code, category, class, severity) = self.telemetry_labels();

        format!(
            "domain={} code={} category={} class={} severity={}",
            self.domain(),
            code,
            category,
            class,
            severity
        )
    }

    /// Builds a structured human-readable incident string.
    ///
    /// This helper is intentionally allocation-based because it is intended for
    /// logging, diagnostics, and telemetry surfaces rather than hot inner loops.
    #[must_use]
    pub fn incident_message(self) -> String {
        format!("{} message=\"{}\"", self.log_prefix(), self)
    }
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSystemTime => write!(
                f,
                "system time is earlier than UNIX_EPOCH; canonical block timestamp generation is invalid"
            ),
            Self::ActiveBlockRequiresTasks => write!(
                f,
                "active block validation failed: an active block must contain at least one task"
            ),
            Self::HeartbeatBlockMustNotContainTasks => write!(
                f,
                "heartbeat block validation failed: heartbeat blocks must not contain tasks"
            ),
            Self::EpochPruneBlockMustNotContainTasks => write!(
                f,
                "epoch-prune block validation failed: epoch-prune blocks must not contain tasks"
            ),
            Self::HeartbeatBlockMustUseZeroStateRoot => write!(
                f,
                "heartbeat block validation failed: heartbeat blocks must use the protocol-defined zero state root"
            ),
            Self::EmptyTaskPayload => {
                write!(f, "task validation failed: task payload must not be empty")
            }
            Self::TaskPayloadTooLarge { size, max } => write!(
                f,
                "task validation failed: payload size {} bytes exceeds maximum allowed size {} bytes",
                size, max
            ),
            Self::TooManyTasks { count, max } => write!(
                f,
                "block validation failed: task count {} exceeds maximum allowed count {}",
                count, max
            ),
            Self::TotalPayloadTooLarge { size, max } => write!(
                f,
                "block validation failed: aggregate payload size {} bytes exceeds maximum allowed size {} bytes",
                size, max
            ),
            Self::LengthOverflow => write!(
                f,
                "canonical encoding or hashing failed: input length exceeds the supported fixed-width boundary"
            ),
            Self::InvalidBlockHeight => write!(
                f,
                "chain-link validation failed: block height relationship is invalid"
            ),
            Self::InvalidPreviousHash => write!(
                f,
                "chain-link validation failed: previous block hash relationship is invalid"
            ),
            Self::DuplicateTaskId => write!(
                f,
                "task validation failed: duplicate task identifier detected"
            ),
            Self::InvalidTimestamp => write!(
                f,
                "block validation failed: timestamp is outside protocol-accepted bounds"
            ),
            Self::InvalidProducer => write!(
                f,
                "block validation failed: producer identity is invalid or unauthorized"
            ),
            Self::InvalidStateRoot => write!(
                f,
                "block validation failed: state root is invalid for the requested operation"
            ),
            Self::InvalidTaskRoot => write!(
                f,
                "block validation failed: task-root commitment is invalid for the requested operation"
            ),
            Self::HashingFailed => write!(
                f,
                "hashing operation failed under current protocol constraints"
            ),
            Self::SerializationFailed => write!(
                f,
                "canonical serialization failed under current protocol constraints"
            ),
        }
    }
}

impl std::error::Error for BlockError {}

#[cfg(test)]
mod tests {
    use super::{BlockError, BlockErrorCategory, BlockErrorClass, BlockErrorSeverity};

    #[test]
    fn symbolic_error_code_is_stable() {
        let err = BlockError::TaskPayloadTooLarge {
            size: 2048,
            max: 1024,
        };

        assert_eq!(err.code(), "TASK_PAYLOAD_TOO_LARGE");
    }

    #[test]
    fn runtime_classification_is_correct() {
        assert!(BlockError::InvalidSystemTime.is_runtime_error());
        assert!(!BlockError::InvalidPreviousHash.is_runtime_error());
    }

    #[test]
    fn severity_mapping_is_correct() {
        assert_eq!(
            BlockError::InvalidSystemTime.severity(),
            BlockErrorSeverity::Critical
        );

        assert_eq!(
            BlockError::DuplicateTaskId.severity(),
            BlockErrorSeverity::Error
        );

        assert_eq!(
            BlockError::TaskPayloadTooLarge { size: 2, max: 1 }.severity(),
            BlockErrorSeverity::Warning
        );
    }

    #[test]
    fn category_mapping_is_correct() {
        assert_eq!(
            BlockError::InvalidPreviousHash.category(),
            BlockErrorCategory::Chain
        );
        assert_eq!(
            BlockError::EmptyTaskPayload.category(),
            BlockErrorCategory::Task
        );
        assert_eq!(
            BlockError::HashingFailed.category(),
            BlockErrorCategory::Hashing
        );
    }

    #[test]
    fn class_mapping_is_correct() {
        assert_eq!(
            BlockError::InvalidTimestamp.class(),
            BlockErrorClass::InvariantViolation
        );
        assert_eq!(
            BlockError::HashingFailed.class(),
            BlockErrorClass::CryptographicFailure
        );
        assert_eq!(
            BlockError::TaskPayloadTooLarge { size: 3, max: 2 }.class(),
            BlockErrorClass::ValidationFailure
        );
    }

    #[test]
    fn display_output_is_descriptive() {
        let err = BlockError::ActiveBlockRequiresTasks;

        assert_eq!(
            err.to_string(),
            "active block validation failed: an active block must contain at least one task"
        );
    }

    #[test]
    fn log_prefix_contains_stable_operational_fields() {
        let err = BlockError::InvalidProducer;
        let line = err.log_prefix();

        assert!(line.contains("domain=AOVM_BLOCK_DOMAIN"));
        assert!(line.contains("code=BLOCK_INVALID_PRODUCER"));
        assert!(line.contains("category=BLOCK"));
        assert!(line.contains("class=INVARIANT_VIOLATION"));
        assert!(line.contains("severity=ERROR"));
    }

    #[test]
    fn incident_message_contains_error_text() {
        let err = BlockError::InvalidTaskRoot;
        let message = err.incident_message();

        assert!(message.contains("BLOCK_INVALID_TASK_ROOT"));
        assert!(message.contains("task-root commitment is invalid"));
    }

    #[test]
    fn security_relevance_flags_are_correct() {
        assert!(BlockError::InvalidPreviousHash.is_security_relevant());
        assert!(BlockError::SerializationFailed.is_security_relevant());
        assert!(BlockError::LengthOverflow.is_security_relevant());
        assert!(!BlockError::EmptyTaskPayload.is_security_relevant());
    }

    #[test]
    fn deterministic_classification_is_correct() {
        assert!(!BlockError::InvalidSystemTime.is_deterministic());
        assert!(BlockError::InvalidStateRoot.is_deterministic());
    }
}

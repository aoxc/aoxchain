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
fn descriptor_is_stable_and_structured() {
    let desc = BlockError::InvalidPreviousHash.descriptor();

    assert_eq!(desc.domain, "AOVM_BLOCK_DOMAIN");
    assert_eq!(desc.code, "BLOCK_INVALID_PREVIOUS_HASH");
    assert_eq!(desc.category, "CHAIN");
    assert_eq!(desc.class, "INVARIANT_VIOLATION");
    assert_eq!(desc.severity, "ERROR");
    assert!(!desc.message.is_empty());
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

// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::keyforge::cli::{QuorumCommand, QuorumSubcommand};
use serde::Serialize;

/// Canonical operator-facing quorum evaluation response.
///
/// This payload is intentionally deterministic and machine-readable so that it
/// can be consumed safely by shells, CI pipelines, dashboards, and audit tools.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct QuorumEvaluationOutput {
    total: u16,
    approvals: u16,
    threshold_bps: u16,
    required_approvals: u16,
    passed: bool,
}

/// Dispatches AOXC quorum subcommands.
///
/// Design objectives:
/// - keep command routing explicit,
/// - preserve a narrow command surface,
/// - separate business logic from stdout emission for improved testability.
pub fn handle(command: QuorumCommand) -> Result<(), String> {
    match command.command {
        QuorumSubcommand::Evaluate {
            total,
            approvals,
            threshold_bps,
        } => evaluate(total, approvals, threshold_bps),
    }
}

/// Evaluates whether the supplied approval count satisfies the configured quorum.
///
/// Validation policy:
/// - `total` must be non-zero,
/// - `approvals` must not exceed `total`,
/// - `threshold_bps` must be in the canonical inclusive range `1..=10_000`.
fn evaluate(total: u16, approvals: u16, threshold_bps: u16) -> Result<(), String> {
    let output = build_quorum_evaluation_output(total, approvals, threshold_bps)?;
    let body = serialize_pretty_json(&output)?;
    println!("{}", body);
    Ok(())
}

/// Builds the canonical quorum evaluation response without performing I/O.
fn build_quorum_evaluation_output(
    total: u16,
    approvals: u16,
    threshold_bps: u16,
) -> Result<QuorumEvaluationOutput, String> {
    validate_quorum_inputs(total, approvals, threshold_bps)?;

    let required = required_approvals(total, threshold_bps);
    let passed = approvals >= required;

    Ok(QuorumEvaluationOutput {
        total,
        approvals,
        threshold_bps,
        required_approvals: required,
        passed,
    })
}

/// Validates operator-supplied quorum parameters.
fn validate_quorum_inputs(total: u16, approvals: u16, threshold_bps: u16) -> Result<(), String> {
    if total == 0 {
        return Err("QUORUM_TOTAL_ZERO".to_string());
    }

    if approvals > total {
        return Err("QUORUM_APPROVALS_EXCEED_TOTAL".to_string());
    }

    if threshold_bps == 0 || threshold_bps > 10_000 {
        return Err("QUORUM_THRESHOLD_INVALID".to_string());
    }

    Ok(())
}

/// Computes the minimum number of approvals required to satisfy the threshold.
///
/// Rounding policy:
/// - The result is rounded upward using canonical ceiling division.
/// - This ensures the returned requirement is never weaker than the requested
///   threshold percentage.
///
/// Safety properties:
/// - The multiplication is promoted to `u32`.
/// - The maximum possible intermediate value remains well below `u32::MAX`.
fn required_approvals(total: u16, threshold_bps: u16) -> u16 {
    let total_u32 = u32::from(total);
    let threshold_bps_u32 = u32::from(threshold_bps);

    let required = total_u32
        .saturating_mul(threshold_bps_u32)
        .saturating_add(9_999)
        / 10_000;

    debug_assert!(required <= u32::from(u16::MAX));

    required as u16
}

/// Serializes an operator-facing value into canonical pretty JSON.
fn serialize_pretty_json<T>(value: &T) -> Result<String, String>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value).map_err(|error| format!("JSON_SERIALIZE_ERROR: {}", error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn required_approvals_rounds_up() {
        assert_eq!(required_approvals(10, 6667), 7);
        assert_eq!(required_approvals(3, 6667), 3);
        assert_eq!(required_approvals(9, 5001), 5);
    }

    #[test]
    fn required_approvals_handles_full_threshold() {
        assert_eq!(required_approvals(9, 10_000), 9);
        assert_eq!(required_approvals(1, 10_000), 1);
    }

    #[test]
    fn required_approvals_handles_minimum_threshold() {
        assert_eq!(required_approvals(1, 1), 1);
        assert_eq!(required_approvals(10, 1), 1);
        assert_eq!(required_approvals(10_000, 1), 1);
    }

    #[test]
    fn validate_quorum_inputs_rejects_zero_total() {
        let result = validate_quorum_inputs(0, 0, 6667);
        assert_eq!(result, Err("QUORUM_TOTAL_ZERO".to_string()));
    }

    #[test]
    fn validate_quorum_inputs_rejects_approvals_exceeding_total() {
        let result = validate_quorum_inputs(5, 6, 6667);
        assert_eq!(result, Err("QUORUM_APPROVALS_EXCEED_TOTAL".to_string()));
    }

    #[test]
    fn validate_quorum_inputs_rejects_invalid_threshold() {
        assert_eq!(
            validate_quorum_inputs(5, 3, 0),
            Err("QUORUM_THRESHOLD_INVALID".to_string())
        );
        assert_eq!(
            validate_quorum_inputs(5, 3, 10_001),
            Err("QUORUM_THRESHOLD_INVALID".to_string())
        );
    }

    #[test]
    fn build_quorum_evaluation_output_marks_passed_when_threshold_is_met() {
        let output = build_quorum_evaluation_output(10, 7, 6667).expect("evaluation must succeed");

        assert_eq!(
            output,
            QuorumEvaluationOutput {
                total: 10,
                approvals: 7,
                threshold_bps: 6667,
                required_approvals: 7,
                passed: true,
            }
        );
    }

    #[test]
    fn build_quorum_evaluation_output_marks_failed_when_threshold_is_not_met() {
        let output = build_quorum_evaluation_output(10, 6, 6667).expect("evaluation must succeed");

        assert_eq!(output.required_approvals, 7);
        assert!(!output.passed);
    }

    #[test]
    fn build_quorum_evaluation_output_rejects_invalid_inputs() {
        let result = build_quorum_evaluation_output(0, 0, 6667);
        assert_eq!(result, Err("QUORUM_TOTAL_ZERO".to_string()));
    }

    #[test]
    fn serialize_pretty_json_returns_valid_json_document() {
        let output = QuorumEvaluationOutput {
            total: 10,
            approvals: 7,
            threshold_bps: 6667,
            required_approvals: 7,
            passed: true,
        };

        let body = serialize_pretty_json(&output).expect("serialization must succeed");
        let parsed: Value = serde_json::from_str(&body).expect("output must be valid JSON");

        assert_eq!(parsed["total"], 10);
        assert_eq!(parsed["approvals"], 7);
        assert_eq!(parsed["threshold_bps"], 6667);
        assert_eq!(parsed["required_approvals"], 7);
        assert_eq!(parsed["passed"], true);
    }

    #[test]
    fn evaluate_executes_successfully_for_valid_input() {
        let result = evaluate(10, 7, 6667);
        assert!(result.is_ok());
    }

    #[test]
    fn handle_dispatches_evaluate_successfully() {
        let command = QuorumCommand {
            command: QuorumSubcommand::Evaluate {
                total: 10,
                approvals: 7,
                threshold_bps: 6667,
            },
        };

        let result = handle(command);
        assert!(result.is_ok());
    }
}

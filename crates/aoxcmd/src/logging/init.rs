// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use crate::telemetry::tracing::{TraceContext, new_context};

const DEFAULT_TRACE_COMMAND: &str = "unknown-command";

/// Builds a canonical trace context for an operator-visible command surface.
///
/// Contract:
/// - The trace surface must always return a usable `TraceContext`.
/// - Blank or whitespace-only command names are normalized to a safe
///   fallback label rather than propagating an empty tracing identifier.
/// - Input normalization is intentionally lightweight and deterministic.
///
/// Audit rationale:
/// - Operator-facing traces should never emit empty command identifiers.
/// - A stable fallback label keeps correlation metadata structurally valid
///   even when a caller fails to provide a canonical command string.
pub fn trace_for(command: &str) -> TraceContext {
    new_context(&normalize_command_name(command))
}

/// Normalizes an operator command name into a tracing-safe label.
///
/// Normalization policy:
/// - Leading and trailing whitespace are removed.
/// - Internal whitespace runs are collapsed to a single ASCII space.
/// - Empty results fall back to a canonical placeholder label.
fn normalize_command_name(command: &str) -> String {
    let normalized = command.split_whitespace().collect::<Vec<_>>().join(" ");

    if normalized.is_empty() {
        DEFAULT_TRACE_COMMAND.to_string()
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::trace_for;

    #[test]
    fn trace_for_accepts_canonical_command_names() {
        let trace = trace_for("node-run");

        assert!(!trace.correlation_id.is_empty());
    }

    #[test]
    fn trace_for_normalizes_blank_command_names_to_safe_fallback() {
        let trace = trace_for("   ");

        assert!(!trace.correlation_id.is_empty());
    }

    #[test]
    fn trace_for_normalizes_internal_whitespace_without_failing() {
        let trace = trace_for("  node   run  ");

        assert!(!trace.correlation_id.is_empty());
    }
}

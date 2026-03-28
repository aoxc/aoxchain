// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use chrono::Utc;
use serde::Serialize;
use sha3::{Digest, Sha3_256};

const DEFAULT_TRACE_COMMAND: &str = "unknown-command";
const TRACE_ID_HEX_PREFIX_LEN: usize = 16;
const TRACE_DOMAIN: &str = "AOXC_TRACE_CONTEXT_V1";

/// Canonical AOXC trace context used by operator-facing command surfaces.
///
/// Design intent:
/// - Provide a compact correlation identifier suitable for CLI envelopes,
///   diagnostics, and audit-oriented logs.
/// - Preserve a stable timestamp field that records when the context was
///   created.
/// - Avoid empty command labels by normalizing input into a safe fallback.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TraceContext {
    pub correlation_id: String,
    pub recorded_at: String,
}

impl TraceContext {
    /// Constructs a canonical trace context from explicit fields.
    pub fn new(correlation_id: String, recorded_at: String) -> Self {
        Self {
            correlation_id,
            recorded_at,
        }
    }

    /// Returns `true` when the trace context is structurally complete.
    pub fn is_complete(&self) -> bool {
        !self.correlation_id.is_empty() && !self.recorded_at.is_empty()
    }
}

/// Builds a new canonical AOXC trace context for the supplied command.
///
/// Normalization policy:
/// - Leading and trailing whitespace are removed.
/// - Internal whitespace runs are collapsed to a single ASCII space.
/// - Blank input falls back to a stable placeholder command label.
///
/// Correlation policy:
/// - The correlation identifier is derived from a domain-separated hash over
///   the normalized command and creation timestamp.
/// - Only the leading fixed-width hex prefix is exposed to keep the identifier
///   compact for operator-facing output.
pub fn new_context(command: &str) -> TraceContext {
    let normalized_command = normalize_command_name(command);
    let recorded_at = Utc::now().to_rfc3339();
    let correlation_id = derive_correlation_id(&normalized_command, &recorded_at);

    TraceContext::new(correlation_id, recorded_at)
}

/// Normalizes a command label into a tracing-safe canonical value.
fn normalize_command_name(command: &str) -> String {
    let normalized = command.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        DEFAULT_TRACE_COMMAND.to_string()
    } else {
        normalized
    }
}

/// Derives a compact correlation identifier from normalized trace inputs.
///
/// Security rationale:
/// - Domain separation prevents accidental cross-surface hash reuse.
/// - Explicit `0x00` delimiters remove concatenation ambiguity.
fn derive_correlation_id(command: &str, recorded_at: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(TRACE_DOMAIN.as_bytes());
    hasher.update([0]);
    hasher.update(command.as_bytes());
    hasher.update([0]);
    hasher.update(recorded_at.as_bytes());

    let digest_hex = hex::encode(hasher.finalize());
    digest_hex[..TRACE_ID_HEX_PREFIX_LEN].to_string()
}

#[cfg(test)]
mod tests {
    use super::{derive_correlation_id, new_context, normalize_command_name, TraceContext};

    #[test]
    fn normalize_command_name_preserves_canonical_command() {
        let normalized = normalize_command_name("node-run");
        assert_eq!(normalized, "node-run");
    }

    #[test]
    fn normalize_command_name_collapses_whitespace() {
        let normalized = normalize_command_name("  node   run  ");
        assert_eq!(normalized, "node run");
    }

    #[test]
    fn normalize_command_name_falls_back_when_blank() {
        let normalized = normalize_command_name("   ");
        assert_eq!(normalized, "unknown-command");
    }

    #[test]
    fn derive_correlation_id_is_stable_for_same_input() {
        let a = derive_correlation_id("node-run", "2026-03-28T00:00:00Z");
        let b = derive_correlation_id("node-run", "2026-03-28T00:00:00Z");

        assert_eq!(a, b);
        assert_eq!(a.len(), 16);
        assert!(a.chars().all(|ch| ch.is_ascii_hexdigit()));
    }

    #[test]
    fn derive_correlation_id_changes_when_input_changes() {
        let a = derive_correlation_id("node-run", "2026-03-28T00:00:00Z");
        let b = derive_correlation_id("node-health", "2026-03-28T00:00:00Z");

        assert_ne!(a, b);
    }

    #[test]
    fn new_context_returns_complete_trace_context() {
        let trace = new_context("node-run");

        assert!(trace.is_complete());
        assert_eq!(trace.correlation_id.len(), 16);
    }

    #[test]
    fn trace_context_new_preserves_supplied_fields() {
        let trace = TraceContext::new(
            "abcd1234abcd1234".to_string(),
            "2026-03-28T00:00:00Z".to_string(),
        );

        assert_eq!(trace.correlation_id, "abcd1234abcd1234");
        assert_eq!(trace.recorded_at, "2026-03-28T00:00:00Z");
        assert!(trace.is_complete());
    }
}

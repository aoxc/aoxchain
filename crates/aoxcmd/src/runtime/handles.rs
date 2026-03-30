// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::Serialize;

const DEFAULT_NODE_HANDLE: &str = "local-node-handle";
const DEFAULT_TELEMETRY_HANDLE: &str = "local-telemetry-handle";
const DEFAULT_LEDGER_HANDLE: &str = "local-ledger-handle";

/// Canonical runtime handle set exposed by AOXC operator-facing status surfaces.
///
/// Design intent:
/// - Provide a compact, serializable summary of the runtime subsystems that are
///   expected to be attached to the local operator process.
/// - Keep the structure lightweight and stable for diagnostics, audit reports,
///   and runtime-status inspection commands.
/// - Avoid exposing internal pointer-like implementation detail; these values
///   are descriptive logical handles, not memory references.
///
/// Current semantics:
/// - `node` identifies the local node execution surface.
/// - `telemetry` identifies the local observability/export surface.
/// - `ledger` identifies the local economy or ledger access surface.
///
/// Operational note:
/// - The current default values are static logical placeholders. They express
///   the intended local handle topology, not dynamically allocated runtime IDs.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuntimeHandleSet {
    pub node: &'static str,
    pub telemetry: &'static str,
    pub ledger: &'static str,
}

impl RuntimeHandleSet {
    /// Constructs a canonical runtime handle set from explicit logical labels.
    pub const fn new(node: &'static str, telemetry: &'static str, ledger: &'static str) -> Self {
        Self {
            node,
            telemetry,
            ledger,
        }
    }

    /// Returns `true` when all logical runtime handles are present and non-empty.
    ///
    /// This is primarily a defensive integrity helper for tests and read-only
    /// diagnostics surfaces.
    pub fn is_complete(&self) -> bool {
        !self.node.is_empty() && !self.telemetry.is_empty() && !self.ledger.is_empty()
    }
}

/// Returns the canonical default AOXC runtime handle set.
///
/// Default policy:
/// - The AOXC command plane currently exposes a deterministic local runtime
///   topology with one logical handle per major subsystem.
/// - These labels are intentionally stable so downstream diagnostics and
///   operator tooling can rely on a predictable serialized surface.
pub fn default_handles() -> RuntimeHandleSet {
    RuntimeHandleSet::new(
        DEFAULT_NODE_HANDLE,
        DEFAULT_TELEMETRY_HANDLE,
        DEFAULT_LEDGER_HANDLE,
    )
}

#[cfg(test)]
mod tests {
    use super::{RuntimeHandleSet, default_handles};

    #[test]
    fn default_handles_returns_canonical_logical_runtime_handles() {
        let handles = default_handles();

        assert_eq!(handles.node, "local-node-handle");
        assert_eq!(handles.telemetry, "local-telemetry-handle");
        assert_eq!(handles.ledger, "local-ledger-handle");
    }

    #[test]
    fn runtime_handle_set_new_preserves_supplied_labels() {
        let handles = RuntimeHandleSet::new("node-a", "telemetry-a", "ledger-a");

        assert_eq!(handles.node, "node-a");
        assert_eq!(handles.telemetry, "telemetry-a");
        assert_eq!(handles.ledger, "ledger-a");
    }

    #[test]
    fn runtime_handle_set_reports_completeness_for_non_empty_labels() {
        let handles = default_handles();

        assert!(handles.is_complete());
    }
}

// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::Serialize;

const DEFAULT_CONSENSUS_MODE: &str = "deterministic-local";
const DEFAULT_QUORUM_PROFILE: &str = "single-operator-bootstrap";

/// Canonical consensus-status surface exposed by AOXC runtime inspection flows.
///
/// Design intent:
/// - Provide a compact, serializable summary of the effective consensus mode
///   visible to operator-facing runtime status commands.
/// - Preserve a stable output contract for diagnostics, audit evidence, and
///   bootstrap readiness surfaces.
/// - Avoid exposing low-level internal consensus state in lightweight status
///   contexts where only the operating mode is required.
///
/// Current semantics:
/// - `consensus_mode` identifies the active consensus execution posture.
/// - `quorum_profile` identifies the currently expected quorum topology.
///
/// Operational note:
/// - The default values below intentionally describe the current local bootstrap
///   posture rather than a dynamic distributed validator quorum.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UnityStatus {
    pub consensus_mode: &'static str,
    pub quorum_profile: &'static str,
}

impl UnityStatus {
    /// Constructs a canonical consensus-status surface from explicit logical labels.
    pub const fn new(consensus_mode: &'static str, quorum_profile: &'static str) -> Self {
        Self {
            consensus_mode,
            quorum_profile,
        }
    }

    /// Returns `true` when both logical status labels are present and non-empty.
    ///
    /// This helper is primarily intended for defensive integrity checks in tests
    /// and read-only diagnostics surfaces.
    pub fn is_complete(&self) -> bool {
        !self.consensus_mode.is_empty() && !self.quorum_profile.is_empty()
    }
}

/// Returns the canonical AOXC unity-status surface.
///
/// Default policy:
/// - AOXC currently reports a deterministic local consensus posture during
///   bootstrap-oriented and single-operator local execution flows.
/// - These labels are intentionally stable so downstream tooling can rely on a
///   predictable operator-facing status contract.
pub fn unity_status() -> UnityStatus {
    UnityStatus::new(DEFAULT_CONSENSUS_MODE, DEFAULT_QUORUM_PROFILE)
}

#[cfg(test)]
mod tests {
    use super::{unity_status, UnityStatus};

    #[test]
    fn unity_status_returns_canonical_bootstrap_consensus_surface() {
        let status = unity_status();

        assert_eq!(status.consensus_mode, "deterministic-local");
        assert_eq!(status.quorum_profile, "single-operator-bootstrap");
    }

    #[test]
    fn unity_status_new_preserves_supplied_labels() {
        let status = UnityStatus::new("bft-cluster", "validator-quorum");

        assert_eq!(status.consensus_mode, "bft-cluster");
        assert_eq!(status.quorum_profile, "validator-quorum");
    }

    #[test]
    fn unity_status_reports_completeness_for_non_empty_labels() {
        let status = unity_status();

        assert!(status.is_complete());
    }
}

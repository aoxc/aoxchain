//! AOXC Quantum-Resilient Key Fabric (QRKF) kernel primitives.
//!
//! This module defines protocol-level identity, profile, lane, and continuity
//! surfaces so key management can evolve without hardcoding a single algorithm
//! family into transaction semantics.

use crate::crypto::fingerprints::canonical_execution_fingerprint;

/// Realm-level identity partition for authority separation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyRealm {
    User,
    Node,
    Validator,
    Governance,
    Treasury,
    Package,
    Recovery,
    Audit,
    Institutional,
    BreakGlass,
}

impl KeyRealm {
    /// Stable realm identifier used by canonical identity envelopes.
    pub const fn as_tag(self) -> &'static str {
        match self {
            Self::User => "usr",
            Self::Node => "nod",
            Self::Validator => "val",
            Self::Governance => "gov",
            Self::Treasury => "tre",
            Self::Package => "pkg",
            Self::Recovery => "rcv",
            Self::Audit => "aud",
            Self::Institutional => "int",
            Self::BreakGlass => "brk",
        }
    }
}

/// Kernel-recognized profile identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CryptoProfileId {
    AoxcCl1,
    AoxcHy1,
    AoxcPq1,
    AoxcHq1,
    AoxcHq2,
    AoxcNod1,
    AoxcRcv1,
}

impl CryptoProfileId {
    /// Stable wire identifier used by profile registry evidence.
    pub const fn wire_id(self) -> &'static str {
        match self {
            Self::AoxcCl1 => "AOXC-CL-1",
            Self::AoxcHy1 => "AOXC-HY-1",
            Self::AoxcPq1 => "AOXC-PQ-1",
            Self::AoxcHq1 => "AOXC-HQ-1",
            Self::AoxcHq2 => "AOXC-HQ-2",
            Self::AoxcNod1 => "AOXC-NOD-1",
            Self::AoxcRcv1 => "AOXC-RCV-1",
        }
    }
}

/// Heterogeneous authorization lanes enforced by policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorizationLane {
    ClassicalAuthenticity,
    PostQuantumAuthenticity,
    Authority,
    HardwareCustody,
    Continuity,
    RecoveryVeto,
}

/// Canonical lane quorum policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanePolicy {
    pub min_approved_lanes: u8,
    pub require_authority_lane: bool,
    pub require_continuity_lane: bool,
    pub require_recovery_veto_lane: bool,
}

impl LanePolicy {
    /// Evaluates whether lane approvals satisfy this policy.
    pub fn is_satisfied(self, approvals: &[AuthorizationLane]) -> bool {
        let lane_count = approvals.len();
        if lane_count < self.min_approved_lanes as usize {
            return false;
        }

        let has_authority = approvals.contains(&AuthorizationLane::Authority);
        let has_continuity = approvals.contains(&AuthorizationLane::Continuity);
        let has_recovery_veto = approvals.contains(&AuthorizationLane::RecoveryVeto);

        (!self.require_authority_lane || has_authority)
            && (!self.require_continuity_lane || has_continuity)
            && (!self.require_recovery_veto_lane || has_recovery_veto)
    }
}

/// Epoch-scoped key bundle metadata for continuity checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpochKeyBundle {
    pub realm: KeyRealm,
    pub profile_id: CryptoProfileId,
    pub epoch_id: u64,
    pub valid_from: u64,
    pub valid_until: u64,
    pub predecessor_fingerprint: Option<String>,
    pub rotation_reason: String,
}

impl EpochKeyBundle {
    /// Returns canonical bundle fingerprint used for predecessor linking.
    pub fn canonical_fingerprint(&self) -> String {
        let payload = format!(
            "{}|{}|{}|{}|{}|{}",
            self.realm.as_tag(),
            self.profile_id.wire_id(),
            self.epoch_id,
            self.valid_from,
            self.valid_until,
            self.rotation_reason
        );
        canonical_execution_fingerprint(b"qrkf/epoch-bundle/v1", payload.as_bytes())
    }

    /// Validates local constraints and predecessor linkage.
    pub fn continuity_is_valid(&self, predecessor: Option<&EpochKeyBundle>) -> bool {
        if self.valid_from >= self.valid_until {
            return false;
        }

        if self.rotation_reason.trim().is_empty() {
            return false;
        }

        match predecessor {
            None => self.predecessor_fingerprint.is_none(),
            Some(prev) => {
                self.realm == prev.realm
                    && self.epoch_id == prev.epoch_id + 1
                    && self.valid_from >= prev.valid_until
                    && self.predecessor_fingerprint.as_deref() == Some(&prev.canonical_fingerprint())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthorizationLane, CryptoProfileId, EpochKeyBundle, KeyRealm, LanePolicy};

    #[test]
    fn lane_policy_requires_expected_lanes() {
        let policy = LanePolicy {
            min_approved_lanes: 3,
            require_authority_lane: true,
            require_continuity_lane: true,
            require_recovery_veto_lane: false,
        };

        assert!(policy.is_satisfied(&[
            AuthorizationLane::Authority,
            AuthorizationLane::Continuity,
            AuthorizationLane::PostQuantumAuthenticity
        ]));

        assert!(!policy.is_satisfied(&[
            AuthorizationLane::ClassicalAuthenticity,
            AuthorizationLane::Continuity,
            AuthorizationLane::PostQuantumAuthenticity
        ]));
    }

    #[test]
    fn continuity_chain_requires_predecessor_link() {
        let first = EpochKeyBundle {
            realm: KeyRealm::Governance,
            profile_id: CryptoProfileId::AoxcHq1,
            epoch_id: 1,
            valid_from: 100,
            valid_until: 200,
            predecessor_fingerprint: None,
            rotation_reason: "bootstrap".to_owned(),
        };
        assert!(first.continuity_is_valid(None));

        let second = EpochKeyBundle {
            realm: KeyRealm::Governance,
            profile_id: CryptoProfileId::AoxcHq2,
            epoch_id: 2,
            valid_from: 200,
            valid_until: 300,
            predecessor_fingerprint: Some(first.canonical_fingerprint()),
            rotation_reason: "profile-upgrade".to_owned(),
        };
        assert!(second.continuity_is_valid(Some(&first)));

        let broken = EpochKeyBundle {
            predecessor_fingerprint: Some("wrong".to_owned()),
            ..second.clone()
        };
        assert!(!broken.continuity_is_valid(Some(&first)));
    }

    #[test]
    fn profile_wire_ids_are_stable() {
        assert_eq!(CryptoProfileId::AoxcCl1.wire_id(), "AOXC-CL-1");
        assert_eq!(CryptoProfileId::AoxcHy1.wire_id(), "AOXC-HY-1");
        assert_eq!(CryptoProfileId::AoxcPq1.wire_id(), "AOXC-PQ-1");
        assert_eq!(CryptoProfileId::AoxcHq1.wire_id(), "AOXC-HQ-1");
        assert_eq!(CryptoProfileId::AoxcHq2.wire_id(), "AOXC-HQ-2");
        assert_eq!(CryptoProfileId::AoxcNod1.wire_id(), "AOXC-NOD-1");
        assert_eq!(CryptoProfileId::AoxcRcv1.wire_id(), "AOXC-RCV-1");
    }
}

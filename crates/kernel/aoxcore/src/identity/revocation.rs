// AOXC MIT License
// Experimental software under active construction.
// This file is part of the AOXC pre-release codebase.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Maximum accepted actor identifier length.
const MAX_ACTOR_ID_LEN: usize = 128;

/// Maximum accepted free-form revocation reason length.
const MAX_REASON_TEXT_LEN: usize = 256;

/// Revocation reason codes.
///
/// These values allow operators and auditors to understand why an identity
/// was revoked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevocationReason {
    KeyCompromise,
    OperatorAction,
    GovernanceDecision,
    ExpiredCertificate,
    Other(String),
}

impl RevocationReason {
    /// Returns a stable symbolic reason code.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::KeyCompromise => "KEY_COMPROMISE",
            Self::OperatorAction => "OPERATOR_ACTION",
            Self::GovernanceDecision => "GOVERNANCE_DECISION",
            Self::ExpiredCertificate => "EXPIRED_CERTIFICATE",
            Self::Other(_) => "OTHER",
        }
    }

    /// Returns a canonicalized revocation reason.
    pub fn canonicalize(&self) -> Result<Self, RevocationError> {
        match self {
            Self::KeyCompromise => Ok(Self::KeyCompromise),
            Self::OperatorAction => Ok(Self::OperatorAction),
            Self::GovernanceDecision => Ok(Self::GovernanceDecision),
            Self::ExpiredCertificate => Ok(Self::ExpiredCertificate),
            Self::Other(value) => {
                if value.is_empty() || value.trim().is_empty() {
                    return Err(RevocationError::InvalidReason);
                }

                if value != value.trim() {
                    return Err(RevocationError::InvalidReason);
                }

                if value.len() > MAX_REASON_TEXT_LEN {
                    return Err(RevocationError::InvalidReason);
                }

                Ok(Self::Other(value.to_string()))
            }
        }
    }
}

/// Represents a single revocation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevocationEntry {
    pub actor_id: String,
    pub reason: RevocationReason,
    pub revoked_at: u64,
}

impl RevocationEntry {
    /// Validates a revocation entry as a self-consistent record.
    pub fn validate(&self) -> Result<(), RevocationError> {
        canonicalize_actor_id(&self.actor_id)?;
        self.reason.canonicalize()?;

        if self.revoked_at == 0 {
            return Err(RevocationError::InvalidRevokedAt);
        }

        Ok(())
    }
}

/// Canonical revocation-list error surface.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RevocationError {
    EmptyActorId,
    InvalidActorId,
    InvalidReason,
    InvalidRevokedAt,
    TimeError,
    SerializationFailed(String),
    ParseFailed(String),
}

impl RevocationError {
    /// Returns a stable symbolic error code suitable for logs and telemetry.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmptyActorId => "REVOCATION_EMPTY_ACTOR_ID",
            Self::InvalidActorId => "REVOCATION_INVALID_ACTOR_ID",
            Self::InvalidReason => "REVOCATION_INVALID_REASON",
            Self::InvalidRevokedAt => "REVOCATION_INVALID_REVOKED_AT",
            Self::TimeError => "REVOCATION_TIME_ERROR",
            Self::SerializationFailed(_) => "REVOCATION_SERIALIZATION_FAILED",
            Self::ParseFailed(_) => "REVOCATION_PARSE_FAILED",
        }
    }
}

impl fmt::Display for RevocationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyActorId => {
                write!(f, "revocation operation failed: actor_id must not be empty")
            }
            Self::InvalidActorId => {
                write!(f, "revocation operation failed: actor_id is not canonical")
            }
            Self::InvalidReason => {
                write!(f, "revocation operation failed: reason is invalid")
            }
            Self::InvalidRevokedAt => {
                write!(f, "revocation operation failed: revoked_at is invalid")
            }
            Self::TimeError => {
                write!(f, "revocation operation failed: system time is invalid")
            }
            Self::SerializationFailed(error) => {
                write!(f, "revocation serialization failed: {}", error)
            }
            Self::ParseFailed(error) => {
                write!(f, "revocation parsing failed: {}", error)
            }
        }
    }
}

impl std::error::Error for RevocationError {}

/// In-memory revocation list.
///
/// This structure functions similarly to a CRL but is optimized for fast
/// lookup and deterministic export.
///
/// Design properties:
/// - deterministic ordering,
/// - strict actor-id validation,
/// - explicit revocation-entry validation,
/// - serialization-friendly shape for synchronization and persistence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevocationList {
    revoked: BTreeMap<String, RevocationEntry>,
}

impl Default for RevocationList {
    fn default() -> Self {
        Self::new()
    }
}

impl RevocationList {
    /// Creates an empty revocation list.
    #[must_use]
    pub fn new() -> Self {
        Self {
            revoked: BTreeMap::new(),
        }
    }

    /// Validates the internal revocation-list state.
    ///
    /// Validation policy:
    /// - each key must match the embedded `actor_id`,
    /// - every actor_id must be canonical,
    /// - every reason must be canonical,
    /// - every `revoked_at` value must be non-zero.
    pub fn validate(&self) -> Result<(), RevocationError> {
        for (key, entry) in &self.revoked {
            let canonical_actor_id = canonicalize_actor_id(&entry.actor_id)?;

            if key != &canonical_actor_id {
                return Err(RevocationError::InvalidActorId);
            }

            entry.validate()?;
        }

        Ok(())
    }

    /// Revokes an actor identity using the current system time.
    ///
    /// Compatibility note:
    /// this method preserves the legacy infallible surface and ignores invalid
    /// input or time acquisition failures. New call paths should prefer
    /// `try_revoke` or `revoke_at` for explicit error handling.
    pub fn revoke(&mut self, actor_id: &str, reason: RevocationReason) {
        let _ = self.try_revoke(actor_id, reason);
    }

    /// Revokes an actor identity using the current system time.
    ///
    /// Return value:
    /// - `Ok(true)` if the actor was newly revoked,
    /// - `Ok(false)` if the actor was already revoked.
    pub fn try_revoke(
        &mut self,
        actor_id: &str,
        reason: RevocationReason,
    ) -> Result<bool, RevocationError> {
        let now = current_time()?;
        self.revoke_at(actor_id, reason, now)
    }

    /// Revokes an actor identity at an explicit timestamp.
    ///
    /// Return value:
    /// - `Ok(true)` if the actor was newly revoked,
    /// - `Ok(false)` if the actor was already revoked.
    pub fn revoke_at(
        &mut self,
        actor_id: &str,
        reason: RevocationReason,
        revoked_at: u64,
    ) -> Result<bool, RevocationError> {
        let canonical_actor_id = canonicalize_actor_id(actor_id)?;
        let canonical_reason = reason.canonicalize()?;

        if revoked_at == 0 {
            return Err(RevocationError::InvalidRevokedAt);
        }

        if self.revoked.contains_key(&canonical_actor_id) {
            return Ok(false);
        }

        let entry = RevocationEntry {
            actor_id: canonical_actor_id.clone(),
            reason: canonical_reason,
            revoked_at,
        };

        self.revoked.insert(canonical_actor_id, entry);
        Ok(true)
    }

    /// Returns true if an actor identity has been revoked.
    ///
    /// Compatibility behavior:
    /// - invalid actor identifiers resolve to `false`.
    #[must_use]
    pub fn is_revoked(&self, actor_id: &str) -> bool {
        let Ok(canonical_actor_id) = canonicalize_actor_id(actor_id) else {
            return false;
        };

        self.revoked.contains_key(&canonical_actor_id)
    }

    /// Returns revocation metadata if present.
    ///
    /// Compatibility behavior:
    /// - invalid actor identifiers resolve to `None`.
    #[must_use]
    pub fn get(&self, actor_id: &str) -> Option<&RevocationEntry> {
        let canonical_actor_id = canonicalize_actor_id(actor_id).ok()?;
        self.revoked.get(&canonical_actor_id)
    }

    /// Returns the number of revoked identities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.revoked.len()
    }

    /// Returns true if the revocation list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.revoked.is_empty()
    }

    /// Deterministically exports the revoked actor IDs.
    ///
    /// Useful for hashing or gossip synchronization.
    #[must_use]
    pub fn export_actor_ids(&self) -> Vec<String> {
        self.revoked.keys().cloned().collect()
    }

    /// Deterministically exports revocation entries.
    #[must_use]
    pub fn export_entries(&self) -> Vec<RevocationEntry> {
        self.revoked.values().cloned().collect()
    }

    /// Returns a deterministic set of revoked actors.
    ///
    /// Compatibility note:
    /// this retains the legacy `HashSet` return type even though the internal
    /// state is ordered.
    #[must_use]
    pub fn export_set(&self) -> HashSet<String> {
        self.revoked.keys().cloned().collect()
    }

    /// Serializes the revocation list to pretty JSON.
    pub fn to_json(&self) -> Result<String, RevocationError> {
        self.validate()?;

        serde_json::to_string_pretty(self)
            .map_err(|error| RevocationError::SerializationFailed(error.to_string()))
    }

    /// Restores a revocation list from JSON and validates it.
    pub fn from_json(data: &str) -> Result<Self, RevocationError> {
        let list: Self = serde_json::from_str(data)
            .map_err(|error| RevocationError::ParseFailed(error.to_string()))?;

        list.validate()?;
        Ok(list)
    }
}

/// Returns current UNIX timestamp in seconds.
///
/// Unlike the previous lenient behavior, invalid system time is surfaced
/// explicitly so that callers can fail safely rather than silently recording
/// a zero timestamp.
fn current_time() -> Result<u64, RevocationError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .map_err(|_| RevocationError::TimeError)
}

/// Canonicalizes and validates an AOXC actor identifier.
///
/// Policy:
/// - must not be blank,
/// - surrounding whitespace is rejected rather than normalized,
/// - must be bounded,
/// - must begin with `AOXC-`,
/// - only ASCII alphanumeric characters plus `_`, `-`, and `.` are accepted.
fn canonicalize_actor_id(actor_id: &str) -> Result<String, RevocationError> {
    if actor_id.is_empty() || actor_id.trim().is_empty() {
        return Err(RevocationError::EmptyActorId);
    }

    if actor_id != actor_id.trim() {
        return Err(RevocationError::InvalidActorId);
    }

    if actor_id.len() > MAX_ACTOR_ID_LEN {
        return Err(RevocationError::InvalidActorId);
    }

    if !actor_id.starts_with("AOXC-") {
        return Err(RevocationError::InvalidActorId);
    }

    if !actor_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
    {
        return Err(RevocationError::InvalidActorId);
    }

    Ok(actor_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revoke_and_query_actor() {
        let mut crl = RevocationList::new();

        let inserted = crl
            .revoke_at("AOXC-VAL-EU-1234", RevocationReason::KeyCompromise, 100)
            .unwrap();

        assert!(inserted);
        assert!(crl.is_revoked("AOXC-VAL-EU-1234"));
        assert_eq!(crl.len(), 1);
    }

    #[test]
    fn duplicate_revoke_returns_false() {
        let mut crl = RevocationList::new();

        let first = crl
            .revoke_at("AOXC-NOD-EU-1", RevocationReason::OperatorAction, 100)
            .unwrap();
        let second = crl
            .revoke_at("AOXC-NOD-EU-1", RevocationReason::OperatorAction, 101)
            .unwrap();

        assert!(first);
        assert!(!second);
        assert_eq!(crl.len(), 1);
        assert_eq!(crl.get("AOXC-NOD-EU-1").unwrap().revoked_at, 100);
    }

    #[test]
    fn deterministic_export() {
        let mut crl = RevocationList::new();

        crl.revoke_at("AOXC-B", RevocationReason::OperatorAction, 100)
            .unwrap();
        crl.revoke_at("AOXC-A", RevocationReason::OperatorAction, 100)
            .unwrap();

        let list = crl.export_actor_ids();

        assert_eq!(list, vec!["AOXC-A".to_string(), "AOXC-B".to_string()]);
    }

    #[test]
    fn export_entries_is_deterministic() {
        let mut crl = RevocationList::new();

        crl.revoke_at("AOXC-B", RevocationReason::OperatorAction, 200)
            .unwrap();
        crl.revoke_at("AOXC-A", RevocationReason::KeyCompromise, 100)
            .unwrap();

        let entries = crl.export_entries();

        assert_eq!(entries[0].actor_id, "AOXC-A");
        assert_eq!(entries[0].reason, RevocationReason::KeyCompromise);
        assert_eq!(entries[1].actor_id, "AOXC-B");
        assert_eq!(entries[1].reason, RevocationReason::OperatorAction);
    }

    #[test]
    fn invalid_actor_id_is_rejected() {
        let mut crl = RevocationList::new();

        let error = crl
            .revoke_at("bad actor", RevocationReason::OperatorAction, 100)
            .unwrap_err();

        assert_eq!(error, RevocationError::InvalidActorId);
    }

    #[test]
    fn invalid_reason_is_rejected() {
        let mut crl = RevocationList::new();

        let error = crl
            .revoke_at(
                "AOXC-ACTOR-1",
                RevocationReason::Other("   ".to_string()),
                100,
            )
            .unwrap_err();

        assert_eq!(error, RevocationError::InvalidReason);
    }

    #[test]
    fn invalid_revoked_at_is_rejected() {
        let mut crl = RevocationList::new();

        let error = crl
            .revoke_at("AOXC-ACTOR-2", RevocationReason::OperatorAction, 0)
            .unwrap_err();

        assert_eq!(error, RevocationError::InvalidRevokedAt);
    }

    #[test]
    fn lookup_of_invalid_actor_returns_safe_defaults() {
        let crl = RevocationList::new();

        assert!(!crl.is_revoked(" bad "));
        assert_eq!(crl.get("bad actor"), None);
    }

    #[test]
    fn json_roundtrip_preserves_revocation_list() {
        let mut crl = RevocationList::new();

        crl.revoke_at("AOXC-ACTOR-3", RevocationReason::GovernanceDecision, 100)
            .unwrap();

        let json = crl.to_json().unwrap();
        let restored = RevocationList::from_json(&json).unwrap();

        assert_eq!(crl, restored);
    }

    #[test]
    fn validate_rejects_corrupted_internal_state() {
        let mut revoked = BTreeMap::new();
        revoked.insert(
            "AOXC-ACTOR-4".to_string(),
            RevocationEntry {
                actor_id: "AOXC-ACTOR-4".to_string(),
                reason: RevocationReason::Other(" bad ".to_string()),
                revoked_at: 100,
            },
        );

        let crl = RevocationList { revoked };

        assert_eq!(crl.validate(), Err(RevocationError::InvalidReason));
    }
}

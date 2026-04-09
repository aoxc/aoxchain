//! Canonical auth-profile registry with typed profile identifiers and versioning.
//!
//! This module provides the canonical registry for versioned authentication
//! profiles. Each registry entry binds:
//! - a cryptographic profile,
//! - threshold policy,
//! - quorum policy,
//! - signer-class bindings.
//!
//! Security posture:
//! - deterministic latest-version resolution,
//! - explicit version addressing,
//! - fail-closed duplicate version handling,
//! - full envelope validation delegated through the canonical profile record.

use std::collections::BTreeMap;

use crate::{
    auth::{
        envelope::{AuthEnvelope, AuthEnvelopeLimits},
        policy::QuorumPolicy,
        scheme::AuthProfile,
        signer::SignerClass,
        threshold::ThresholdPolicy,
    },
    errors::{AoxcvmError, AoxcvmResult},
};

/// Typed identifier of an auth profile in registry state.
///
/// This wrapper prevents accidental mixing of raw numeric identifiers with
/// unrelated identifiers elsewhere in the codebase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AuthProfileId(u32);

impl AuthProfileId {
    /// Creates a new typed auth-profile identifier.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the underlying stable numeric identifier.
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// Versioned auth profile record in canonical registry state.
///
/// This structure is intentionally minimal and authoritative. It describes
/// the exact validation policy that must be applied to an incoming auth
/// envelope for a specific profile version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthProfileRecord {
    pub profile: AuthProfile,
    pub threshold: ThresholdPolicy,
    pub quorum: QuorumPolicy,
    /// key_id => signer class binding.
    pub signer_classes: BTreeMap<String, SignerClass>,
}

impl AuthProfileRecord {
    /// Performs full envelope validation using the configured cryptographic
    /// profile, threshold policy, and quorum policy.
    ///
    /// Validation order is intentionally fail-closed:
    /// 1. Envelope structural and cryptographic metadata validation
    /// 2. Threshold policy evaluation
    /// 3. Quorum policy evaluation
    ///
    /// This ordering ensures that malformed or cryptographically incompatible
    /// input is rejected before governance semantics are considered.
    pub fn validate_envelope(
        &self,
        envelope: &AuthEnvelope,
        limits: AuthEnvelopeLimits,
    ) -> AoxcvmResult<()> {
        envelope.validate(self.profile, limits)?;

        if !self.threshold.is_satisfied_by(envelope) {
            return Err(AoxcvmError::PolicyViolation(
                "threshold failed: signer constraints not satisfied",
            ));
        }

        let signer_classes = self
            .signer_classes
            .iter()
            .map(|(key_id, signer_class)| (key_id.as_str(), *signer_class))
            .collect::<BTreeMap<&str, SignerClass>>();

        self.quorum.validate(envelope, &signer_classes)
    }
}

/// Canonical registry of auth profile definitions by id and version.
///
/// Internally a `BTreeMap` is used to preserve deterministic ordering and
/// stable latest-version resolution semantics.
#[derive(Debug, Clone, Default)]
pub struct AuthProfileRegistry {
    records: BTreeMap<AuthProfileId, BTreeMap<u16, AuthProfileRecord>>,
}

impl AuthProfileRegistry {
    /// Inserts a versioned profile record.
    ///
    /// The operation fails if the given profile id already contains the same
    /// version number. Silent overwrite is explicitly forbidden.
    pub fn insert_version(
        &mut self,
        id: AuthProfileId,
        version: u16,
        record: AuthProfileRecord,
    ) -> AoxcvmResult<()> {
        let versions = self.records.entry(id).or_default();

        if versions.contains_key(&version) {
            return Err(AoxcvmError::DuplicateAuthProfileVersion {
                profile_id: id.as_u32(),
                version,
            });
        }

        versions.insert(version, record);
        Ok(())
    }

    /// Resolves an explicit profile version.
    ///
    /// The function fails if either:
    /// - the profile id is unknown, or
    /// - the requested version is not registered for that profile id.
    pub fn get_version(&self, id: AuthProfileId, version: u16) -> AoxcvmResult<&AuthProfileRecord> {
        self.records
            .get(&id)
            .ok_or(AoxcvmError::UnknownAuthProfile {
                profile_id: id.as_u32(),
            })?
            .get(&version)
            .ok_or(AoxcvmError::UnknownAuthProfileVersion {
                profile_id: id.as_u32(),
                version,
            })
    }

    /// Resolves the latest registered version for a profile id.
    ///
    /// Latest-version resolution is deterministic because versions are stored
    /// in an ordered map. The highest numeric version is considered canonical.
    pub fn get_latest(&self, id: AuthProfileId) -> AoxcvmResult<(u16, &AuthProfileRecord)> {
        let versions = self
            .records
            .get(&id)
            .ok_or(AoxcvmError::UnknownAuthProfile {
                profile_id: id.as_u32(),
            })?;

        versions
            .last_key_value()
            .map(|(version, record)| (*version, record))
            .ok_or(AoxcvmError::UnknownAuthProfile {
                profile_id: id.as_u32(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthProfileId, AuthProfileRecord, AuthProfileRegistry};
    use std::collections::BTreeMap;

    use crate::auth::{
        envelope::{AuthEnvelope, AuthEnvelopeLimits, SignatureEntry},
        policy::QuorumPolicy,
        scheme::{AuthProfile, SignatureAlgorithm},
        signer::SignerClass,
        threshold::ThresholdPolicy,
    };

    /// Returns a deterministic test-only signature payload whose size is
    /// compatible with the selected algorithm.
    ///
    /// These values are intentionally chosen to satisfy realistic
    /// post-quantum signature metadata constraints rather than relying on
    /// undersized placeholder buffers that can cause false-negative failures.
    fn valid_test_signature_bytes(algorithm: SignatureAlgorithm, fill: u8) -> Vec<u8> {
        let signature_len = match algorithm {
            SignatureAlgorithm::MlDsa65 => 3309,
            SignatureAlgorithm::MlDsa87 => 4627,
            SignatureAlgorithm::SlhDsa128s => 7856,
            SignatureAlgorithm::SlhDsa128f => 17088,
            SignatureAlgorithm::SlhDsa192s => 16224,
            SignatureAlgorithm::SlhDsa192f => 35664,
            SignatureAlgorithm::SlhDsa256s => 29792,
            SignatureAlgorithm::SlhDsa256f => 49856,
        };

        vec![fill; signature_len]
    }

    /// Builds a deterministic signature entry with algorithm-compatible
    /// metadata. This helper centralizes fixture correctness and prevents
    /// policy tests from failing due to unrelated metadata size violations.
    fn signature_entry(
        algorithm: SignatureAlgorithm,
        key_id: &str,
        fill: u8,
    ) -> SignatureEntry {
        SignatureEntry {
            algorithm,
            key_id: key_id.to_owned(),
            signature: valid_test_signature_bytes(algorithm, fill),
        }
    }

    /// Builds the canonical test record used across registry validation tests.
    fn build_record() -> AuthProfileRecord {
        AuthProfileRecord {
            profile: AuthProfile::PostQuantumStrict,
            threshold: ThresholdPolicy {
                min_signers: 2,
                require_post_quantum: true,
            },
            quorum: QuorumPolicy {
                min_total: 2,
                min_governance: 1,
                min_operations: 0,
                min_system: 1,
            },
            signer_classes: BTreeMap::from([
                ("gov-1".to_owned(), SignerClass::Governance),
                ("sys-1".to_owned(), SignerClass::System),
            ]),
        }
    }

    #[test]
    fn registry_latest_version_resolution_is_deterministic() {
        let mut registry = AuthProfileRegistry::default();
        let id = AuthProfileId::new(7);

        registry
            .insert_version(id, 1, build_record())
            .expect("v1 insertion must succeed");
        registry
            .insert_version(id, 2, build_record())
            .expect("v2 insertion must succeed");

        let (version, _) = registry
            .get_latest(id)
            .expect("latest profile version must resolve deterministically");

        assert_eq!(version, 2);
    }

    #[test]
    fn registry_rejects_duplicate_profile_version() {
        let mut registry = AuthProfileRegistry::default();
        let id = AuthProfileId::new(8);

        registry
            .insert_version(id, 1, build_record())
            .expect("initial version insertion must succeed");

        let duplicate = registry.insert_version(id, 1, build_record());

        assert!(
            duplicate.is_err(),
            "duplicate version insertion must fail closed"
        );
    }

    #[test]
    fn record_validation_enforces_threshold_and_quorum() {
        let record = build_record();

        let envelope = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 1,
            signers: vec![
                signature_entry(SignatureAlgorithm::MlDsa65, "gov-1", 0x09),
                signature_entry(SignatureAlgorithm::MlDsa87, "sys-1", 0x0A),
            ],
        };

        assert!(
            record
                .validate_envelope(&envelope, AuthEnvelopeLimits::default())
                .is_ok(),
            "valid post-quantum signer set must satisfy threshold and quorum"
        );
    }

    #[test]
    fn record_validation_rejects_missing_required_system_quorum() {
        let record = build_record();

        let envelope = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 2,
            signers: vec![
                signature_entry(SignatureAlgorithm::MlDsa65, "gov-1", 0x21),
                signature_entry(SignatureAlgorithm::MlDsa87, "gov-1", 0x22),
            ],
        };

        let err = record
            .validate_envelope(&envelope, AuthEnvelopeLimits::default())
            .expect_err("missing system quorum must be rejected");

        assert!(
            err.to_string().contains("quorum")
                || err.to_string().contains("policy")
                || err.to_string().contains("signer"),
            "unexpected error for quorum rejection: {err}"
        );
    }

    #[test]
    fn explicit_version_resolution_returns_registered_record() {
        let mut registry = AuthProfileRegistry::default();
        let id = AuthProfileId::new(9);

        registry
            .insert_version(id, 3, build_record())
            .expect("version insertion must succeed");

        let resolved = registry
            .get_version(id, 3)
            .expect("explicit version lookup must resolve");

        assert_eq!(resolved.profile, AuthProfile::PostQuantumStrict);
    }

    #[test]
    fn unknown_profile_version_is_rejected() {
        let mut registry = AuthProfileRegistry::default();
        let id = AuthProfileId::new(10);

        registry
            .insert_version(id, 1, build_record())
            .expect("version insertion must succeed");

        let missing = registry.get_version(id, 99);

        assert!(
            missing.is_err(),
            "unknown profile version lookup must fail closed"
        );
    }

    #[test]
    fn unknown_profile_latest_lookup_is_rejected() {
        let registry = AuthProfileRegistry::default();
        let missing = registry.get_latest(AuthProfileId::new(404));

        assert!(
            missing.is_err(),
            "latest-version lookup for unknown profile must fail closed"
        );
    }
}

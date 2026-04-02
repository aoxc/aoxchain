//! Canonical auth-profile registry with typed profile identifiers and versioning.

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthProfileRecord {
    pub profile: AuthProfile,
    pub threshold: ThresholdPolicy,
    pub quorum: QuorumPolicy,
    /// key_id => signer class binding.
    pub signer_classes: BTreeMap<String, SignerClass>,
}

impl AuthProfileRecord {
    /// Full envelope validation using profile, threshold, and quorum rules.
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
            .map(|(k, v)| (k.as_str(), *v))
            .collect::<BTreeMap<&str, SignerClass>>();

        self.quorum.validate(envelope, &signer_classes)
    }
}

/// Canonical registry of auth profile definitions by id and version.
#[derive(Debug, Clone, Default)]
pub struct AuthProfileRegistry {
    records: BTreeMap<AuthProfileId, BTreeMap<u16, AuthProfileRecord>>,
}

impl AuthProfileRegistry {
    /// Inserts a versioned profile record.
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

    /// Resolves latest version for a profile id.
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
    use std::collections::BTreeMap;

    use crate::auth::{
        envelope::{AuthEnvelope, AuthEnvelopeLimits, SignatureEntry},
        policy::QuorumPolicy,
        scheme::{AuthProfile, SignatureAlgorithm},
        signer::SignerClass,
        threshold::ThresholdPolicy,
    };

    use super::{AuthProfileId, AuthProfileRecord, AuthProfileRegistry};

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
            .expect("v1 insert should pass");
        registry
            .insert_version(id, 2, build_record())
            .expect("v2 insert should pass");

        let (version, _) = registry.get_latest(id).expect("latest should resolve");
        assert_eq!(version, 2);
    }

    #[test]
    fn record_validation_enforces_threshold_and_quorum() {
        let record = build_record();

        let envelope = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 1,
            signers: vec![
                SignatureEntry {
                    algorithm: SignatureAlgorithm::MlDsa65,
                    key_id: "gov-1".to_owned(),
                    signature: vec![9_u8; 128],
                },
                SignatureEntry {
                    algorithm: SignatureAlgorithm::MlDsa87,
                    key_id: "sys-1".to_owned(),
                    signature: vec![10_u8; 128],
                },
            ],
        };

        assert!(
            record
                .validate_envelope(&envelope, AuthEnvelopeLimits::default())
                .is_ok()
        );
    }
}

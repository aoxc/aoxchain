//! Quorum-policy helpers for multi-signer constitutional auth checks.

use std::collections::BTreeMap;

use crate::{
    auth::{envelope::AuthEnvelope, signer::SignerClass},
    errors::{AoxcvmError, AoxcvmResult},
};

/// Class-aware quorum requirements for envelope approval.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuorumPolicy {
    /// Minimum total number of signers.
    pub min_total: usize,
    /// Required governance signers.
    pub min_governance: usize,
    /// Required operations signers.
    pub min_operations: usize,
    /// Required system signers.
    pub min_system: usize,
}

impl QuorumPolicy {
    /// Validates whether signer classes bound to an envelope satisfy this policy.
    pub fn validate(
        self,
        envelope: &AuthEnvelope,
        signer_classes: &BTreeMap<&str, SignerClass>,
    ) -> AoxcvmResult<()> {
        if envelope.signers.len() < self.min_total {
            return Err(AoxcvmError::PolicyViolation(
                "quorum failed: min_total threshold not met",
            ));
        }

        let mut governance = 0;
        let mut operations = 0;
        let mut system = 0;

        for signer in &envelope.signers {
            let class =
                signer_classes
                    .get(signer.key_id.as_str())
                    .ok_or(AoxcvmError::PolicyViolation(
                        "quorum failed: signer class binding missing",
                    ))?;
            match class {
                SignerClass::Governance => governance += 1,
                SignerClass::Operations => operations += 1,
                SignerClass::System => system += 1,
                SignerClass::Application => {}
            }
        }

        if governance < self.min_governance {
            return Err(AoxcvmError::PolicyViolation(
                "quorum failed: governance signer requirement not met",
            ));
        }
        if operations < self.min_operations {
            return Err(AoxcvmError::PolicyViolation(
                "quorum failed: operations signer requirement not met",
            ));
        }
        if system < self.min_system {
            return Err(AoxcvmError::PolicyViolation(
                "quorum failed: system signer requirement not met",
            ));
        }

        Ok(())
    }
}

impl Default for QuorumPolicy {
    fn default() -> Self {
        Self {
            min_total: 1,
            min_governance: 0,
            min_operations: 0,
            min_system: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::auth::{
        envelope::{AuthEnvelope, SignatureEntry},
        scheme::SignatureAlgorithm,
        signer::SignerClass,
    };

    use super::QuorumPolicy;

    #[test]
    fn quorum_requires_bound_signer_classes() {
        let envelope = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 1,
            signers: vec![SignatureEntry {
                algorithm: SignatureAlgorithm::MlDsa65,
                key_id: "gov-1".to_owned(),
                signature: vec![7_u8; 128],
            }],
        };

        let policy = QuorumPolicy {
            min_total: 1,
            min_governance: 1,
            min_operations: 0,
            min_system: 0,
        };

        let classes = BTreeMap::new();
        assert!(policy.validate(&envelope, &classes).is_err());
    }

    #[test]
    fn quorum_accepts_class_thresholds() {
        let envelope = AuthEnvelope {
            domain: "tx".to_owned(),
            nonce: 2,
            signers: vec![
                SignatureEntry {
                    algorithm: SignatureAlgorithm::MlDsa65,
                    key_id: "gov-1".to_owned(),
                    signature: vec![7_u8; 128],
                },
                SignatureEntry {
                    algorithm: SignatureAlgorithm::MlDsa87,
                    key_id: "sys-1".to_owned(),
                    signature: vec![8_u8; 128],
                },
            ],
        };

        let policy = QuorumPolicy {
            min_total: 2,
            min_governance: 1,
            min_operations: 0,
            min_system: 1,
        };

        let classes = BTreeMap::from([
            ("gov-1", SignerClass::Governance),
            ("sys-1", SignerClass::System),
        ]);

        assert!(policy.validate(&envelope, &classes).is_ok());
    }
}

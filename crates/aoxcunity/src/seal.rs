use serde::{Deserialize, Serialize};

use crate::validator::ValidatorId;

const QUORUM_CERTIFICATE_DOMAIN_V1: &[u8] = b"AOXC_QUORUM_CERTIFICATE_V1";
const AUTHENTICATED_QUORUM_CERTIFICATE_DOMAIN_V1: &[u8] =
    b"AOXC_AUTHENTICATED_QUORUM_CERTIFICATE_V1";

/// Canonical quorum certificate.
///
/// This structure binds finality evidence to a specific block, round, and
/// signer set. In this phase the certificate is built from authenticated vote
/// admission and deterministic signer ordering, providing a stable proof shape
/// for future signature aggregation upgrades.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuorumCertificate {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub signers: Vec<ValidatorId>,
    pub observed_voting_power: u64,
    pub total_voting_power: u64,
    pub threshold_numerator: u64,
    pub threshold_denominator: u64,
    pub certificate_hash: [u8; 32],
}

impl QuorumCertificate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        block_hash: [u8; 32],
        height: u64,
        round: u64,
        signers: Vec<ValidatorId>,
        observed_voting_power: u64,
        total_voting_power: u64,
        threshold_numerator: u64,
        threshold_denominator: u64,
    ) -> Self {
        let signers = canonicalize_signers(signers);
        let certificate_hash = compute_certificate_hash(
            block_hash,
            height,
            round,
            &signers,
            observed_voting_power,
            total_voting_power,
            threshold_numerator,
            threshold_denominator,
        );

        Self {
            block_hash,
            height,
            round,
            signers,
            observed_voting_power,
            total_voting_power,
            threshold_numerator,
            threshold_denominator,
            certificate_hash,
        }
    }
}

/// Finalized block seal.
///
/// This type models the minimum cryptographic or quorum-backed evidence
/// required to mark a block as finalized in the fork-choice view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockSeal {
    pub block_hash: [u8; 32],
    pub finalized_round: u64,
    pub attestation_root: [u8; 32],
    pub certificate: QuorumCertificate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticatedQuorumCertificate {
    pub certificate: QuorumCertificate,
    pub network_id: u32,
    pub epoch: u64,
    pub validator_set_root: [u8; 32],
    pub signature_scheme: u16,
    pub authenticated_hash: [u8; 32],
}

impl AuthenticatedQuorumCertificate {
    #[must_use]
    pub fn new(
        certificate: QuorumCertificate,
        network_id: u32,
        epoch: u64,
        validator_set_root: [u8; 32],
        signature_scheme: u16,
    ) -> Self {
        let authenticated_hash = compute_authenticated_certificate_hash(
            &certificate,
            network_id,
            epoch,
            validator_set_root,
            signature_scheme,
        );
        Self {
            certificate,
            network_id,
            epoch,
            validator_set_root,
            signature_scheme,
            authenticated_hash,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn compute_certificate_hash(
    block_hash: [u8; 32],
    height: u64,
    round: u64,
    signers: &[ValidatorId],
    observed_voting_power: u64,
    total_voting_power: u64,
    threshold_numerator: u64,
    threshold_denominator: u64,
) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(QUORUM_CERTIFICATE_DOMAIN_V1);
    hasher.update(block_hash);
    hasher.update(height.to_le_bytes());
    hasher.update(round.to_le_bytes());
    hasher.update((signers.len() as u64).to_le_bytes());
    for signer in signers {
        hasher.update(signer);
    }
    hasher.update(observed_voting_power.to_le_bytes());
    hasher.update(total_voting_power.to_le_bytes());
    hasher.update(threshold_numerator.to_le_bytes());
    hasher.update(threshold_denominator.to_le_bytes());
    hasher.finalize().into()
}

fn compute_authenticated_certificate_hash(
    certificate: &QuorumCertificate,
    network_id: u32,
    epoch: u64,
    validator_set_root: [u8; 32],
    signature_scheme: u16,
) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(AUTHENTICATED_QUORUM_CERTIFICATE_DOMAIN_V1);
    hasher.update(network_id.to_le_bytes());
    hasher.update(epoch.to_le_bytes());
    hasher.update(validator_set_root);
    hasher.update(signature_scheme.to_le_bytes());
    hasher.update(certificate.certificate_hash);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::{AuthenticatedQuorumCertificate, QuorumCertificate};

    #[test]
    fn quorum_certificate_hash_is_deterministic_for_signer_order() {
        let a = QuorumCertificate::new([1u8; 32], 10, 3, vec![[2u8; 32], [1u8; 32]], 20, 30, 2, 3);
        let b = QuorumCertificate::new([1u8; 32], 10, 3, vec![[1u8; 32], [2u8; 32]], 20, 30, 2, 3);

        assert_eq!(a.certificate_hash, b.certificate_hash);
        assert_eq!(a.signers, b.signers);
    }

    #[test]
    fn authenticated_certificate_hash_changes_with_context() {
        let certificate =
            QuorumCertificate::new([1u8; 32], 10, 3, vec![[1u8; 32], [2u8; 32]], 20, 30, 2, 3);
        let a = AuthenticatedQuorumCertificate::new(certificate.clone(), 2626, 1, [9u8; 32], 1);
        let b = AuthenticatedQuorumCertificate::new(certificate, 2626, 2, [9u8; 32], 1);

        assert_ne!(a.authenticated_hash, b.authenticated_hash);
    }
}

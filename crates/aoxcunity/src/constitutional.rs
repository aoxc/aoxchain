use serde::{Deserialize, Serialize};

use crate::seal::QuorumCertificate;
use crate::validator::ValidatorId;

const EXECUTION_CERTIFICATE_DOMAIN_V1: &[u8] = b"AOXC_EXECUTION_CERTIFICATE_V1";
const LEGITIMACY_CERTIFICATE_DOMAIN_V1: &[u8] = b"AOXC_LEGITIMACY_CERTIFICATE_V1";
const CONTINUITY_CERTIFICATE_DOMAIN_V1: &[u8] = b"AOXC_CONTINUITY_CERTIFICATE_V1";
const CONSTITUTIONAL_SEAL_DOMAIN_V1: &[u8] = b"AOXC_CONSTITUTIONAL_SEAL_V1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionCertificate {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub epoch: u64,
    pub validator_set_hash: [u8; 32],
    pub quorum_certificate: QuorumCertificate,
    pub certificate_hash: [u8; 32],
}

impl ExecutionCertificate {
    #[must_use]
    pub fn new(
        epoch: u64,
        validator_set_hash: [u8; 32],
        quorum_certificate: QuorumCertificate,
    ) -> Self {
        let certificate_hash = compute_execution_certificate_hash(
            quorum_certificate.block_hash,
            quorum_certificate.height,
            quorum_certificate.round,
            epoch,
            validator_set_hash,
            quorum_certificate.certificate_hash,
        );

        Self {
            block_hash: quorum_certificate.block_hash,
            height: quorum_certificate.height,
            round: quorum_certificate.round,
            epoch,
            validator_set_hash,
            quorum_certificate,
            certificate_hash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegitimacyCertificate {
    pub block_hash: [u8; 32],
    pub authority_epoch: u64,
    pub constitution_root: [u8; 32],
    pub validator_authority_commitment: [u8; 32],
    pub transition_proof_root: [u8; 32],
    pub signers: Vec<ValidatorId>,
    pub certificate_hash: [u8; 32],
}

impl LegitimacyCertificate {
    #[must_use]
    pub fn new(
        block_hash: [u8; 32],
        authority_epoch: u64,
        constitution_root: [u8; 32],
        validator_authority_commitment: [u8; 32],
        transition_proof_root: [u8; 32],
        mut signers: Vec<ValidatorId>,
    ) -> Self {
        signers.sort();
        signers.dedup();
        let certificate_hash = compute_legitimacy_certificate_hash(
            block_hash,
            authority_epoch,
            constitution_root,
            validator_authority_commitment,
            transition_proof_root,
            &signers,
        );

        Self {
            block_hash,
            authority_epoch,
            constitution_root,
            validator_authority_commitment,
            transition_proof_root,
            signers,
            certificate_hash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContinuityCertificate {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub epoch: u64,
    pub timeout_round: u64,
    pub signers: Vec<ValidatorId>,
    pub observed_power: u64,
    pub certificate_hash: [u8; 32],
}

impl ContinuityCertificate {
    #[must_use]
    pub fn new(
        block_hash: [u8; 32],
        height: u64,
        round: u64,
        epoch: u64,
        timeout_round: u64,
        observed_power: u64,
        mut signers: Vec<ValidatorId>,
    ) -> Self {
        signers.sort();
        signers.dedup();
        let certificate_hash = compute_continuity_certificate_hash(
            block_hash,
            height,
            round,
            epoch,
            timeout_round,
            observed_power,
            &signers,
        );

        Self {
            block_hash,
            height,
            round,
            epoch,
            timeout_round,
            signers,
            observed_power,
            certificate_hash,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstitutionalSeal {
    pub block_hash: [u8; 32],
    pub height: u64,
    pub round: u64,
    pub epoch: u64,
    pub execution_certificate_hash: [u8; 32],
    pub legitimacy_certificate_hash: [u8; 32],
    pub continuity_certificate_hash: [u8; 32],
    pub seal_hash: [u8; 32],
}

impl ConstitutionalSeal {
    pub fn compose(
        execution: &ExecutionCertificate,
        legitimacy: &LegitimacyCertificate,
        continuity: &ContinuityCertificate,
    ) -> Option<Self> {
        if execution.block_hash != legitimacy.block_hash
            || execution.block_hash != continuity.block_hash
            || execution.height != continuity.height
            || execution.round != continuity.round
            || execution.epoch != legitimacy.authority_epoch
            || execution.epoch != continuity.epoch
        {
            return None;
        }

        let seal_hash = compute_constitutional_seal_hash(
            execution.block_hash,
            execution.height,
            execution.round,
            execution.epoch,
            execution.certificate_hash,
            legitimacy.certificate_hash,
            continuity.certificate_hash,
        );

        Some(Self {
            block_hash: execution.block_hash,
            height: execution.height,
            round: execution.round,
            epoch: execution.epoch,
            execution_certificate_hash: execution.certificate_hash,
            legitimacy_certificate_hash: legitimacy.certificate_hash,
            continuity_certificate_hash: continuity.certificate_hash,
            seal_hash,
        })
    }
}

fn compute_execution_certificate_hash(
    block_hash: [u8; 32],
    height: u64,
    round: u64,
    epoch: u64,
    validator_set_hash: [u8; 32],
    quorum_certificate_hash: [u8; 32],
) -> [u8; 32] {
    compute_hash(
        EXECUTION_CERTIFICATE_DOMAIN_V1,
        &[
            &block_hash,
            &height.to_le_bytes(),
            &round.to_le_bytes(),
            &epoch.to_le_bytes(),
            &validator_set_hash,
            &quorum_certificate_hash,
        ],
    )
}

fn compute_legitimacy_certificate_hash(
    block_hash: [u8; 32],
    authority_epoch: u64,
    constitution_root: [u8; 32],
    validator_authority_commitment: [u8; 32],
    transition_proof_root: [u8; 32],
    signers: &[ValidatorId],
) -> [u8; 32] {
    let authority_epoch_bytes = authority_epoch.to_le_bytes();
    let signer_len = (signers.len() as u64).to_le_bytes();
    let mut parts: Vec<&[u8]> = vec![
        &block_hash,
        &authority_epoch_bytes,
        &constitution_root,
        &validator_authority_commitment,
        &transition_proof_root,
        &signer_len,
    ];
    for signer in signers {
        parts.push(signer);
    }
    compute_hash(LEGITIMACY_CERTIFICATE_DOMAIN_V1, &parts)
}

fn compute_continuity_certificate_hash(
    block_hash: [u8; 32],
    height: u64,
    round: u64,
    epoch: u64,
    timeout_round: u64,
    observed_power: u64,
    signers: &[ValidatorId],
) -> [u8; 32] {
    let height_bytes = height.to_le_bytes();
    let round_bytes = round.to_le_bytes();
    let epoch_bytes = epoch.to_le_bytes();
    let timeout_round_bytes = timeout_round.to_le_bytes();
    let observed_power_bytes = observed_power.to_le_bytes();
    let signer_len = (signers.len() as u64).to_le_bytes();
    let mut parts: Vec<&[u8]> = vec![
        &block_hash,
        &height_bytes,
        &round_bytes,
        &epoch_bytes,
        &timeout_round_bytes,
        &observed_power_bytes,
        &signer_len,
    ];
    for signer in signers {
        parts.push(signer);
    }
    compute_hash(CONTINUITY_CERTIFICATE_DOMAIN_V1, &parts)
}

fn compute_constitutional_seal_hash(
    block_hash: [u8; 32],
    height: u64,
    round: u64,
    epoch: u64,
    execution_certificate_hash: [u8; 32],
    legitimacy_certificate_hash: [u8; 32],
    continuity_certificate_hash: [u8; 32],
) -> [u8; 32] {
    compute_hash(
        CONSTITUTIONAL_SEAL_DOMAIN_V1,
        &[
            &block_hash,
            &height.to_le_bytes(),
            &round.to_le_bytes(),
            &epoch.to_le_bytes(),
            &execution_certificate_hash,
            &legitimacy_certificate_hash,
            &continuity_certificate_hash,
        ],
    )
}

fn compute_hash(domain: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(domain);
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use crate::seal::QuorumCertificate;

    use super::{
        ConstitutionalSeal, ContinuityCertificate, ExecutionCertificate, LegitimacyCertificate,
    };

    #[test]
    fn legitimacy_certificate_hash_is_deterministic_for_signer_order() {
        let a = LegitimacyCertificate::new(
            [1u8; 32],
            7,
            [2u8; 32],
            [3u8; 32],
            [4u8; 32],
            vec![[9u8; 32], [8u8; 32]],
        );
        let b = LegitimacyCertificate::new(
            [1u8; 32],
            7,
            [2u8; 32],
            [3u8; 32],
            [4u8; 32],
            vec![[8u8; 32], [9u8; 32]],
        );

        assert_eq!(a.certificate_hash, b.certificate_hash);
        assert_eq!(a.signers, b.signers);
    }

    #[test]
    fn constitutional_seal_requires_matching_block_and_epoch() {
        let qc = QuorumCertificate::new([5u8; 32], 11, 3, vec![[1u8; 32]], 10, 10, 2, 3);
        let execution = ExecutionCertificate::new(4, [6u8; 32], qc);
        let legitimacy = LegitimacyCertificate::new(
            [5u8; 32],
            4,
            [7u8; 32],
            [8u8; 32],
            [9u8; 32],
            vec![[1u8; 32]],
        );
        let continuity = ContinuityCertificate::new([5u8; 32], 11, 3, 5, 4, 10, vec![[1u8; 32]]);

        assert!(ConstitutionalSeal::compose(&execution, &legitimacy, &continuity).is_none());
    }

    #[test]
    fn constitutional_seal_composition_is_deterministic() {
        let qc = QuorumCertificate::new([5u8; 32], 11, 3, vec![[2u8; 32], [1u8; 32]], 20, 30, 2, 3);
        let execution = ExecutionCertificate::new(4, [6u8; 32], qc);
        let legitimacy = LegitimacyCertificate::new(
            [5u8; 32],
            4,
            [7u8; 32],
            [8u8; 32],
            [9u8; 32],
            vec![[2u8; 32], [1u8; 32]],
        );
        let continuity =
            ContinuityCertificate::new([5u8; 32], 11, 3, 4, 4, 20, vec![[2u8; 32], [1u8; 32]]);

        let a = ConstitutionalSeal::compose(&execution, &legitimacy, &continuity).unwrap();
        let b = ConstitutionalSeal::compose(&execution, &legitimacy, &continuity).unwrap();

        assert_eq!(a, b);
    }
}

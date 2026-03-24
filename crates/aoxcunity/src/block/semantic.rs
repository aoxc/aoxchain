use crate::block::types::{
    BlockBody, BlockBuildError, BlockSection, CAPABILITY_AI_ATTESTATION, CAPABILITY_CONSTITUTIONAL,
    CAPABILITY_EXECUTION, CAPABILITY_IDENTITY, CAPABILITY_PQ_ROTATION, CAPABILITY_SETTLEMENT,
    CAPABILITY_TIME_SEAL, PostQuantumSection, TimeSealSection,
};

/// Validates section-level semantic invariants for a canonical block body.
///
/// This layer is intentionally independent from hashing/canonical ordering so
/// policy checks can evolve without destabilizing deterministic hash behavior.
pub fn validate_block_semantics(
    timestamp: u64,
    crypto_epoch: u64,
    body: &BlockBody,
) -> Result<(), BlockBuildError> {
    let mut time_seal: Option<&TimeSealSection> = None;
    let mut pq_section: Option<&PostQuantumSection> = None;
    let mut ai_section_present = false;
    let mut ai_policy_hash = [0u8; 32];
    let mut ai_replay_nonce = 0u64;

    for section in &body.sections {
        match section {
            BlockSection::TimeSeal(section) => time_seal = Some(section),
            BlockSection::PostQuantum(section) => pq_section = Some(section),
            BlockSection::Ai(section) => {
                ai_section_present = true;
                ai_policy_hash = section.policy_hash;
                ai_replay_nonce = section.replay_nonce;
            }
            _ => {}
        }
    }

    if let Some(section) = time_seal {
        validate_time_seal(timestamp, section)?;
    }

    if ai_section_present {
        if ai_policy_hash == [0u8; 32] {
            return Err(BlockBuildError::AiSectionMissingPolicyHash);
        }

        if ai_replay_nonce == 0 {
            return Err(BlockBuildError::AiSectionZeroReplayNonce);
        }
    }

    if let Some(section) = pq_section {
        validate_post_quantum_policy(crypto_epoch, section)?;
    }

    Ok(())
}

fn validate_time_seal(timestamp: u64, section: &TimeSealSection) -> Result<(), BlockBuildError> {
    if section.valid_from > section.valid_until {
        return Err(BlockBuildError::InvalidTimeSealRange);
    }

    if timestamp < section.valid_from || timestamp > section.valid_until {
        return Err(BlockBuildError::TimestampOutsideTimeSealWindow);
    }

    Ok(())
}

fn validate_post_quantum_policy(
    crypto_epoch: u64,
    section: &PostQuantumSection,
) -> Result<(), BlockBuildError> {
    let policy_id = section.signature_policy_id;
    if policy_id == 0 {
        return Err(BlockBuildError::PostQuantumMissingSignaturePolicy);
    }
    if !(1..=4).contains(&policy_id) {
        return Err(BlockBuildError::PostQuantumInvalidSignaturePolicy);
    }

    const PQ_MANDATORY_START_EPOCH: u64 = 100;
    const POLICY_PQ_MANDATORY: u32 = 4;
    if crypto_epoch >= PQ_MANDATORY_START_EPOCH && policy_id != POLICY_PQ_MANDATORY {
        return Err(BlockBuildError::CryptoEpochRequiresPqMandatory);
    }
    if policy_id == POLICY_PQ_MANDATORY && !section.downgrade_prohibited {
        return Err(BlockBuildError::PqMandatoryRequiresDowngradeProtection);
    }

    Ok(())
}

pub fn validate_capability_section_alignment(
    body: &BlockBody,
    capability_flags: u64,
) -> Result<(), BlockBuildError> {
    let mut expected = 0u64;
    for section in &body.sections {
        match section {
            BlockSection::Execution(_) | BlockSection::LaneCommitment(_) => {
                expected |= CAPABILITY_EXECUTION;
            }
            BlockSection::Identity(_) => expected |= CAPABILITY_IDENTITY,
            BlockSection::ExternalProof(_) | BlockSection::ExternalSettlement(_) => {
                expected |= CAPABILITY_SETTLEMENT;
            }
            BlockSection::Ai(_) => expected |= CAPABILITY_AI_ATTESTATION,
            BlockSection::PostQuantum(_) => expected |= CAPABILITY_PQ_ROTATION,
            BlockSection::Constitutional(_) => expected |= CAPABILITY_CONSTITUTIONAL,
            BlockSection::TimeSeal(_) => expected |= CAPABILITY_TIME_SEAL,
        }
    }

    if expected != capability_flags {
        return Err(BlockBuildError::CapabilitySectionMismatch);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_block_semantics, validate_capability_section_alignment};
    use crate::block::types::{
        BlockBody, BlockBuildError, BlockSection, CAPABILITY_EXECUTION, PostQuantumSection,
        TimeSealSection,
    };

    #[test]
    fn rejects_unsupported_signature_policy_id() {
        let err = validate_block_semantics(
            10,
            1,
            &BlockBody {
                sections: vec![BlockSection::PostQuantum(PostQuantumSection {
                    scheme_registry_root: [1u8; 32],
                    signer_set_root: [2u8; 32],
                    hybrid_policy_root: [3u8; 32],
                    signature_policy_id: 9,
                    downgrade_prohibited: true,
                })],
            },
        )
        .unwrap_err();

        assert_eq!(err, BlockBuildError::PostQuantumInvalidSignaturePolicy);
    }

    #[test]
    fn rejects_capability_section_mismatch() {
        let err = validate_capability_section_alignment(
            &BlockBody {
                sections: vec![BlockSection::TimeSeal(TimeSealSection {
                    valid_from: 1,
                    valid_until: 2,
                    epoch_action_root: [0u8; 32],
                    delayed_effect_root: [0u8; 32],
                })],
            },
            CAPABILITY_EXECUTION,
        )
        .unwrap_err();

        assert_eq!(err, BlockBuildError::CapabilitySectionMismatch);
    }
}

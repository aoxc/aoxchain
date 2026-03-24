use crate::block::types::{
    BlockBody, BlockBuildError, BlockSection, PostQuantumSection, TimeSealSection,
};

/// Validates section-level semantic invariants for a canonical block body.
///
/// This layer is intentionally independent from hashing/canonical ordering so
/// policy checks can evolve without destabilizing deterministic hash behavior.
pub fn validate_block_semantics(timestamp: u64, body: &BlockBody) -> Result<(), BlockBuildError> {
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

    if let Some(section) = pq_section
        && section.signature_policy_id == 0
    {
        return Err(BlockBuildError::PostQuantumMissingSignaturePolicy);
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
